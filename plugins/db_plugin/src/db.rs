// L-0 Database Plugin v2.0
// SQLite backend (default), with connection string support for other databases
// Build: cd tools/db && cargo build --release
// Binary: tools/db/target/release/db_plugin

use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::sync::Mutex;

// Config file to persist DB path across process invocations
const DB_CONFIG_PATH: &str = ".l0_db_config.json";

// Global connection (SQLite is not thread-safe by default)
static DB: Mutex<Option<Connection>> = Mutex::new(None);
static DB_PATH: Mutex<Option<String>> = Mutex::new(None);

/// Load DB path from config and auto-connect
fn ensure_connection() -> bool {
    let db = DB.lock().unwrap();
    if db.is_some() {
        return true;
    }
    drop(db); // Release lock before reading file

    // Try to load from config file
    if let Ok(content) = fs::read_to_string(DB_CONFIG_PATH) {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(path) = config["path"].as_str() {
                if let Ok(conn) = Connection::open(path) {
                    let _ = conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;");
                    let mut db = DB.lock().unwrap();
                    *db = Some(conn);
                    drop(db);
                    let mut db_path = DB_PATH.lock().unwrap();
                    *db_path = Some(path.to_string());
                    return true;
                }
            }
        }
    }
    false
}

/// Save DB path to config file
fn save_db_config(path: &str) {
    let config = json!({"path": path});
    let _ = fs::write(DB_CONFIG_PATH, config.to_string());
}

#[derive(Deserialize)]
struct Request {
    method: String,
    args: Vec<String>,
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    for line in stdin.lock().lines() {
        if let Ok(request_str) = line {
            let response = handle_request(&request_str);
            writeln!(stdout_lock, "{}", response).unwrap();
            stdout_lock.flush().unwrap();
        }
    }
}

fn handle_request(request_str: &str) -> String {
    // Parse request JSON
    let request: Request = match serde_json::from_str(request_str) {
        Ok(r) => r,
        Err(e) => return json!({"ok": false, "error": format!("Invalid JSON: {}", e)}).to_string(),
    };

    let args = &request.args;

    match request.method.as_str() {
        "init" => cmd_init(args),
        "connect" => cmd_connect(args),
        "create_table" => cmd_create_table(args),
        "insert" => cmd_insert(args),
        "select" => cmd_select(args),
        "update" => cmd_update(args),
        "delete" => cmd_delete(args),
        "drop_table" => cmd_drop_table(args),
        "list_tables" => cmd_list_tables(args),
        "exec" => cmd_exec(args),
        "query" => cmd_query(args),
        _ => json!({"ok": false, "error": format!("Unknown method: {}", request.method)}).to_string(),
    }
}

// === Database Commands ===

/// Initialize/connect to SQLite database
/// Args: [path] (default: "l0.db")
fn cmd_init(args: &[String]) -> String {
    let path = args.get(0).map(|s| s.as_str()).unwrap_or("l0.db");

    match Connection::open(path) {
        Ok(conn) => {
            // Enable WAL mode for better concurrency
            let _ = conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;");

            let mut db = DB.lock().unwrap();
            *db = Some(conn);

            let mut db_path = DB_PATH.lock().unwrap();
            *db_path = Some(path.to_string());

            // Persist config for future process invocations
            save_db_config(path);

            json!({"ok": true, "value": format!("SQLite connected: {}", path)}).to_string()
        }
        Err(e) => json!({"ok": false, "error": format!("Failed to open database: {}", e)}).to_string(),
    }
}

/// Connect with connection string (for future MySQL/PostgreSQL support)
/// Args: [connection_string]
/// Formats:
///   sqlite:path/to/db.db
///   mysql://user:pass@host:port/dbname (future)
///   postgres://user:pass@host:port/dbname (future)
fn cmd_connect(args: &[String]) -> String {
    let conn_str = match args.get(0) {
        Some(s) => s,
        None => return json!({"ok": false, "error": "Usage: connect <connection_string>"}).to_string(),
    };

    if conn_str.starts_with("sqlite:") {
        let path = &conn_str[7..];
        return cmd_init(&[path.to_string()]);
    }

    if conn_str.starts_with("mysql://") {
        return json!({"ok": false, "error": "MySQL support requires mysql_plugin (not bundled). Use sqlite: for embedded database."}).to_string();
    }

    if conn_str.starts_with("postgres://") || conn_str.starts_with("postgresql://") {
        return json!({"ok": false, "error": "PostgreSQL support requires pg_plugin (not bundled). Use sqlite: for embedded database."}).to_string();
    }

    // Default: treat as SQLite path
    cmd_init(&[conn_str.clone()])
}

/// Create table
/// Args: [table_name] or [table_name|column_definitions]
/// Column definitions: "id INTEGER PRIMARY KEY, name TEXT, age INTEGER"
fn cmd_create_table(args: &[String]) -> String {
    ensure_connection();
    let db = DB.lock().unwrap();
    let conn = match db.as_ref() {
        Some(c) => c,
        None => return json!({"ok": false, "error": "Database not initialized. Call init first."}).to_string(),
    };

    let arg = match args.get(0) {
        Some(s) => s,
        None => return json!({"ok": false, "error": "Usage: create_table table_name[|columns]"}).to_string(),
    };

    let (table_name, columns) = if let Some(idx) = arg.find('|') {
        (&arg[..idx], &arg[idx+1..])
    } else {
        (arg.as_str(), "id INTEGER PRIMARY KEY AUTOINCREMENT, data TEXT")
    };

    let sql = format!("CREATE TABLE IF NOT EXISTS {} ({})", table_name, columns);

    match conn.execute(&sql, []) {
        Ok(_) => json!({"ok": true, "value": format!("Table '{}' created", table_name)}).to_string(),
        Err(e) => json!({"ok": false, "error": format!("CREATE TABLE failed: {}", e)}).to_string(),
    }
}

/// Insert data
/// Args: [table|json_data] or [table|col1,col2|val1,val2]
fn cmd_insert(args: &[String]) -> String {
    ensure_connection();
    let db = DB.lock().unwrap();
    let conn = match db.as_ref() {
        Some(c) => c,
        None => return json!({
            "ok": false,
            "error": "Database not initialized. Call DB_INIT (0x9000) first with database path.",
            "hint": "SETS 255, \"app.db\"\nTEXEC 0x9000, 255, 0"
        }).to_string(),
    };

    let arg = match args.get(0) {
        Some(s) => s,
        None => return json!({
            "ok": false,
            "error": "Missing arguments for INSERT",
            "usage": "table|{json} or table|cols|vals",
            "examples": ["articles|{\"title\":\"Hello\"}", "articles|title,content|Hello,World"]
        }).to_string(),
    };

    let parts: Vec<&str> = arg.splitn(3, '|').collect();

    if parts.len() < 2 {
        return json!({
            "ok": false,
            "error": "Invalid INSERT format",
            "got": arg,
            "usage": "table|{json} or table|cols|vals",
            "examples": ["articles|{\"title\":\"Hello\"}", "articles|title,content|Hello,World"]
        }).to_string();
    }

    let table = parts[0];

    // Check if it's JSON format
    if parts[1].starts_with('{') {
        // Parse JSON and insert
        let json_str = if parts.len() == 2 {
            parts[1]
        } else {
            // Rejoin if pipe was in JSON
            &arg[table.len()+1..]
        };

        match serde_json::from_str::<HashMap<String, Value>>(json_str) {
            Ok(data) => {
                let columns: Vec<&str> = data.keys().map(|k| k.as_str()).collect();
                let placeholders: Vec<&str> = (0..columns.len()).map(|_| "?").collect();

                let sql = format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    table,
                    columns.join(", "),
                    placeholders.join(", ")
                );

                let values: Vec<String> = columns.iter()
                    .map(|&col| value_to_sql_string(data.get(col).unwrap()))
                    .collect();

                let params: Vec<&dyn rusqlite::ToSql> = values.iter()
                    .map(|v| v as &dyn rusqlite::ToSql)
                    .collect();

                match conn.execute(&sql, params.as_slice()) {
                    Ok(_) => {
                        let last_id = conn.last_insert_rowid();
                        json!({"ok": true, "value": last_id}).to_string()
                    }
                    Err(e) => json!({"ok": false, "error": format!("INSERT failed: {}", e)}).to_string(),
                }
            }
            Err(e) => json!({"ok": false, "error": format!("Invalid JSON: {}", e)}).to_string(),
        }
    } else if parts.len() >= 3 {
        // Format: table|col1,col2|val1,val2
        let columns = parts[1];
        let values_str = parts[2];

        let cols: Vec<&str> = columns.split(',').map(|s| s.trim()).collect();
        let vals: Vec<&str> = values_str.split(',').map(|s| s.trim()).collect();

        if cols.len() != vals.len() {
            return json!({"ok": false, "error": "Column and value count mismatch"}).to_string();
        }

        let placeholders: Vec<&str> = (0..cols.len()).map(|_| "?").collect();
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            cols.join(", "),
            placeholders.join(", ")
        );

        let params: Vec<&dyn rusqlite::ToSql> = vals.iter()
            .map(|v| v as &dyn rusqlite::ToSql)
            .collect();

        match conn.execute(&sql, params.as_slice()) {
            Ok(_) => {
                let last_id = conn.last_insert_rowid();
                json!({"ok": true, "value": last_id}).to_string()
            }
            Err(e) => json!({"ok": false, "error": format!("INSERT failed: {}", e)}).to_string(),
        }
    } else {
        json!({"ok": false, "error": "Usage: insert table|{json} or table|cols|vals"}).to_string()
    }
}

/// Select data
/// Args: [table] or [table|condition] or [table|condition|columns]
fn cmd_select(args: &[String]) -> String {
    ensure_connection();
    let db = DB.lock().unwrap();
    let conn = match db.as_ref() {
        Some(c) => c,
        None => return json!({
            "ok": false,
            "error": "Database not initialized. Call DB_INIT (0x9000) first.",
            "hint": "SETS 255, \"app.db\"\nTEXEC 0x9000, 255, 0"
        }).to_string(),
    };

    let arg = match args.get(0) {
        Some(s) => s,
        None => return json!({
            "ok": false,
            "error": "Missing table name for SELECT",
            "usage": "table or table|condition or table|condition|columns",
            "examples": ["articles", "articles|id>5", "articles|status=active|id,title"]
        }).to_string(),
    };

    let parts: Vec<&str> = arg.splitn(3, '|').collect();
    let table = parts[0];
    let condition = parts.get(1).map(|s| *s).unwrap_or("1=1");
    let columns = parts.get(2).map(|s| *s).unwrap_or("*");

    // Build SQL
    let sql = if condition == "*" || condition.is_empty() {
        format!("SELECT {} FROM {}", columns, table)
    } else {
        format!("SELECT {} FROM {} WHERE {}", columns, table, condition)
    };

    match query_to_json(conn, &sql) {
        Ok(results) => json!({"ok": true, "value": results}).to_string(),
        Err(e) => json!({
            "ok": false,
            "error": format!("SELECT failed: {}", e),
            "sql": sql,
            "hint": "Check table name exists and condition syntax is valid SQL"
        }).to_string(),
    }
}

/// Update data
/// Args: [table|condition|json_data] or [table|condition|col=val,col2=val2]
fn cmd_update(args: &[String]) -> String {
    ensure_connection();
    let db = DB.lock().unwrap();
    let conn = match db.as_ref() {
        Some(c) => c,
        None => return json!({"ok": false, "error": "Database not initialized"}).to_string(),
    };

    let arg = match args.get(0) {
        Some(s) => s,
        None => return json!({"ok": false, "error": "Usage: update table|condition|{json} or table|condition|col=val"}).to_string(),
    };

    let parts: Vec<&str> = arg.splitn(3, '|').collect();

    if parts.len() < 3 {
        return json!({"ok": false, "error": "Usage: update table|condition|data"}).to_string();
    }

    let table = parts[0];
    let condition = parts[1];
    let data_str = parts[2];

    // Build SET clause
    let set_clause = if data_str.starts_with('{') {
        // JSON format
        match serde_json::from_str::<HashMap<String, Value>>(data_str) {
            Ok(data) => {
                data.iter()
                    .map(|(k, v)| format!("{} = '{}'", k, value_to_sql_string(v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
            Err(e) => return json!({"ok": false, "error": format!("Invalid JSON: {}", e)}).to_string(),
        }
    } else {
        // col=val format - properly quote values
        data_str.split(',')
            .filter_map(|pair| {
                let parts: Vec<&str> = pair.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let col = parts[0].trim();
                    let val = parts[1].trim().replace("'", "''"); // Escape single quotes
                    Some(format!("{} = '{}'", col, val))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    };

    let sql = format!("UPDATE {} SET {} WHERE {}", table, set_clause, condition);

    match conn.execute(&sql, []) {
        Ok(rows) => json!({"ok": true, "value": format!("{} rows updated", rows)}).to_string(),
        Err(e) => json!({"ok": false, "error": format!("UPDATE failed: {}", e)}).to_string(),
    }
}

/// Delete data
/// Args: [table|condition]
fn cmd_delete(args: &[String]) -> String {
    ensure_connection();
    let db = DB.lock().unwrap();
    let conn = match db.as_ref() {
        Some(c) => c,
        None => return json!({
            "ok": false,
            "error": "Database not initialized. Call DB_INIT (0x9000) first.",
            "hint": "SETS 255, \"app.db\"\nTEXEC 0x9000, 255, 0"
        }).to_string(),
    };

    let arg = match args.get(0) {
        Some(s) => s,
        None => return json!({
            "ok": false,
            "error": "Missing arguments for DELETE",
            "usage": "table|condition",
            "examples": ["articles|id=5", "users|status=inactive"]
        }).to_string(),
    };

    let parts: Vec<&str> = arg.splitn(2, '|').collect();

    if parts.len() < 2 {
        return json!({
            "ok": false,
            "error": "Invalid DELETE format - missing condition",
            "got": arg,
            "usage": "table|condition",
            "examples": ["articles|id=5", "users|created_at<'2024-01-01'"]
        }).to_string();
    }

    let table = parts[0];
    let condition = parts[1];

    let sql = format!("DELETE FROM {} WHERE {}", table, condition);

    match conn.execute(&sql, []) {
        Ok(rows) => json!({"ok": true, "value": format!("{} rows deleted", rows)}).to_string(),
        Err(e) => json!({"ok": false, "error": format!("DELETE failed: {}", e)}).to_string(),
    }
}

/// Drop table
/// Args: [table_name]
fn cmd_drop_table(args: &[String]) -> String {
    ensure_connection();
    let db = DB.lock().unwrap();
    let conn = match db.as_ref() {
        Some(c) => c,
        None => return json!({"ok": false, "error": "Database not initialized"}).to_string(),
    };

    let table = match args.get(0) {
        Some(s) => s,
        None => return json!({"ok": false, "error": "Usage: drop_table table_name"}).to_string(),
    };

    let sql = format!("DROP TABLE IF EXISTS {}", table);

    match conn.execute(&sql, []) {
        Ok(_) => json!({"ok": true, "value": format!("Table '{}' dropped", table)}).to_string(),
        Err(e) => json!({"ok": false, "error": format!("DROP TABLE failed: {}", e)}).to_string(),
    }
}

/// List all tables
fn cmd_list_tables(_args: &[String]) -> String {
    ensure_connection();
    let db = DB.lock().unwrap();
    let conn = match db.as_ref() {
        Some(c) => c,
        None => return json!({"ok": false, "error": "Database not initialized"}).to_string(),
    };

    let sql = "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'";

    let result = match conn.prepare(sql) {
        Ok(mut stmt) => {
            let tables: Vec<String> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            json!({"ok": true, "value": tables}).to_string()
        }
        Err(e) => json!({"ok": false, "error": format!("Failed to list tables: {}", e)}).to_string(),
    };
    result
}

/// Execute raw SQL (for DDL/DML)
/// Args: [sql_statement]
fn cmd_exec(args: &[String]) -> String {
    ensure_connection();
    let db = DB.lock().unwrap();
    let conn = match db.as_ref() {
        Some(c) => c,
        None => return json!({"ok": false, "error": "Database not initialized"}).to_string(),
    };

    let sql = match args.get(0) {
        Some(s) => s,
        None => return json!({"ok": false, "error": "Usage: exec <sql_statement>"}).to_string(),
    };

    match conn.execute(sql, []) {
        Ok(rows) => json!({"ok": true, "value": format!("{} rows affected", rows)}).to_string(),
        Err(e) => json!({"ok": false, "error": format!("EXEC failed: {}", e)}).to_string(),
    }
}

/// Execute raw SQL query (for SELECT)
/// Args: [sql_query]
fn cmd_query(args: &[String]) -> String {
    ensure_connection();
    let db = DB.lock().unwrap();
    let conn = match db.as_ref() {
        Some(c) => c,
        None => return json!({"ok": false, "error": "Database not initialized"}).to_string(),
    };

    let sql = match args.get(0) {
        Some(s) => s,
        None => return json!({"ok": false, "error": "Usage: query <sql_query>"}).to_string(),
    };

    match query_to_json(conn, sql) {
        Ok(results) => json!({"ok": true, "value": results}).to_string(),
        Err(e) => json!({"ok": false, "error": format!("QUERY failed: {}", e)}).to_string(),
    }
}

// === Helper Functions ===

fn value_to_sql_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
        Value::Null => "NULL".to_string(),
        _ => v.to_string(),
    }
}

fn query_to_json(conn: &Connection, sql: &str) -> Result<Vec<HashMap<String, Value>>, String> {
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

    let column_names: Vec<String> = stmt
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    let rows = stmt
        .query_map([], |row| {
            let mut map = HashMap::new();
            for (i, col_name) in column_names.iter().enumerate() {
                let value: Value = match row.get_ref(i) {
                    Ok(rusqlite::types::ValueRef::Null) => Value::Null,
                    Ok(rusqlite::types::ValueRef::Integer(n)) => json!(n),
                    Ok(rusqlite::types::ValueRef::Real(f)) => json!(f),
                    Ok(rusqlite::types::ValueRef::Text(s)) => {
                        Value::String(String::from_utf8_lossy(s).to_string())
                    }
                    Ok(rusqlite::types::ValueRef::Blob(b)) => {
                        Value::String(format!("<blob:{} bytes>", b.len()))
                    }
                    Err(_) => Value::Null,
                };
                map.insert(col_name.clone(), value);
            }
            Ok(map)
        })
        .map_err(|e| e.to_string())?;

    let results: Vec<HashMap<String, Value>> = rows
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}
