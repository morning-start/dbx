package com.dbx.agent.jdbcbridge;

import com.dbx.agent.ConfiguredJdbcAgent;
import com.dbx.agent.ConnectParams;
import com.dbx.agent.JdbcAgentProfile;
import com.dbx.agent.JsonRpcServer;

import java.util.Collections;

/**
 * A universal JDBC bridge agent that dynamically loads any JDBC driver from the
 * classpath and connects using the given driver class and JDBC URL.
 *
 * Usage:
 *   java -cp dbx-jdbc-bridge.jar:driver/lib/* JdbcBridge {driver_class} {jdbc_url}
 *
 * The driver JARs must be on the classpath. The driver class and JDBC URL are
 * passed as command-line arguments. All standard JDBC metadata operations are
 * handled by ConfiguredJdbcAgent / StandardJdbcMetadata.
 */
public final class JdbcBridge extends ConfiguredJdbcAgent {

    private final String driverClass;
    private final String jdbcUrl;

    public JdbcBridge(String driverClass, String jdbcUrl) {
        super(new JdbcAgentProfile(
            driverClass,
            "",       // urlTemplate — not used when connection_string is set
            0,        // defaultPort
            false,    // skipExecutionContext
            Collections.emptySet(),  // excludedSchemas
            Collections.emptyList(), // tableTypes — use all types
            "\"",     // identifierQuote
            "SET SCHEMA",
            true,     // catalogFallbackEnabled
            false,    // nativeTableDdlSupported
            false,    // objectSourceSupported
            false     // triggersSupported
        ));
        this.driverClass = driverClass;
        this.jdbcUrl = jdbcUrl;
    }

    @Override
    protected String driverClass() {
        return driverClass;
    }

    @Override
    protected String buildJdbcUrl(ConnectParams params) {
        // Prefer the URL passed as a command-line argument.
        if (jdbcUrl != null && !jdbcUrl.isEmpty()) {
            return jdbcUrl;
        }
        // Fall back to the connection_string from ConnectParams.
        String connStr = params.getConnection_string();
        if (connStr != null && !connStr.isEmpty()) {
            return connStr;
        }
        // Last resort: use the profile's URL template (unlikely to work for generic bridges).
        return super.buildJdbcUrl(params);
    }

    public static void main(String[] args) {
        if (args.length < 2) {
            System.err.println("Usage: JdbcBridge <driver_class> <jdbc_url>");
            System.exit(1);
        }
        String driverClass = args[0];
        String jdbcUrl = args[1];
        new JsonRpcServer(new JdbcBridge(driverClass, jdbcUrl)).run();
    }
}