use std::collections::HashSet;

use crate::models::connection::DatabaseType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgentCatalogEntry {
    pub db_type: DatabaseType,
    pub key: &'static str,
    pub label: &'static str,
    pub store_visible: bool,
    pub profiles: &'static [AgentDriverProfile],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentDriverProfile {
    pub profile: &'static str,
    pub key: &'static str,
    pub label: &'static str,
    pub store_visible: bool,
    // JDBC auto-download fields (empty strings when not applicable)
    pub maven_coordinate: &'static str,
    pub driver_class: &'static str,
    pub jdbc_url_template: &'static str,
    pub jdbc_url_prefix: &'static str,
    pub default_port: u16,
    pub is_default: bool,
}

// ---------------------------------------------------------------------------
// Profile constants
// ---------------------------------------------------------------------------

const ORACLE_PROFILES: &[AgentDriverProfile] = &[
    AgentDriverProfile {
        profile: "oracle-legacy", key: "oracle", label: "Oracle", store_visible: false,
        maven_coordinate: "", driver_class: "", jdbc_url_template: "", jdbc_url_prefix: "",
        default_port: 0, is_default: false,
    },
    AgentDriverProfile {
        profile: "oracle-10g", key: "oracle", label: "Oracle", store_visible: false,
        maven_coordinate: "", driver_class: "", jdbc_url_template: "", jdbc_url_prefix: "",
        default_port: 0, is_default: false,
    },
];

const GBASE_PROFILES: &[AgentDriverProfile] = &[
    AgentDriverProfile {
        profile: "gbase8s", key: "gbase8s", label: "GBase 8s", store_visible: true,
        maven_coordinate: "", driver_class: "", jdbc_url_template: "", jdbc_url_prefix: "",
        default_port: 0, is_default: false,
    },
    AgentDriverProfile {
        profile: "gbase8a", key: "gbase8a", label: "GBase 8a", store_visible: true,
        maven_coordinate: "", driver_class: "", jdbc_url_template: "", jdbc_url_prefix: "",
        default_port: 0, is_default: false,
    },
];

const MONGODB_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "mongodb-legacy", key: "mongodb", label: "MongoDB (Legacy)", store_visible: false,
    maven_coordinate: "", driver_class: "", jdbc_url_template: "", jdbc_url_prefix: "",
    default_port: 0, is_default: false,
}];

// JDBC bridge profiles — databases whose drivers are available on Maven Central.
// The `key` for all JDBC bridge profiles is the database type key (e.g. "hive"),
// so the runtime launches the generic jdbc-bridge agent with the driver JARs
// downloaded from Maven.

const H2_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "h2", key: "h2", label: "H2", store_visible: true,
    maven_coordinate: "com.h2database:h2:2.3.232",
    driver_class: "org.h2.Driver",
    jdbc_url_template: "jdbc:h2:tcp://{host}:{port}/{database}",
    jdbc_url_prefix: "jdbc:h2:",
    default_port: 9092,
    is_default: true,
}];

const HIVE_PROFILES: &[AgentDriverProfile] = &[
    AgentDriverProfile {
        profile: "hive2", key: "hive", label: "Hive 2.x", store_visible: true,
        maven_coordinate: "org.apache.hive:hive-jdbc:4.0.0",
        driver_class: "org.apache.hive.jdbc.HiveDriver",
        jdbc_url_template: "jdbc:hive2://{host}:{port}/{database}",
        jdbc_url_prefix: "jdbc:hive2:",
        default_port: 10000,
        is_default: true,
    },
    AgentDriverProfile {
        profile: "hive1", key: "hive", label: "Hive 1.x", store_visible: true,
        maven_coordinate: "org.apache.hive:hive-jdbc:1.2.2",
        driver_class: "org.apache.hadoop.hive.jdbc.HiveDriver",
        jdbc_url_template: "jdbc:hive://{host}:{port}/{database}",
        jdbc_url_prefix: "jdbc:hive:",
        default_port: 10000,
        is_default: false,
    },
];

const SPARK_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "spark", key: "spark", label: "Apache Spark", store_visible: true,
    maven_coordinate: "org.apache.spark:spark-hive-thriftserver_2.12:3.5.5",
    driver_class: "org.apache.hive.jdbc.HiveDriver",
    jdbc_url_template: "jdbc:hive2://{host}:{port}/{database}",
    jdbc_url_prefix: "jdbc:hive2:",
    default_port: 10000,
    is_default: true,
}];

const TRINO_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "trino", key: "trino", label: "Trino", store_visible: true,
    maven_coordinate: "io.trino:trino-jdbc:470",
    driver_class: "io.trino.jdbc.TrinoDriver",
    jdbc_url_template: "jdbc:trino://{host}:{port}/{database}",
    jdbc_url_prefix: "jdbc:trino:",
    default_port: 8080,
    is_default: true,
}];

const DB2_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "db2", key: "db2", label: "IBM DB2", store_visible: true,
    maven_coordinate: "com.ibm.db2:jcc:11.5.9.0",
    driver_class: "com.ibm.db2.jcc.DB2Driver",
    jdbc_url_template: "jdbc:db2://{host}:{port}/{database}",
    jdbc_url_prefix: "jdbc:db2:",
    default_port: 50000,
    is_default: true,
}];

const INFORMIX_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "informix", key: "informix", label: "IBM Informix", store_visible: true,
    maven_coordinate: "com.ibm.informix:ifxjdbc:4.50.10.0",
    driver_class: "com.informix.jdbc.IfxDriver",
    jdbc_url_template: "jdbc:informix-sqli://{host}:{port}/{database}:informixserver={host}",
    jdbc_url_prefix: "jdbc:informix-sqli:",
    default_port: 9088,
    is_default: true,
}];

const SNOWFLAKE_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "snowflake", key: "snowflake", label: "Snowflake", store_visible: true,
    maven_coordinate: "net.snowflake:snowflake-jdbc:3.22.1",
    driver_class: "net.snowflake.client.jdbc.SnowflakeDriver",
    jdbc_url_template: "jdbc:snowflake://{host}:{port}/{database}",
    jdbc_url_prefix: "jdbc:snowflake:",
    default_port: 443,
    is_default: true,
}];

const CLICKHOUSE_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "clickhouse", key: "clickhouse", label: "ClickHouse", store_visible: true,
    maven_coordinate: "com.clickhouse:clickhouse-jdbc:0.7.2",
    driver_class: "com.clickhouse.jdbc.ClickHouseDriver",
    jdbc_url_template: "jdbc:clickhouse://{host}:{port}/{database}",
    jdbc_url_prefix: "jdbc:clickhouse:",
    default_port: 8123,
    is_default: true,
}];

const REDSHIFT_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "redshift", key: "redshift", label: "Redshift", store_visible: true,
    maven_coordinate: "com.amazon.redshift:redshift-jdbc42:2.1.0.12",
    driver_class: "com.amazon.redshift.jdbc.Driver",
    jdbc_url_template: "jdbc:redshift://{host}:{port}/{database}",
    jdbc_url_prefix: "jdbc:redshift:",
    default_port: 5439,
    is_default: true,
}];

const BIGQUERY_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "bigquery", key: "bigquery", label: "Google BigQuery", store_visible: true,
    maven_coordinate: "com.google.cloud:google-cloud-bigquery-jdbc:2.48.0",
    driver_class: "com.google.cloud.bigquery.jdbc.BigQueryDriver",
    jdbc_url_template: "jdbc:bigquery://{host}:{port}/{database}",
    jdbc_url_prefix: "jdbc:bigquery:",
    default_port: 443,
    is_default: true,
}];

const VERTICA_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "vertica", key: "vertica", label: "Vertica", store_visible: true,
    maven_coordinate: "com.vertica:vertica-jdbc:24.4.0",
    driver_class: "com.vertica.jdbc.Driver",
    jdbc_url_template: "jdbc:vertica://{host}:{port}/{database}",
    jdbc_url_prefix: "jdbc:vertica:",
    default_port: 5433,
    is_default: true,
}];

const EXASOL_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "exasol", key: "exasol", label: "Exasol", store_visible: true,
    maven_coordinate: "com.exasol:exasol-jdbc:7.1.30",
    driver_class: "com.exasol.jdbc.EXADriver",
    jdbc_url_template: "jdbc:exa:{host}:{port}",
    jdbc_url_prefix: "jdbc:exa:",
    default_port: 8563,
    is_default: true,
}];

const SAPHANA_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "saphana", key: "saphana", label: "SAP HANA", store_visible: true,
    maven_coordinate: "com.sap.cloud.db.jdbc:ngdbc:2.25.13",
    driver_class: "com.sap.db.jdbc.Driver",
    jdbc_url_template: "jdbc:sap://{host}:{port}/{database}",
    jdbc_url_prefix: "jdbc:sap:",
    default_port: 39041,
    is_default: true,
}];

const TERADATA_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "teradata", key: "teradata", label: "Teradata", store_visible: true,
    maven_coordinate: "com.teradata:terajdbc:20.0.00.17",
    driver_class: "com.teradata.jdbc.TeraDriver",
    jdbc_url_template: "jdbc:teradata://{host}:{port}/{database}",
    jdbc_url_prefix: "jdbc:teradata:",
    default_port: 1025,
    is_default: true,
}];

const FIREBIRD_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "firebird", key: "firebird", label: "Firebird", store_visible: true,
    maven_coordinate: "org.firebirdsql.jdbc:jaybird:6.0.0",
    driver_class: "org.firebirdsql.jdbc.FBDriver",
    jdbc_url_template: "jdbc:firebirdsql://{host}:{port}/{database}",
    jdbc_url_prefix: "jdbc:firebirdsql:",
    default_port: 3050,
    is_default: true,
}];

const NEO4J_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "neo4j", key: "neo4j", label: "Neo4j", store_visible: true,
    maven_coordinate: "org.neo4j:neo4j-jdbc-driver:6.0.0",
    driver_class: "org.neo4j.jdbc.Driver",
    jdbc_url_template: "jdbc:neo4j://{host}:{port}/{database}",
    jdbc_url_prefix: "jdbc:neo4j:",
    default_port: 7687,
    is_default: true,
}];

const CASSANDRA_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "cassandra", key: "cassandra", label: "Apache Cassandra", store_visible: true,
    maven_coordinate: "com.dbschema:CassandraJdbcDriver:2.2.0",
    driver_class: "com.dbschema.CassandraJdbcDriver",
    jdbc_url_template: "jdbc:cassandra://{host}:{port}/{database}",
    jdbc_url_prefix: "jdbc:cassandra:",
    default_port: 9042,
    is_default: true,
}];

const KYLIN_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "kylin", key: "kylin", label: "Apache Kylin", store_visible: true,
    maven_coordinate: "org.apache.kylin:kylin-jdbc:5.0.0",
    driver_class: "org.apache.kylin.jdbc.Driver",
    jdbc_url_template: "jdbc:kylin://{host}:{port}/{database}",
    jdbc_url_prefix: "jdbc:kylin:",
    default_port: 7070,
    is_default: true,
}];

const DATABRICKS_PROFILES: &[AgentDriverProfile] = &[AgentDriverProfile {
    profile: "databricks", key: "databricks", label: "Databricks SQL", store_visible: true,
    maven_coordinate: "com.databricks:databricks-jdbc:2.6.39",
    driver_class: "com.databricks.client.jdbc.Driver",
    jdbc_url_template: "jdbc:databricks://{host}:{port}/{database}",
    jdbc_url_prefix: "jdbc:databricks:",
    default_port: 443,
    is_default: true,
}];

// ---------------------------------------------------------------------------
// Extra entries
// ---------------------------------------------------------------------------

const EXTRA_AGENT_LABELS: &[(&str, &str)] =
    &[("kafka", "Apache Kafka"), ("sqlserver-legacy", "SQL Server legacy compatibility component")];
const EXTRA_DRIVER_STORE_ENTRIES: &[(&str, &str)] =
    &[("kafka", "Apache Kafka"), ("sqlserver-legacy", "SQL Server legacy compatibility component")];

// ---------------------------------------------------------------------------
// Main catalog
// ---------------------------------------------------------------------------

const AGENT_CATALOG: &[AgentCatalogEntry] = &[
    AgentCatalogEntry {
        db_type: DatabaseType::Dameng,
        key: "dameng",
        label: "达梦 DM8",
        store_visible: true,
        profiles: &[],
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Kingbase,
        key: "kingbase",
        label: "人大金仓 KingbaseES",
        store_visible: true,
        profiles: &[],
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Highgo,
        key: "highgo",
        label: "瀚高 HighGo",
        store_visible: true,
        profiles: &[],
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Vastbase,
        key: "vastbase",
        label: "Vastbase",
        store_visible: true,
        profiles: &[],
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Goldendb,
        key: "goldendb",
        label: "GoldenDB",
        store_visible: true,
        profiles: &[],
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Databend,
        key: "databend",
        label: "Databend",
        store_visible: true,
        profiles: &[],
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Databricks,
        key: "databricks",
        label: "Databricks SQL",
        store_visible: true,
        profiles: DATABRICKS_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::SapHana,
        key: "saphana",
        label: "SAP HANA",
        store_visible: true,
        profiles: SAPHANA_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Teradata,
        key: "teradata",
        label: "Teradata",
        store_visible: true,
        profiles: TERADATA_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Vertica,
        key: "vertica",
        label: "Vertica",
        store_visible: true,
        profiles: VERTICA_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Firebird,
        key: "firebird",
        label: "Firebird",
        store_visible: true,
        profiles: FIREBIRD_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Exasol,
        key: "exasol",
        label: "Exasol",
        store_visible: true,
        profiles: EXASOL_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::OceanbaseOracle,
        key: "oceanbase-oracle",
        label: "OceanBase Oracle Mode",
        store_visible: true,
        profiles: &[],
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Gbase,
        key: "gbase8a",
        label: "GBase 8a",
        store_visible: true,
        profiles: GBASE_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Access,
        key: "access",
        label: "Microsoft Access",
        store_visible: true,
        profiles: &[],
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Oracle,
        key: "oracle",
        label: "Oracle",
        store_visible: true,
        profiles: ORACLE_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::H2,
        key: "h2",
        label: "H2",
        store_visible: true,
        profiles: H2_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Snowflake,
        key: "snowflake",
        label: "Snowflake",
        store_visible: true,
        profiles: SNOWFLAKE_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Trino,
        key: "trino",
        label: "Trino",
        store_visible: true,
        profiles: TRINO_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Hive,
        key: "hive",
        label: "Apache Hive",
        store_visible: true,
        profiles: HIVE_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Spark,
        key: "spark",
        label: "Apache Spark",
        store_visible: true,
        profiles: SPARK_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Db2,
        key: "db2",
        label: "IBM DB2",
        store_visible: true,
        profiles: DB2_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Informix,
        key: "informix",
        label: "IBM Informix",
        store_visible: true,
        profiles: INFORMIX_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::InfluxDb,
        key: "influxdb",
        label: "InfluxDB",
        store_visible: false,
        profiles: &[],
    },
    AgentCatalogEntry {
        db_type: DatabaseType::ClickHouse,
        key: "clickhouse",
        label: "ClickHouse",
        store_visible: true,
        profiles: CLICKHOUSE_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Redshift,
        key: "redshift",
        label: "Redshift",
        store_visible: true,
        profiles: REDSHIFT_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Neo4j,
        key: "neo4j",
        label: "Neo4j",
        store_visible: true,
        profiles: NEO4J_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Cassandra,
        key: "cassandra",
        label: "Apache Cassandra",
        store_visible: true,
        profiles: CASSANDRA_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Bigquery,
        key: "bigquery",
        label: "Google BigQuery",
        store_visible: true,
        profiles: BIGQUERY_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Kylin,
        key: "kylin",
        label: "Apache Kylin",
        store_visible: true,
        profiles: KYLIN_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Sundb,
        key: "sundb",
        label: "SunDB",
        store_visible: true,
        profiles: &[],
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Oscar,
        key: "oscar",
        label: "神通 OSCAR",
        store_visible: true,
        profiles: &[],
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Yashandb,
        key: "yashandb",
        label: "崖山 YashanDB",
        store_visible: true,
        profiles: &[],
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Tdengine,
        key: "tdengine",
        label: "TDengine",
        store_visible: true,
        profiles: &[],
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Xugu,
        key: "xugu",
        label: "虚谷 XuguDB",
        store_visible: true,
        profiles: &[],
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Iotdb,
        key: "iotdb",
        label: "Apache IoTDB",
        store_visible: true,
        profiles: &[],
    },
    AgentCatalogEntry { db_type: DatabaseType::Etcd, key: "etcd", label: "etcd", store_visible: true, profiles: &[] },
    AgentCatalogEntry {
        db_type: DatabaseType::ZooKeeper,
        key: "zookeeper",
        label: "Apache ZooKeeper",
        store_visible: true,
        profiles: &[],
    },
    AgentCatalogEntry {
        db_type: DatabaseType::MongoDb,
        key: "mongodb",
        label: "MongoDB (Legacy)",
        store_visible: true,
        profiles: MONGODB_PROFILES,
    },
    AgentCatalogEntry {
        db_type: DatabaseType::Iris,
        key: "iris",
        label: "InterSystems IRIS",
        store_visible: true,
        profiles: &[],
    },
];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn entries() -> &'static [AgentCatalogEntry] {
    AGENT_CATALOG
}

pub fn agent_key(db_type: &DatabaseType, driver_profile: Option<&str>) -> Option<&'static str> {
    if *db_type == DatabaseType::MessageQueue {
        return (driver_profile == Some("kafka")).then_some("kafka");
    }
    if *db_type == DatabaseType::SqlServer {
        return driver_profile
            .is_some_and(|profile| profile.eq_ignore_ascii_case("sqlserver-legacy"))
            .then_some("sqlserver-legacy");
    }
    let entry = entry_for_db_type(db_type)?;
    if let Some(driver_profile) = driver_profile {
        if let Some(profile) = entry.profiles.iter().find(|profile| profile.profile == driver_profile) {
            return Some(profile.key);
        }
    }
    Some(entry.key)
}

pub fn is_agent_type(db_type: &DatabaseType) -> bool {
    entry_for_db_type(db_type).is_some()
}

pub fn driver_store_entries() -> impl Iterator<Item = (&'static str, &'static str)> {
    let mut seen = HashSet::new();
    entries()
        .iter()
        .flat_map(move |entry| {
            let base = entry.store_visible.then_some((entry.key, entry.label));
            let profiles = entry
                .profiles
                .iter()
                .filter(|profile| profile.store_visible)
                .map(|profile| (profile.key, profile.label));
            base.into_iter().chain(profiles)
        })
        .chain(EXTRA_DRIVER_STORE_ENTRIES.iter().copied())
        .filter(move |(key, _)| seen.insert(*key))
}

pub fn label_for_key(agent_key: &str) -> Option<&'static str> {
    if let Some((_, label)) = EXTRA_AGENT_LABELS.iter().find(|(key, _)| *key == agent_key) {
        return Some(label);
    }
    for entry in entries() {
        if entry.key == agent_key {
            return Some(entry.label);
        }
        if let Some(profile) = entry.profiles.iter().find(|profile| profile.key == agent_key) {
            return Some(profile.label);
        }
    }
    None
}

/// Return the profile matching the given URL prefix for a database type.
/// If no profile matches, returns the default profile (is_default = true).
pub fn match_profile_by_url(db_type: &DatabaseType, jdbc_url: &str) -> Option<&'static AgentDriverProfile> {
    let entry = entry_for_db_type(db_type)?;
    if entry.profiles.is_empty() {
        return None;
    }
    // Try to match by URL prefix first.
    for profile in entry.profiles {
        if !profile.jdbc_url_prefix.is_empty() && jdbc_url.starts_with(profile.jdbc_url_prefix) {
            return Some(profile);
        }
    }
    // Fall back to the default profile.
    entry.profiles.iter().find(|p| p.is_default).or_else(|| Some(&entry.profiles[0]))
}

/// Return a specific profile by its profile name for a given database type.
pub fn get_profile(db_type: &DatabaseType, profile_name: &str) -> Option<&'static AgentDriverProfile> {
    let entry = entry_for_db_type(db_type)?;
    entry.profiles.iter().find(|p| p.profile == profile_name)
}

/// Return the default profile for a database type.
pub fn default_profile(db_type: &DatabaseType) -> Option<&'static AgentDriverProfile> {
    let entry = entry_for_db_type(db_type)?;
    entry.profiles.iter().find(|p| p.is_default).or(entry.profiles.first())
}

/// Return the profile for a given key (database-type key) and optional profile name.
pub fn profile_for_key(key: &str, profile_name: Option<&str>) -> Option<&'static AgentDriverProfile> {
    let entry = entries().iter().find(|e| e.key == key)?;
    if let Some(name) = profile_name {
        entry.profiles.iter().find(|p| p.profile == name)
    } else {
        entry.profiles.iter().find(|p| p.is_default).or(entry.profiles.first())
    }
}

/// Check whether a database type uses the JDBC bridge (i.e. has a Maven coordinate).
pub fn is_jdbc_bridge_type(db_type: &DatabaseType) -> bool {
    entry_for_db_type(db_type)
        .map(|entry| entry.profiles.iter().any(|p| !p.maven_coordinate.is_empty()))
        .unwrap_or(false)
}

/// Check whether a key/profile pair has a Maven coordinate (i.e. uses the JDBC bridge).
pub fn is_jdbc_bridge_key(key: &str, profile_name: Option<&str>) -> bool {
    profile_for_key(key, profile_name)
        .map(|p| !p.maven_coordinate.is_empty())
        .unwrap_or(false)
}

fn entry_for_db_type(db_type: &DatabaseType) -> Option<&'static AgentCatalogEntry> {
    entries().iter().find(|entry| entry.db_type == *db_type)
}