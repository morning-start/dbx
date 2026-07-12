# PIP-0002: JDBC 驱动自动下载

## 状态

Accepted

## 摘要

使 DBX 用户通过 JDBC 连接数据库时，驱动从 Maven Central 自动下载。核心思路是复用已有的 `DbxMavenResolver`，用一个通用 JDBC 桥接 Agent（`dbx-jdbc-bridge`）替代多数据库定制 Agent JAR，并根据 JDBC URL 前缀自动匹配驱动版本。

## 背景

当前 DBX 驱动分两类：

- **原生驱动**：编译进 Rust 二进制（MySQL、PostgreSQL、SQLite、Redis 等）。
- **定制 Agent JAR**：每种数据库一个独立 JAR（Oracle、达梦、Hive、Spark 等 30+ 种），通过 Java 子进程运行。

对 JDBC 类型数据库，当前流程要求用户先手动安装 Agent JAR 才能连接，打断了"填写 → 连接"的单一心流。

已有 JDBC Plugin（`plugins/jdbc/`）内含 `DbxMavenResolver`（基于 Eclipse Aether），可从 Maven Central 解析下载 JDBC JAR，但此前仅用于 PrestoSQL 和通用 JDBC 连接，Agent 体系未复用。

## 范围

- **本次范围**：JDBC 驱动自动下载。已在 `agent_catalog.rs` 中配置 JDBC bridge profiles 的数据库类型（见下）。
- **不在范围**：原生驱动（MySQL、PostgreSQL 等）、国产数据库（达梦、金仓、虚谷等，驱动不在 Maven Central 上）。
- **Oracle**：当前 `ORACLE_PROFILES` 中 `maven_coordinate` 为空。Oracle JDBC 驱动不在 Maven Central 公开仓库中，不纳入自动下载范围。
- **保留能力**：用户仍可通过驱动管理界面手动导入 JDBC JAR 文件。

## 架构

```
┌──────────────────────────────┐
│  Frontend (Vue)              │
│  ConnectionDialog            │
│  DriverStoreDialog           │
│  api.ts / tauri.ts           │
├──────────────────────────────┤
│  Tauri Commands (Rust)       │
│  connection.rs               │
│  agents.rs                   │
├──────────────────────────────┤
│  Core (Rust)                 │
│                              │
│  agent_catalog.rs            │
│    ├─ AgentDriverProfile     │
│    ├─ match_profile_by_url() │
│    └─ is_jdbc_bridge_type()  │
│                              │
│  agent_runtime.rs            │
│    ├─ spawn_client_for_key() │
│    ├─ ensure_jdbc_bridge_    │
│    │  driver_installed()     │
│    └─ resolve_jdbc_bridge_   │
│       launch_spec()          │
│                              │
│  jdbc.rs                     │
│    └─ install_jdbc_driver_   │
│       from_maven()           │
│                              │
│  agent_service.rs            │
│    └─ AgentProgressEvent     │
├──────────────────────────────┤
│  Java Subprocess             │
│                              │
│  DbxMavenResolver            │
│    └─ 解析 Maven 坐标 → JAR  │
│                              │
│  JdbcBridge                  │
│    └─ 动态加载 JDBC 驱动     │
└──────────────────────────────┘
```

## 关键接口

### AgentDriverProfile（catalog）

```rust
pub struct AgentDriverProfile {
    pub profile: &'static str,            // "hive2"
    pub key: &'static str,                // "hive"
    pub label: &'static str,              // "Hive 2.x"
    pub store_visible: bool,
    pub maven_coordinate: &'static str,   // "org.apache.hive:hive-jdbc:4.0.0"
    pub driver_class: &'static str,       // "org.apache.hive.jdbc.HiveDriver"
    pub jdbc_url_template: &'static str,  // "jdbc:hive2://{host}:{port}/{database}"
    pub jdbc_url_prefix: &'static str,    // "jdbc:hive2:" — 用于自动匹配
    pub default_port: u16,
    pub is_default: bool,
}
```

### 自动匹配流程

```
用户输入 JDBC URL — jdbc:hive2://host:10000/db
  → match_profile_by_url() 遍历所有 profile
  → 按 jdbc_url_prefix 前缀匹配
  → 无匹配时返回 default_profile
  → 下载对应 maven_coordinate 的驱动
  → 用对应 driver_class 启动 JdbcBridge
```

### state.json 记录

```json
{
  "hive": {
    "version": "org.apache.hive:hive-jdbc:4.0.0",
    "installed_at": "2026-07-12T10:00:00+00:00",
    "jre": "jre21"
  }
}
```

注意：`version` 字段存储的是完整 Maven coordinate（`groupId:artifactId:version`），而非纯版本号。

### JdbcBridge 启动

```bash
java -cp dbx-jdbc-bridge.jar:drivers/hive/lib/* \
     com.dbx.agent.jdbcbridge.JdbcBridge \
     org.apache.hive.jdbc.HiveDriver \
     jdbc:hive2://{host}:{port}/{database}
```

第二个参数是 `jdbc_url_template`（含 `{host}` 占位符）。实际 JDBC URL 由 `ConfiguredJdbcAgent.buildJdbcUrl()` 在握手阶段根据连接参数拼装。

### 下载触发位置

`spawn_client_for_key()` → `ensure_jdbc_bridge_driver_installed()`：

```
检查 ~/.dbx/agents/drivers/{key}/lib/ 有无 JAR
  → 有 → 直接启动
  → 无 →
      获取 profile.maven_coordinate
      运行 DbxMavenResolver resolve --coordinate {coordinate}
      复制 JAR 到 lib/
      写入 state.json
      启动 JdbcBridge
```

## 已配置的 JDBC Bridge Profiles

| key | profile | maven_coordinate | driver_class |
|-----|---------|-----------------|--------------|
| h2 | h2 | com.h2database:h2:2.3.232 | org.h2.Driver |
| hive | hive2 (default) | org.apache.hive:hive-jdbc:4.0.0 | org.apache.hive.jdbc.HiveDriver |
| hive | hive1 | org.apache.hive:hive-jdbc:1.2.2 | org.apache.hadoop.hive.jdbc.HiveDriver |
| spark | spark | org.apache.spark:spark-hive-thriftserver_2.12:3.5.5 | org.apache.hive.jdbc.HiveDriver |
| trino | trino | io.trino:trino-jdbc:470 | io.trino.jdbc.TrinoDriver |
| db2 | db2 | com.ibm.db2:jcc:11.5.9.0 | com.ibm.db2.jcc.DB2Driver |
| informix | informix | com.ibm.informix:ifxjdbc:4.50.10.0 | com.informix.jdbc.IfxDriver |
| snowflake | snowflake | net.snowflake:snowflake-jdbc:3.22.1 | net.snowflake.client.jdbc.SnowflakeDriver |
| clickhouse | clickhouse | com.clickhouse:clickhouse-jdbc:0.7.2 | com.clickhouse.jdbc.ClickHouseDriver |
| redshift | redshift | com.amazon.redshift:redshift-jdbc42:2.1.0.12 | com.amazon.redshift.jdbc.Driver |
| bigquery | bigquery | com.google.cloud:google-cloud-bigquery-jdbc:2.48.0 | com.google.cloud.bigquery.jdbc.BigQueryDriver |
| vertica | vertica | com.vertica:vertica-jdbc:24.4.0 | com.vertica.jdbc.Driver |
| exasol | exasol | com.exasol:exasol-jdbc:7.1.30 | com.exasol.jdbc.EXADriver |
| saphana | saphana | com.sap.cloud.db.jdbc:ngdbc:2.25.13 | com.sap.db.jdbc.Driver |
| teradata | teradata | com.teradata:terajdbc:20.0.00.17 | com.teradata.jdbc.TeraDriver |
| firebird | firebird | org.firebirdsql.jdbc:jaybird:6.0.0 | org.firebirdsql.jdbc.FBDriver |
| neo4j | neo4j | org.neo4j:neo4j-jdbc-driver:6.0.0 | org.neo4j.jdbc.Driver |
| cassandra | cassandra | com.dbschema:CassandraJdbcDriver:2.2.0 | com.dbschema.CassandraJdbcDriver |
| kylin | kylin | org.apache.kylin:kylin-jdbc:5.0.0 | org.apache.kylin.jdbc.Driver |
| databricks | databricks | com.databricks:databricks-jdbc:2.6.39 | com.databricks.client.jdbc.Driver |

## 设计决策

### ADR-1：profile 级别而不是 entry 级别

同一数据库类型的不同版本需要不同的 Maven 坐标和 driver class。profile 级别支持从 `jdbc_url_prefix` 自动路由到具体版本。

### ADR-2：重用 ConfiguredJdbcAgent 而不是全新协议

现有 Agent JAR 已统一使用 JSON-RPC over stdin/stdout。`ConfiguredJdbcAgent` 封装了连接、元数据查询、查询执行等全部 JDBC 操作。重用意味着：
- 零协议变更，前端和后端无需修改
- 免去实现 `test_connection`、`query`、`disconnect` 等方法

### ADR-3：driver lib 不用 per-profile 子目录

当前 JAR 统一放在 `~/.dbx/agents/drivers/{key}/lib/` 下。简化实现，同一数据库类型同时连接多个 profile 版本的概率很低。若未来需要多版本共存，可升级为 `{key}/{profile}/lib/`。

### ADR-4：传递 jdbc_url_template 而非实际 URL

`JdbcBridge` 接收 `jdbc_url_template`（含 `{host}` 占位符）。实际 URL 由 `ConfiguredJdbcAgent.buildJdbcUrl()` 在握手阶段拼接。调用方不必提前拼完整 URL。

### ADR-5：Oracle 不纳入本 PIP

Oracle JDBC 驱动不在 Maven Central 公开仓库中，`maven_coordinate` 留空，不触发自动下载，除非后续确认可通过 Oracle Maven 仓库公开获取。

## 风险与缓解

| 风险 | 影响 | 缓解 | 优先 |
|------|------|------|------|
| Maven Central 国内访问慢 | 首次连接超时 | 支持 `--repo` 参数配置镜像（阿里云等） | P1 |
| JDBC 驱动版本兼容性 | 部分驱动类加载失败 | 每个 profile 验证通过后再合并 | P1 |
| JRE 未安装 | Maven 解析器无法运行 | 复用已有 Agent JRE 下载逻辑 | P1 |
| 传递依赖体积大 | 用户等待时间长 | 前端弹窗显示文件大小 + 进度 | P2 |
| 硬编码 Maven Central URL | 无法切换镜像 | `ensure_jdbc_bridge_driver_installed` 中 `--repo` 改为可配置 | P1 |

## 兼容性

- JSON-RPC 协议不变，`JdbcBridge` 继承 `ConfiguredJdbcAgent`。
- `state.json` 向后兼容，`InstalledDriver.version` 扩展为 Maven coordinate。
- 已安装旧版 Agent JAR 不受影响，仅当驱动缺失时触发自动下载。
- 原生驱动完全不受影响。

## 回滚策略

- 通用 JDBC 桥接 Agent 与旧版定制 Agent JAR 互斥。出问题时手动安装旧版 JAR 即可回退。
- `agent_catalog.rs` 的 JDBC bridge 改造通过 `maven_coordinate.is_empty()` 天然控制开/关。
- 前端改动可独立回滚。

## 术语表

| 术语 | 定义 |
|------|------|
| 原生驱动 | 编译进 Rust 二进制（MySQL、PostgreSQL、SQLite、Redis 等） |
| JDBC 桥接 Agent | 通用 Java Agent（`JdbcBridge.java`），通过动态 classpath 加载 JDBC 驱动 |
| 定制 Agent JAR | 旧方案，每种数据库一个独立 Agent JAR |
| JDBC bridge profile | `agent_catalog.rs` 中 `AgentDriverProfile` 配置 |
| DbxMavenResolver | Java 子进程，基于 Eclipse Aether 实现 Maven 坐标解析 |
| `version`（state.json） | `InstalledDriver.version`，实际存储 Maven coordinate |
