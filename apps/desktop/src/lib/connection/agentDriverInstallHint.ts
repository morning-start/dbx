import type { DatabaseType } from "@/types/database";
import { supportsDriverManagement } from "@/lib/database/databaseCapabilities";

export interface AgentDriverInstallState {
  db_type: string;
  installed: boolean;
  update_available?: boolean;
}

export function agentDriverInstallKey(dbType: DatabaseType | undefined, driverProfile?: string): string | undefined {
  if (dbType === "oracle") return "oracle";
  if (dbType === "mongodb") return "mongodb";
  if (dbType === "dameng") return "dameng";
  if (dbType === "mq") return driverProfile === "kafka" ? "kafka" : undefined;
  return driverProfile && driverProfile !== dbType ? driverProfile : dbType;
}

export function showAgentDriverInstallHint(dbType: DatabaseType | undefined, drivers: readonly AgentDriverInstallState[], driverProfile?: string): boolean {
  if (isJdbcBridgeType(dbType)) return false;
  if (!supportsDriverManagement(dbType)) return false;
  const driverKey = agentDriverInstallKey(dbType, driverProfile);
  if (!driverKey) return false;
  return drivers.find((driver) => driver.db_type === driverKey)?.installed !== true;
}
export function hasAgentDriverUpdate(dbType: DatabaseType | undefined, drivers: readonly AgentDriverInstallState[], driverProfile?: string): boolean {
  if (isJdbcBridgeType(dbType)) return false;
  if (!supportsDriverManagement(dbType)) return false;
  const driverKey = agentDriverInstallKey(dbType, driverProfile);
  return drivers.find((driver) => driver.db_type === driverKey)?.update_available === true;
}

export function appendAgentDriverUpdateHint(message: string, hint: string): string {
  if (!message.trim()) return hint;
  if (message.includes(hint)) return message;
  return `${message}\n\n${hint}`;
}

/// JDBC bridge types: their drivers are auto-downloaded from Maven Central by the
/// backend (JdbcBridge), so the old-style `installAgent` flow must not be triggered.
const JDBC_BRIDGE_DB_TYPES: Record<string, true> = {
  h2: true,
  hive: true,
  spark: true,
  trino: true,
  db2: true,
  informix: true,
  snowflake: true,
  clickhouse: true,
  redshift: true,
  bigquery: true,
  vertica: true,
  exasol: true,
  saphana: true,
  teradata: true,
  firebird: true,
  neo4j: true,
  cassandra: true,
  kylin: true,
  databricks: true,
};

export function isJdbcBridgeType(dbType: DatabaseType | undefined): boolean {
  return dbType !== undefined && JDBC_BRIDGE_DB_TYPES[dbType] === true;
}
