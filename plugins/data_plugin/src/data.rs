// L-0 Data Plugin
// 处理 JSON 数据操作：json_load, json_save, json_get, json_set
// 编译: rustc -O data_plugin.rs -o data_plugin

use std::fs;
use std::io::{self, BufRead};
use std::collections::HashMap;

// 简单的内存 JSON 存储
static mut JSON_STORE: Option<HashMap<String, String>> = None;

fn get_store() -> &'static mut HashMap<String, String> {
    unsafe {
        if JSON_STORE.is_none() {
            JSON_STORE = Some(HashMap::new());
        }
        JSON_STORE.as_mut().unwrap()
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
        "json_load" => {
            // 从文件加载 JSON 到内存
            // args[0] = 文件路径, args[1] = key (存储键)
            if args.len() >= 2 {
                match fs::read_to_string(&args[0]) {
                    Ok(content) => {
                        get_store().insert(args[1].clone(), content.clone());
                        format!(r#"{{"ok":true,"value":"{}"}}"#,
                            escape_json(&content))
                    }
                    Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
                }
            } else if args.len() == 1 {
                // 只加载返回内容
                match fs::read_to_string(&args[0]) {
                    Ok(content) => format!(r#"{{"ok":true,"value":"{}"}}"#,
                        escape_json(&content)),
                    Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
                }
            } else {
                r#"{"ok":false,"error":"usage: json_load path [key]"}"#.to_string()
            }
        }

        "json_save" => {
            // 保存 JSON 到文件
            // args[0] = 文件路径, args[1] = JSON 内容
            if args.len() >= 2 {
                match fs::write(&args[0], &args[1]) {
                    Ok(_) => r#"{"ok":true,"value":"saved"}"#.to_string(),
                    Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
                }
            } else {
                r#"{"ok":false,"error":"usage: json_save path content"}"#.to_string()
            }
        }

        "json_get" => {
            // 从内存获取 JSON
            // args[0] = key
            if let Some(key) = args.get(0) {
                if let Some(value) = get_store().get(key) {
                    format!(r#"{{"ok":true,"value":"{}"}}"#, escape_json(value))
                } else {
                    r#"{"ok":false,"error":"key not found"}"#.to_string()
                }
            } else {
                r#"{"ok":false,"error":"usage: json_get key"}"#.to_string()
            }
        }

        "json_set" => {
            // 设置内存中的 JSON
            // args[0] = key, args[1] = value
            if args.len() >= 2 {
                get_store().insert(args[0].clone(), args[1].clone());
                r#"{"ok":true,"value":"set"}"#.to_string()
            } else {
                r#"{"ok":false,"error":"usage: json_set key value"}"#.to_string()
            }
        }

        "json_parse" => {
            // 解析 JSON 并提取字段
            // args[0] = JSON 字符串, args[1] = 字段路径 (如 "user.name")
            if args.len() >= 2 {
                let value = extract_json_field(&args[0], &args[1]);
                format!(r#"{{"ok":true,"value":"{}"}}"#, escape_json(&value))
            } else {
                r#"{"ok":false,"error":"usage: json_parse json_str field_path"}"#.to_string()
            }
        }

        _ => format!(r#"{{"ok":false,"error":"unknown method: {}"}}"#, method),
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
}

fn extract_json_field(json: &str, path: &str) -> String {
    // 简单的 JSON 字段提取 (支持 obj.field 格式)
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = json.to_string();

    for part in parts {
        let pattern = format!("\"{}\":", part);
        if let Some(start) = current.find(&pattern) {
            let rest = &current[start + pattern.len()..];
            let rest = rest.trim_start();

            if rest.starts_with('"') {
                // String value
                let inner = &rest[1..];
                if let Some(end) = inner.find('"') {
                    current = inner[..end].to_string();
                } else {
                    return String::new();
                }
            } else if rest.starts_with('{') {
                // Object value
                let mut depth = 0;
                let mut end_idx = 0;
                for (i, c) in rest.char_indices() {
                    match c {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                end_idx = i + 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                current = rest[..end_idx].to_string();
            } else if rest.starts_with('[') {
                // Array value
                let mut depth = 0;
                let mut end_idx = 0;
                for (i, c) in rest.char_indices() {
                    match c {
                        '[' => depth += 1,
                        ']' => {
                            depth -= 1;
                            if depth == 0 {
                                end_idx = i + 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                current = rest[..end_idx].to_string();
            } else {
                // Number/bool value
                let end = rest.find(|c| c == ',' || c == '}' || c == ']')
                    .unwrap_or(rest.len());
                current = rest[..end].trim().to_string();
            }
        } else {
            return String::new();
        }
    }

    current
}

fn extract_string(json: &str, key: &str) -> String {
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

    // Try with space: "args": [
    let (start_pos, offset) = if let Some(pos) = json.find("\"args\": [") {
        (Some(pos), 9)
    } else if let Some(pos) = json.find("\"args\":[") {
        (Some(pos), 8)
    } else {
        (None, 0)
    };

    if let Some(start) = start_pos {
        let rest = &json[start + offset..];

        // Parse args array, properly handling strings with brackets
        let mut in_string = false;
        let mut current = String::new();
        let mut chars = rest.chars().peekable();
        let mut escape_next = false;

        while let Some(c) = chars.next() {
            if escape_next {
                // Handle JSON escape sequences
                match c {
                    'n' => current.push('\n'),
                    'r' => current.push('\r'),
                    't' => current.push('\t'),
                    '\\' => current.push('\\'),
                    '"' => current.push('"'),
                    _ => current.push(c),
                }
                escape_next = false;
                continue;
            }

            match c {
                '\\' if in_string => {
                    escape_next = true;
                }
                '"' if !in_string => {
                    in_string = true;
                }
                '"' if in_string => {
                    in_string = false;
                    args.push(current.clone());
                    current.clear();
                }
                ']' if !in_string => {
                    // End of args array - only break when NOT inside a string
                    break;
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
