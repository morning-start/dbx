# PIP-0002 实施追踪

跟踪 JDBC 驱动自动下载的实施进展、已知问题、前后端差距。本文档随代码更新，不是 static 设计决策。

## 实施状态总览

| 组件 | 文件 | 状态 | 备注 |
|------|------|------|------|
| JdbcBridge.java | `agents/drivers/jdbc-bridge/` | ✅ 已实现 | 继承 ConfiguredJdbcAgent，动态 classpath |
| AgentDriverProfile | `crates/dbx-core/src/agent_catalog.rs` | ✅ 已实现 | 含 maven_coordinate / driver_class / jdbc_url_prefix |
| match_profile_by_url() | `crates/dbx-core/src/agent_catalog.rs` | ✅ 已实现 | 按 URL 前缀匹配 |
| is_jdbc_bridge_type() | `crates/dbx-core/src/agent_catalog.rs` | ✅ 已实现 | 判断 DB type 是否使用 JDBC bridge |
| ensure_jdbc_bridge_driver_installed() | `crates/dbx-core/src/agent_runtime.rs` | ✅ 已实现 | 自动 Maven 下载 + 写入 state.json |
| resolve_jdbc_bridge_launch_spec() | `crates/dbx-core/src/agent_runtime.rs` | ✅ 已实现 | 构建 classpath + 启动参数 |
| spawn_client_for_key() JDBC 分支 | `crates/dbx-core/src/agent_runtime.rs` | ✅ 已实现 | 自动下载 → 启动 |
| JDBC bridge profiles (H2/Hive/Spark 等 20 种) | `crates/dbx-core/src/agent_catalog.rs` | ✅ 已实现 | 见 PIP-0002 配置表 |
| 驱动管理界面 | `apps/desktop/src/components/config/DriverStoreDialog.vue` | ✅ 已实现 | Agent / JDBC / Storage 三 Tab |
| 下载进度弹窗 | `apps/desktop/src/components/connection/ConnectionDialog.vue` | ✅ 已实现 | showAgentInstallDialog |
| 数据库类型列表均等展示 | `apps/desktop/src/components/connection/ConnectionDialog.vue` | ✅ 已实现 | dbOptions 无 installed 过滤 |
| Maven 下载进度事件 | `crates/dbx-core/src/agent_runtime.rs` | ❌ 缺失 | ensure_jdbc_bridge_driver_installed 无事件发射 |
| 前端避免双重安装 | `apps/desktop/src/components/connection/ConnectionDialog.vue` | ❌ 缺失 | 需要跳过 JDBC bridge 类型的旧 Agent 安装 |
| isJdbcBridgeType 前端判断 | 前端 | ❌ 缺失 | 需要从 Tauri 命令暴露 |
| Hive 1.x/2.x 前端 profile 选择 | `apps/desktop/src/components/connection/ConnectionDialog.vue` | ❌ 缺失 | dbOptions 只有一条 "Hive" |
| driver_store_entries() 暴露给前端 | `src-tauri/src/commands/agents.rs` | ❌ 缺失 | AgentDriverInfo 缺少 profiles/is_jdbc_bridge |
| Maven 镜像配置 | `crates/dbx-core/src/agent_runtime.rs` | ❌ 缺失 | `--repo` 硬编码 |
| ClickHouse manifest 同步 | `crates/dbx-core/assets/database-drivers.manifest.json` | ❌ 不一致 | runtimeMode native，实际走 JDBC bridge |

## 数据流分析

### 期望路径（最终目标）

```
用户点击"测试连接"
  │
  ├─ Frontend: testConnection()
  │   ├─ [JDBC bridge 类型] → 跳过旧 Agent 安装
  │   └─ api.testConnection(config)
  │
  └─ Backend: test_connection()
      └─ match db_type:
          ├─ agent type (is_agent_type) →
          │   test_agent_connection()
          │   → call_daemon_method_with_timeout()
          │   → spawn_client_for_key()
          │       │
          │       ├─ [JDBC bridge] →
          │       │   ├─ is_jdbc_bridge_driver_installed()?
          │       │   │   └─ 否 → ensure_jdbc_bridge_driver_installed()
          │       │   │       ├─ emit("agent-install-progress", step:"maven-resolve")
          │       │   │       ├─ emit(transfer("maven-download", downloaded, total))
          │       │   │       ├─ DbxMavenResolver 下载 JAR
          │       │   │       └─ emit(step("done"))
          │       │   └─ JdbcBridge 启动
          │       │
          │       └─ [传统 Agent] →
          │           └─ 直接启动定制 Agent JAR
```

### 当前路径（有缺陷）

```
用户点击"测试连接"
  │
  ├─ Frontend: testConnection()
  │   ├─ ensureRequiredAgentDriverInstalled("hive")    ← 第一步
  │   │   └─ api.installAgent("hive")                  ← 下载旧版定制 Agent JAR
  │   │       ├─ emit("agent-install-progress")        ← 有进度
  │   │       └─ 下载完成
  │   └─ api.testConnection(config)
  │
  └─ Backend: test_connection()
      └─ spawn_client_for_key("hive")                  ← 第二步
          ├─ is_jdbc_bridge_driver_installed? → false
          └─ ensure_jdbc_bridge_driver_installed()     ← 再次 Maven 下载
              └─ ❌ 无进度事件发射
```

## 已识别问题

### P0：双重安装路径

**问题**：前端 `ensureRequiredAgentDriverInstalled` 对 JDBC bridge 类型也调用 `api.installAgent(driverKey)`（旧版定制 Agent JAR）。后端 `spawn_client_for_key` 发现驱动未安装时，通过 `ensure_jdbc_bridge_driver_installed` 再次 Maven 下载。用户经历两轮下载。

**根因**：前端 `agentDriverInstallKey()` 对所有 `driverManagement == true` 的类型返回 key，没有区分 JDBC bridge。

**修复**：
1. 前端新增 `isJdbcBridgeType()` 函数，判断依据：
   - 后端暴露 `agent_catalog::is_jdbc_bridge_type()` 到前端
   - 或前端维护一份 JDBC bridge 类型白名单
2. `ensureRequiredAgentDriverInstalled()` 中跳过 JDBC bridge 类型
3. 让后端 `spawn_client_for_key()` 的 Maven 下载透明处理

### P0：Maven 下载缺少进度事件

**问题**：`ensure_jdbc_bridge_driver_installed()` 全程没有发射 `AgentProgressEvent`。前端进度弹窗无法感知下载状态。

**涉及代码**：`crates/dbx-core/src/agent_runtime.rs:183-270`

**修复**：异步运行 `DbxMavenResolver` 子进程，在下载前/中/后发射进度事件：
- `step:"maven-resolve"` — 正在解析坐标
- `transfer("maven-download", downloaded, total)` — 下载进度
- `step("done")` — 完成
- `step("error", message)` — 失败

### P1：Hive 1.x/2.x profile 前端无选择入口

**问题**：后端有两个 profiles（`hive2` + `hive1`），有 `match_profile_by_url()` 自动匹配。但前端 `driverProfiles` 和 `dbOptions` 里 Hive 只有一条。

**影响**：用户如果填错 URL 前缀（如 Hive 1.x 用 `jdbc:hive2://`），连接会失败，但没有 UI 提示切换 profile。新建连接时也无法主动选择 Hive 1.x。

**修复**：
- 后端：新增 Tauri 命令 `list_driver_profiles(db_type)` 返回 profile 列表
- 前端：Hive 连接表单增加版本选择器（与 OceanBase/GBase 类似）

### P1：Maven 镜像配置不可配置

**问题**：`ensure_jdbc_bridge_driver_installed()` 中 `--repo` 硬编码为 `https://repo.maven.apache.org/maven2/`。

**修复**：
- 从全局设置 `maven_mirror_url` 读取
- 或在 `agent_catalog` 的 profile 中允许 override
- 未配置时默认使用当前硬编码值

### P1：`AgentDriverInfo` 缺少 JDBC bridge 字段

**问题**：`build_agent_list()` 返回的 `AgentDriverInfo` 没有 `is_jdbc_bridge`、`profiles`、`maven_coordinate` 等字段。前端驱动管理界面无法显示哪些类型是 auto-download 的。

**影响**：前端无法差异化显示 JDBC bridge 类型（比如显示 "Maven Central" 来源标签）。

**修复**：在 `AgentDriverInfo` 中新增可选字段 `is_jdbc_bridge: bool`。

### P2：ClickHouse manifest 与后端 catalog 不一致

**问题**：`database-drivers.manifest.json` 中 ClickHouse `runtimeMode: "native"`, `driverManagement: false`，但 `agent_catalog.rs` 中 ClickHouse 有 JDBC bridge profile。

**影响**：前端 `supportsDriverManagement("clickhouse")` → `false`，不会在连接对话框中触发任何驱动安装提示。

**修复**：同步 manifest 中 ClickHouse 的 `runtimeMode` 为 `"agent"`、`driverManagement: true`，或确认实际是否应使用 JDBC bridge。

## 修复优先级

| 优先 | 问题 | 文件 | 估量 |
|------|------|------|------|
| ⚠️ P0 | 双重安装路径 | ConnectionDialog.vue + api.ts | 小 |
| ⚠️ P0 | Maven 下载无进度事件 | agent_runtime.rs | 中 |
| 🔶 P1 | Hive 前端 profile 选择 | ConnectionDialog.vue + Tauri 命令 | 中 |
| 🔶 P1 | Maven 镜像配置硬编码 | agent_runtime.rs | 小 |
| 🔶 P1 | AgentDriverInfo 缺字段 | agent_manager.rs + Tauri 命令 | 小 |
| 🔷 P2 | ClickHouse manifest 同步 | database-drivers.manifest.json | 小 |

## 已完成的验收测试

后端单元测试已覆盖：

| 测试 | 路径 | 覆盖内容 |
|------|------|----------|
| `match_profile_by_url_works` | agent_runtime.rs | `jdbc:hive2://` → hive2, `jdbc:hive://` → hive1, `jdbc:trino://` → trino |
| `jdbc_bridge_types_are_detected` | agent_runtime.rs | H2/Hive/Trino/Spark/Db2 是 bridge 类型；Dameng/Kingbase 不是 |
| `prestosql_does_not_use_agent_driver` | agent_runtime.rs | PrestoSQL 不走 agent driver |
| `trino_uses_only_trino_agent_driver` | agent_runtime.rs | Trino 只走 trino key |
| `is_jdbc_bridge_type`（implied） | agent_catalog.rs | 所有含 maven_coordinate 的 profile |
