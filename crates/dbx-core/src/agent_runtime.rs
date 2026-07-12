use serde::de::DeserializeOwned;
use std::path::PathBuf;
use std::time::Duration;

use crate::agent_catalog;
use crate::agent_manager::{AgentManager, AgentState, DEFAULT_JRE_KEY};
use crate::agent_service::AgentProgressEvent;
use crate::database_capabilities;
use crate::db::agent_driver::{AgentDriverClient, AgentLaunchSpec, AgentMethod};
use crate::jdbc;
use crate::models::connection::DatabaseType;

pub fn db_type_to_agent_key(db_type: &DatabaseType, driver_profile: Option<&str>) -> Option<&'static str> {
    database_capabilities::agent_key(db_type, driver_profile)
}

pub fn is_agent_type(db_type: &DatabaseType) -> bool {
    database_capabilities::is_agent_type(db_type)
}

pub async fn stop_daemons(manager: &AgentManager) {
    manager.daemons.lock().await.clear();
}

pub async fn stop_daemon_by_key(manager: &AgentManager, agent_key: &str) {
    manager.daemons.lock().await.remove(agent_key);
}

pub async fn restart_daemon_by_key(manager: &AgentManager, agent_key: &str) -> Result<(), String> {
    manager.daemons.lock().await.remove(agent_key);
    let client = spawn_client_for_key(manager, agent_key, &[], |_| {}).await?;
    manager.daemons.lock().await.insert(agent_key.to_string(), client);
    Ok(())
}
pub async fn spawn_connection_client(
    manager: &AgentManager,
    db_type: &DatabaseType,
    driver_profile: Option<&str>,
    extra_java_args: &[String],
    progress: impl Fn(AgentProgressEvent),
) -> Result<AgentDriverClient, String> {
    let keys = runtime_agent_key_candidates(db_type, driver_profile)
        .ok_or_else(|| format!("{:?} is not an agent-driven database type", db_type))?;

    if keys.len() == 1 {
        return spawn_client_for_key(manager, keys[0], extra_java_args, progress).await;
    }

    spawn_first_available_client(manager, &keys, extra_java_args, progress).await
}

pub async fn call_daemon<T: DeserializeOwned + Send + 'static>(
    manager: &AgentManager,
    db_type: &DatabaseType,
    driver_profile: Option<&str>,
    method: &str,
    params: serde_json::Value,
    progress: impl Fn(AgentProgressEvent),
) -> Result<T, String> {
    let keys = runtime_agent_key_candidates(db_type, driver_profile)
        .ok_or_else(|| format!("{:?} is not an agent-driven database type", db_type))?;
    let key = first_installed_agent_key(manager, &keys).unwrap_or(keys[0]).to_string();

    let mut daemons = manager.daemons.lock().await;

    if !daemons.contains_key(&key) {
        let client = spawn_client_for_key(manager, &key, &[], progress).await?;
        daemons.insert(key.clone(), client);
    }

    let client = daemons.get_mut(&key).unwrap();
    match client.call::<T>(method, params.clone()).await {
        Ok(result) => Ok(result),
        Err(err) => {
            log::warn!("[agent] daemon call failed, respawning: {err}");
            daemons.remove(&key);
            let mut new_client = spawn_client_for_key(manager, &key, &[], |_| {}).await?;
            let result = new_client.call::<T>(method, params).await?;
            daemons.insert(key, new_client);
            Ok(result)
        }
    }
}

pub async fn call_daemon_with_timeout<T: DeserializeOwned + Send + 'static>(
    manager: &AgentManager,
    db_type: &DatabaseType,
    driver_profile: Option<&str>,
    method: &str,
    params: serde_json::Value,
    timeout_duration: Option<Duration>,
    progress: impl Fn(AgentProgressEvent),
) -> Result<T, String> {
    let keys = runtime_agent_key_candidates(db_type, driver_profile)
        .ok_or_else(|| format!("{:?} is not an agent-driven database type", db_type))?;
    let key = first_installed_agent_key(manager, &keys).unwrap_or(keys[0]).to_string();

    let mut daemons = manager.daemons.lock().await;

    if !daemons.contains_key(&key) {
        let client = spawn_client_for_key(manager, &key, &[], progress).await?;
        daemons.insert(key.clone(), client);
    }

    let client = daemons.get_mut(&key).unwrap();
    match client.call_with_timeout::<T>(method, params.clone(), timeout_duration).await {
        Ok(result) => Ok(result),
        Err(err) => {
            log::warn!("[agent] daemon call failed, respawning: {err}");
            daemons.remove(&key);
            let mut new_client = spawn_client_for_key(manager, &key, &[], |_| {}).await?;
            let result = new_client.call_with_timeout::<T>(method, params, timeout_duration).await?;
            daemons.insert(key, new_client);
            Ok(result)
        }
    }
}

pub async fn call_daemon_method<T: DeserializeOwned + Send + 'static>(
    manager: &AgentManager,
    db_type: &DatabaseType,
    driver_profile: Option<&str>,
    method: AgentMethod,
    params: serde_json::Value,
    progress: impl Fn(AgentProgressEvent),
) -> Result<T, String> {
    call_daemon(manager, db_type, driver_profile, method.as_str(), params, progress).await
}

pub async fn call_daemon_method_with_timeout<T: DeserializeOwned + Send + 'static>(
    manager: &AgentManager,
    db_type: &DatabaseType,
    driver_profile: Option<&str>,
    method: AgentMethod,
    params: serde_json::Value,
    timeout_duration: Option<Duration>,
    progress: impl Fn(AgentProgressEvent),
) -> Result<T, String> {
    call_daemon_with_timeout(manager, db_type, driver_profile, method.as_str(), params, timeout_duration, progress)
        .await
}

fn runtime_agent_key_candidates(db_type: &DatabaseType, driver_profile: Option<&str>) -> Option<Vec<&'static str>> {
    let primary = db_type_to_agent_key(db_type, driver_profile)?;
    Some(vec![primary])
}

fn first_installed_agent_key<'a>(manager: &AgentManager, keys: &'a [&'static str]) -> Option<&'a str> {
    keys.iter().copied().find(|key| manager.is_driver_installed(key))
}
async fn spawn_first_available_client(
    manager: &AgentManager,
    keys: &[&'static str],
    extra_java_args: &[String],
    progress: impl Fn(AgentProgressEvent),
) -> Result<AgentDriverClient, String> {
    let mut last_error = None;
    for key in keys {
        match spawn_client_for_key(manager, key, extra_java_args, &progress).await {
            Ok(client) => return Ok(client),
            Err(err) => last_error = Some(err),
        }
    }
    Err(last_error.unwrap_or_else(|| "No agent driver candidates available".to_string()))
}

/// Check whether a given agent key is a JDBC bridge type (Maven-resolved driver).
fn is_jdbc_bridge_key(key: &str) -> bool {
    agent_catalog::is_jdbc_bridge_key(key, None)
}

/// The agent driver lib directory for a JDBC bridge profile.
///   ~/.dbx/agents/drivers/{key}/lib/
fn jdbc_bridge_lib_dir(manager: &AgentManager, key: &str) -> PathBuf {
    manager.driver_dir(key).join("lib")
}

/// Check whether the JDBC bridge driver JARs are installed.
fn is_jdbc_bridge_driver_installed(manager: &AgentManager, key: &str) -> bool {
    let lib_dir = jdbc_bridge_lib_dir(manager, key);
    if !lib_dir.exists() {
        return false;
    }
    // Consider installed if the lib directory has at least one JAR file.
    std::fs::read_dir(&lib_dir)
        .ok()
        .map(|entries| entries.flatten().any(|e| e.path().extension().is_some_and(|ext| ext == "jar")))
        .unwrap_or(false)
}

/// Ensure the JDBC bridge driver JARs are downloaded from Maven Central.
/// Downloads to the JDBC plugin's Maven cache, then copies to the agent's
/// driver lib directory.
async fn ensure_jdbc_bridge_driver_installed(
    manager: &AgentManager,
    key: &str,
    progress: impl Fn(AgentProgressEvent),
) -> Result<(), String> {
    let profile = agent_catalog::profile_for_key(key, None)
        .ok_or_else(|| format!("No JDBC bridge profile found for key: {key}"))?;

    let coordinate = profile.maven_coordinate;
    if coordinate.is_empty() {
        return Err(format!("No Maven coordinate configured for key: {key}"));
    }
    progress(AgentProgressEvent::step("maven-resolve").with_batch(Some(key), None, None));

    let plugins_root = manager.plugins_root();
    let plugin_dir = plugins_root.join("jdbc");
    let resolver = crate::jdbc::jdbc_maven_resolver_executable(&plugin_dir);
    if !resolver.exists() {
        return Err("JDBC Maven resolver is not installed. Please install the JDBC plugin first.".to_string());
    }

    let local_repo = plugin_dir.join("maven-cache");
    std::fs::create_dir_all(&local_repo).map_err(|err| err.to_string())?;

    // Run the Maven resolver to download the driver JARs.
    let mut command = crate::process::new_tokio_command(&resolver);
    command
        .arg("resolve")
        .arg("--coordinate")
        .arg(coordinate)
        .arg("--local-repo")
        .arg(&local_repo)
        .arg("--repo")
        .arg("https://repo.maven.apache.org/maven2/");

    let output = command.output().await.map_err(|err| format!("Failed to run JDBC Maven resolver: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        progress(AgentProgressEvent::step("error").with_batch(Some(key), None, None));
        return Err(if stderr.is_empty() { "JDBC Maven resolver failed".to_string() } else { stderr.to_string() });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let resolved: jdbc::MavenResolveOutput =
        serde_json::from_str(&stdout).map_err(|err| format!("Failed to parse JDBC Maven resolver output: {err}"))?;

    // Copy the downloaded JARs to the agent's driver lib directory.
    let lib_dir = jdbc_bridge_lib_dir(manager, key);
    std::fs::create_dir_all(&lib_dir).map_err(|err| err.to_string())?;

    for artifact in &resolved.artifacts {
        if !artifact.extension.eq_ignore_ascii_case("jar") {
            continue;
        }
        let source = PathBuf::from(&artifact.file);
        if !source.exists() {
            progress(AgentProgressEvent::step("error").with_batch(Some(key), None, None));
            return Err(format!("Resolved artifact file does not exist: {}", source.display()));
        }
        let file_name = source
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("Invalid resolved artifact path: {}", source.display()))?;
        let target = lib_dir.join(file_name);
        if target.exists() {
            continue;
        }
        std::fs::copy(&source, &target).map_err(|err| err.to_string())?;
    }

    // Validate that at least one JAR was copied.
    let jar_count = std::fs::read_dir(&lib_dir)
        .ok()
        .map(|entries| entries.flatten().filter(|e| e.path().extension().is_some_and(|ext| ext == "jar")).count())
        .unwrap_or(0);
    if jar_count == 0 {
        progress(AgentProgressEvent::step("error").with_batch(Some(key), None, None));
        return Err(format!("Maven resolver did not produce any JAR files for {coordinate}"));
    }

    // Update state.json to record the installation.
    let mut state = manager.load_state();
    state.installed_drivers.insert(
        key.to_string(),
        crate::agent_manager::InstalledDriver {
            version: resolved.coordinate.clone(),
            installed_at: chrono::Utc::now().to_rfc3339(),
            jre: DEFAULT_JRE_KEY.to_string(),
        },
    );
    manager.save_state(&state)?;

    progress(AgentProgressEvent::step("done").with_batch(Some(key), None, None));
    Ok(())
}

/// Build the launch spec for the JDBC bridge agent.
/// The classpath includes the jdbc-bridge JAR and all driver JARs in the lib directory.
fn resolve_jdbc_bridge_launch_spec(
    manager: &AgentManager,
    state: &AgentState,
    key: &str,
    jre_key: &str,
    extra_java_args: &[String],
) -> Result<AgentLaunchSpec, String> {
    let profile = agent_catalog::profile_for_key(key, None)
        .ok_or_else(|| format!("No JDBC bridge profile found for key: {key}"))?;

    let java = manager.resolve_java_runtime(state, jre_key)?;
    let bridge_jar = manager.driver_dir("jdbc-bridge").join("agent.jar");
    if !bridge_jar.exists() {
        return Err("JDBC bridge agent is not installed. Please install the jdbc-bridge agent first.".to_string());
    }
    let lib_dir = jdbc_bridge_lib_dir(manager, key);

    // Build the classpath: jdbc-bridge.jar + all driver JARs.
    let mut classpath = bridge_jar.to_string_lossy().to_string();
    if lib_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&lib_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "jar") {
                    let sep = if cfg!(windows) { ';' } else { ':' };
                    classpath.push(sep);
                    classpath.push_str(&path.to_string_lossy());
                }
            }
        }
    }

    // Build Java args.
    let mut args = vec![
        "-Dfile.encoding=UTF-8".to_string(),
        "-Dsun.stdout.encoding=UTF-8".to_string(),
        "-Dsun.stderr.encoding=UTF-8".to_string(),
        "-Djava.net.useSystemProxies=false".to_string(),
        "--add-opens=java.sql/java.sql=ALL-UNNAMED".to_string(),
    ];
    args.extend(extra_java_args.iter().cloned());
    args.push("-cp".to_string());
    args.push(classpath);
    args.push("com.dbx.agent.jdbcbridge.JdbcBridge".to_string());
    args.push(profile.driver_class.to_string());
    args.push(profile.jdbc_url_template.to_string());

    Ok(AgentLaunchSpec::new(java).with_args(args))
}

async fn spawn_client_for_key(
    manager: &AgentManager,
    key: &str,
    extra_java_args: &[String],
    progress: impl Fn(AgentProgressEvent),
) -> Result<AgentDriverClient, String> {
    // For JDBC bridge types, ensure the driver is downloaded, then launch
    // the jdbc-bridge agent with the driver JARs on the classpath.
    if is_jdbc_bridge_key(key) {
        if !is_jdbc_bridge_driver_installed(manager, key) {
            ensure_jdbc_bridge_driver_installed(manager, key, progress).await?;
        }

        let state = manager.load_state();
        let jre_key = state.installed_drivers.get(key).map(|driver| driver.jre.as_str()).unwrap_or(DEFAULT_JRE_KEY);
        let launch = resolve_jdbc_bridge_launch_spec(manager, &state, key, jre_key, extra_java_args)?;
        let mut client = AgentDriverClient::spawn(launch).await?;
        client.try_optional_handshake(manager.agent_app_version()).await;
        return Ok(client);
    }

    // Standard agent driver flow.
    let state = manager.load_state();
    let jre_key = state.installed_drivers.get(key).map(|driver| driver.jre.as_str()).unwrap_or(DEFAULT_JRE_KEY);

    let launch = manager.resolve_agent_launch_spec_with_extra_args(&state, key, jre_key, extra_java_args)?;
    let mut client = AgentDriverClient::spawn(launch).await?;
    client.try_optional_handshake(manager.agent_app_version()).await;
    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prestosql_does_not_use_agent_driver() {
        assert_eq!(runtime_agent_key_candidates(&DatabaseType::PrestoSql, None), None);
    }

    #[test]
    fn trino_uses_only_trino_agent_driver() {
        assert_eq!(runtime_agent_key_candidates(&DatabaseType::Trino, None).unwrap(), vec!["trino"]);
    }

    #[test]
    fn jdbc_bridge_types_are_detected() {
        assert!(agent_catalog::is_jdbc_bridge_type(&DatabaseType::H2));
        assert!(agent_catalog::is_jdbc_bridge_type(&DatabaseType::Hive));
        assert!(agent_catalog::is_jdbc_bridge_type(&DatabaseType::Trino));
        assert!(agent_catalog::is_jdbc_bridge_type(&DatabaseType::Spark));
        assert!(agent_catalog::is_jdbc_bridge_type(&DatabaseType::Db2));
        assert!(!agent_catalog::is_jdbc_bridge_type(&DatabaseType::Dameng));
        assert!(!agent_catalog::is_jdbc_bridge_type(&DatabaseType::Kingbase));
    }

    #[test]
    fn match_profile_by_url_works() {
        let hive2 = agent_catalog::match_profile_by_url(&DatabaseType::Hive, "jdbc:hive2://host:10000/db");
        assert!(hive2.is_some());
        assert_eq!(hive2.unwrap().profile, "hive2");

        let hive1 = agent_catalog::match_profile_by_url(&DatabaseType::Hive, "jdbc:hive://host:10000/db");
        assert!(hive1.is_some());
        assert_eq!(hive1.unwrap().profile, "hive1");

        // Trino URL
        let trino = agent_catalog::match_profile_by_url(&DatabaseType::Trino, "jdbc:trino://host:8080/db");
        assert!(trino.is_some());
        assert_eq!(trino.unwrap().profile, "trino");
    }
}
