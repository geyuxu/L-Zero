// L-0 Database Plugin
// 简单的 JSON 文件数据库：init, create_table, insert, select, update, delete
// 编译: rustc -O db_plugin.rs -o db_plugin

use std::fs;
use std::io::{self, BufRead};
use std::collections::HashMap;

// 数据库结构: { "tables": { "table_name": [ {row1}, {row2}, ... ] } }
static mut DB_PATH: Option<String> = None;

fn get_db_path() -> String {
    unsafe {
        DB_PATH.clone().unwrap_or_else(|| "l0_database.json".to_string())
    }
}

fn set_db_path(path: &str) {
    unsafe {
        DB_PATH = Some(path.to_string());
    }
}

fn main() {
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        if let Ok(request) = line {
            let response = handle_request(&request);
            println!("{}", response);
        }
    }
}

fn handle_request(request: &str) -> String {
    let method = extract_string(request, "method");
    let args = extract_args(request);

    match method.as_str() {
        "init" => {
            // 初始化数据库
            // args[0] = 数据库文件路径
            let path = args.get(0).map(|s| s.as_str()).unwrap_or("l0_database.json");
            set_db_path(path);

            // 如果文件不存在，创建空数据库
            if !std::path::Path::new(path).exists() {
                let empty_db = r#"{"tables":{}}"#;
                if let Err(e) = fs::write(path, empty_db) {
                    return format!(r#"{{"ok":false,"error":"{}"}}"#, e);
                }
            }

            format!(r#"{{"ok":true,"value":"initialized: {}"}}"#, path)
        }

        "create_table" => {
            // 创建表
            // args[0] = 表名, args[1] = 列定义 (可选，如 "id,name,age")
            if let Some(table_name) = args.get(0) {
                let mut db = load_db();

                if !db.contains_key(table_name) {
                    db.insert(table_name.clone(), Vec::new());
                    if let Err(e) = save_db(&db) {
                        return format!(r#"{{"ok":false,"error":"{}"}}"#, e);
                    }
                    format!(r#"{{"ok":true,"value":"table {} created"}}"#, table_name)
                } else {
                    format!(r#"{{"ok":true,"value":"table {} exists"}}"#, table_name)
                }
            } else {
                r#"{"ok":false,"error":"usage: create_table table_name"}"#.to_string()
            }
        }

        "insert" => {
            // 插入数据
            // 格式: "table_name|{json}" 或 args[0]=table, args[1]=json
            let (table_name, row_data) = if args.len() >= 2 {
                (args[0].clone(), args[1].clone())
            } else if let Some(arg) = args.get(0) {
                if let Some(idx) = arg.find('|') {
                    (arg[..idx].to_string(), arg[idx+1..].to_string())
                } else {
                    return r#"{"ok":false,"error":"usage: insert table_name|{json}"}"#.to_string();
                }
            } else {
                return r#"{"ok":false,"error":"usage: insert table_name|{json}"}"#.to_string();
            };

            let mut db = load_db();

            let exists = db.contains_key(&table_name);
            if exists {
                db.get_mut(&table_name).unwrap().push(row_data.clone());
                let row_id = db.get(&table_name).unwrap().len() - 1;
                if let Err(e) = save_db(&db) {
                    return format!(r#"{{"ok":false,"error":"{}"}}"#, e);
                }
                format!(r#"{{"ok":true,"value":"inserted at row {}"}}"#, row_id)
            } else {
                format!(r#"{{"ok":false,"error":"table {} not found"}}"#, table_name)
            }
        }

        "select" => {
            // 查询数据
            // 格式: "table_name" 或 "table_name|condition"
            let (table_name, condition) = if args.len() >= 2 {
                (args[0].clone(), args[1].clone())
            } else if let Some(arg) = args.get(0) {
                if let Some(idx) = arg.find('|') {
                    (arg[..idx].to_string(), arg[idx+1..].to_string())
                } else {
                    (arg.clone(), "*".to_string())
                }
            } else {
                return r#"{"ok":false,"error":"usage: select table_name[|condition]"}"#.to_string();
            };
            let db = load_db();

            if let Some(table) = db.get(&table_name) {
                let results: Vec<&String> = if condition == "*" {
                    table.iter().collect()
                } else {
                    // 简单条件解析: "field=value"
                    let parts: Vec<&str> = condition.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let field = parts[0];
                        let value = parts[1];
                        table.iter().filter(|row| {
                            row.contains(&format!("\"{}\":", field)) &&
                            (row.contains(&format!("\"{}\":\"{}\"", field, value)) ||
                             row.contains(&format!("\"{}\":{}", field, value)))
                        }).collect()
                    } else {
                        table.iter().collect()
                    }
                };

                let json_array: Vec<String> = results.iter()
                    .map(|r| (*r).clone())
                    .collect();

                format!(r#"{{"ok":true,"value":[{}]}}"#, json_array.join(","))
            } else {
                format!(r#"{{"ok":false,"error":"table {} not found"}}"#, table_name)
            }
        }

        "update" => {
            // 更新数据
            // 格式: "table_name|condition|{new_json}"
            let parts: Vec<&str> = if let Some(arg) = args.get(0) {
                arg.splitn(3, '|').collect()
            } else {
                return r#"{"ok":false,"error":"usage: update table|condition|{json}"}"#.to_string();
            };

            if parts.len() < 3 {
                return r#"{"ok":false,"error":"usage: update table|condition|{json}"}"#.to_string();
            }

            let table_name = parts[0];
            let condition = parts[1];
            let new_data = parts[2];

            let mut db = load_db();

            if let Some(table) = db.get_mut(table_name) {
                let cond_parts: Vec<&str> = condition.splitn(2, '=').collect();
                let mut updated = 0;

                if cond_parts.len() == 2 {
                    let field = cond_parts[0];
                    let value = cond_parts[1];

                    for row in table.iter_mut() {
                        if row.contains(&format!("\"{}\":", field)) &&
                           (row.contains(&format!("\"{}\":\"{}\"", field, value)) ||
                            row.contains(&format!("\"{}\":{}", field, value))) {
                            *row = new_data.to_string();
                            updated += 1;
                        }
                    }
                }

                if let Err(e) = save_db(&db) {
                    return format!(r#"{{"ok":false,"error":"{}"}}"#, e);
                }
                format!(r#"{{"ok":true,"value":"updated {} rows"}}"#, updated)
            } else {
                format!(r#"{{"ok":false,"error":"table {} not found"}}"#, table_name)
            }
        }

        "delete" => {
            // 删除数据
            // 格式: "table_name|condition"
            let (table_name, condition) = if args.len() >= 2 {
                (args[0].clone(), args[1].clone())
            } else if let Some(arg) = args.get(0) {
                if let Some(idx) = arg.find('|') {
                    (arg[..idx].to_string(), arg[idx+1..].to_string())
                } else {
                    return r#"{"ok":false,"error":"usage: delete table|condition"}"#.to_string();
                }
            } else {
                return r#"{"ok":false,"error":"usage: delete table|condition"}"#.to_string();
            };

            let mut db = load_db();

            if let Some(table) = db.get_mut(&table_name) {
                let parts: Vec<&str> = condition.splitn(2, '=').collect();
                let original_len = table.len();

                if parts.len() == 2 {
                    let field = parts[0];
                    let value = parts[1];

                    table.retain(|row| {
                        !(row.contains(&format!("\"{}\":", field)) &&
                          (row.contains(&format!("\"{}\":\"{}\"", field, value)) ||
                           row.contains(&format!("\"{}\":{}", field, value))))
                    });
                }

                let deleted = original_len - table.len();
                if let Err(e) = save_db(&db) {
                    return format!(r#"{{"ok":false,"error":"{}"}}"#, e);
                }
                format!(r#"{{"ok":true,"value":"deleted {} rows"}}"#, deleted)
            } else {
                format!(r#"{{"ok":false,"error":"table {} not found"}}"#, table_name)
            }
        }

        "drop_table" => {
            // 删除表
            // args[0] = 表名
            if let Some(table_name) = args.get(0) {
                let mut db = load_db();

                if db.remove(table_name).is_some() {
                    if let Err(e) = save_db(&db) {
                        return format!(r#"{{"ok":false,"error":"{}"}}"#, e);
                    }
                    format!(r#"{{"ok":true,"value":"table {} dropped"}}"#, table_name)
                } else {
                    format!(r#"{{"ok":false,"error":"table {} not found"}}"#, table_name)
                }
            } else {
                r#"{"ok":false,"error":"usage: drop_table table_name"}"#.to_string()
            }
        }

        "list_tables" => {
            let db = load_db();
            let tables: Vec<String> = db.keys()
                .map(|k| format!("\"{}\"", k))
                .collect();
            format!(r#"{{"ok":true,"value":[{}]}}"#, tables.join(","))
        }

        _ => format!(r#"{{"ok":false,"error":"unknown method: {}"}}"#, method),
    }
}

// 加载数据库
fn load_db() -> HashMap<String, Vec<String>> {
    let path = get_db_path();
    let content = fs::read_to_string(&path).unwrap_or_else(|_| r#"{"tables":{}}"#.to_string());

    // 解析 {"tables": {...}}
    let mut db = HashMap::new();

    if let Some(tables_start) = content.find("\"tables\":") {
        let rest = &content[tables_start + 9..];
        if let Some(obj_start) = rest.find('{') {
            let rest = &rest[obj_start..];

            // 简单解析表
            let mut depth = 0;
            let mut in_string = false;
            let mut current_table = String::new();
            let mut in_table_name = false;
            let mut in_table_array = false;
            let mut current_row = String::new();
            let mut row_depth = 0;
            let mut rows: Vec<String> = Vec::new();

            for c in rest.chars() {
                match c {
                    '"' if !in_string => {
                        in_string = true;
                        if depth == 1 && !in_table_array {
                            in_table_name = true;
                            current_table.clear();
                        }
                        // Add quote to row content if we're inside a row
                        if in_table_array && row_depth > 0 {
                            current_row.push(c);
                        }
                    }
                    '"' if in_string => {
                        in_string = false;
                        in_table_name = false;
                        // Add quote to row content if we're inside a row
                        if in_table_array && row_depth > 0 {
                            current_row.push(c);
                        }
                    }
                    _ if in_string && in_table_name => {
                        current_table.push(c);
                    }
                    '[' if !in_string && depth == 1 => {
                        in_table_array = true;
                        rows.clear();
                    }
                    ']' if !in_string && in_table_array && row_depth == 0 => {
                        in_table_array = false;
                        if !current_table.is_empty() {
                            db.insert(current_table.clone(), rows.clone());
                        }
                    }
                    '{' if !in_string => {
                        depth += 1;
                        if in_table_array {
                            row_depth += 1;
                            if row_depth == 1 {
                                current_row.clear();
                            }
                            current_row.push(c);
                        }
                    }
                    '}' if !in_string => {
                        if in_table_array && row_depth > 0 {
                            current_row.push(c);
                            row_depth -= 1;
                            if row_depth == 0 {
                                rows.push(current_row.clone());
                            }
                        }
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ if in_table_array && row_depth > 0 => {
                        current_row.push(c);
                    }
                    _ => {}
                }
            }
        }
    }

    db
}

// 保存数据库
fn save_db(db: &HashMap<String, Vec<String>>) -> Result<(), String> {
    let path = get_db_path();

    let mut tables_json = Vec::new();
    for (name, rows) in db {
        tables_json.push(format!("\"{}\":[{}]", name, rows.join(",")));
    }

    let json = format!(r#"{{"tables":{{{}}}}}"#, tables_json.join(","));

    fs::write(&path, &json).map_err(|e| e.to_string())
}

fn extract_string(json: &str, key: &str) -> String {
    // 尝试两种格式: "key":"value" 和 "key": "value" (有空格)
    for pattern in &[format!("\"{}\":\"", key), format!("\"{}\": \"", key)] {
        if let Some(start) = json.find(pattern) {
            let rest = &json[start + pattern.len()..];
            if let Some(end) = rest.find('"') {
                return rest[..end].to_string();
            }
        }
    }
    String::new()
}

fn extract_args(json: &str) -> Vec<String> {
    let mut args = Vec::new();

    // 尝试两种格式
    let (start_pos, offset) = if let Some(pos) = json.find("\"args\": [") {
        (Some(pos), 9)
    } else if let Some(pos) = json.find("\"args\":[") {
        (Some(pos), 8)
    } else {
        (None, 0)
    };

    if let Some(start) = start_pos {
        let rest = &json[start + offset..];

        // Find the closing ] that matches the opening [, accounting for strings
        let mut in_string = false;
        let mut escape_next = false;
        let mut bracket_depth = 1;
        let mut end_pos = rest.len();

        for (i, c) in rest.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }
            match c {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '[' if !in_string => bracket_depth += 1,
                ']' if !in_string => {
                    bracket_depth -= 1;
                    if bracket_depth == 0 {
                        end_pos = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        let args_str = &rest[..end_pos];

        // Parse strings within the array
        let mut in_string = false;
        let mut current = String::new();
        let mut chars = args_str.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '"' if !in_string => {
                    in_string = true;
                }
                '"' if in_string => {
                    in_string = false;
                    args.push(current.clone());
                    current.clear();
                }
                '\\' if in_string => {
                    if let Some(&next) = chars.peek() {
                        chars.next();
                        match next {
                            'n' => current.push('\n'),
                            'r' => current.push('\r'),
                            't' => current.push('\t'),
                            _ => current.push(next),
                        }
                    }
                }
                _ if in_string => {
                    current.push(c);
                }
                _ => {}
            }
        }
    }

    args
}
