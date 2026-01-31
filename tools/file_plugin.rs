// L-0 File Plugin
// 处理文件操作：read, write, list, exists, delete
// 编译: rustc -O file_plugin.rs -o file_plugin

use std::fs;
use std::io::{self, BufRead};

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
    // 解析 {"method":"xxx","args":[...]}
    let method = extract_string(request, "method");
    let args = extract_args(request);

    match method.as_str() {
        "read" => {
            if let Some(path) = args.get(0) {
                match fs::read_to_string(path) {
                    Ok(content) => format!(r#"{{"ok":true,"value":"{}"}}"#,
                        content.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")),
                    Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
                }
            } else {
                r#"{"ok":false,"error":"missing path"}"#.to_string()
            }
        }

        "write" => {
            if args.len() >= 2 {
                match fs::write(&args[0], &args[1]) {
                    Ok(_) => r#"{"ok":true,"value":true}"#.to_string(),
                    Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
                }
            } else {
                r#"{"ok":false,"error":"missing path or content"}"#.to_string()
            }
        }

        "list" => {
            if let Some(path) = args.get(0) {
                match fs::read_dir(path) {
                    Ok(entries) => {
                        let files: Vec<String> = entries
                            .filter_map(|e| e.ok())
                            .map(|e| format!("\"{}\"", e.file_name().to_string_lossy()))
                            .collect();
                        format!(r#"{{"ok":true,"value":[{}]}}"#, files.join(","))
                    }
                    Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
                }
            } else {
                r#"{"ok":false,"error":"missing path"}"#.to_string()
            }
        }

        "exists" => {
            if let Some(path) = args.get(0) {
                let exists = std::path::Path::new(path).exists();
                format!(r#"{{"ok":true,"value":{}}}"#, exists)
            } else {
                r#"{"ok":false,"error":"missing path"}"#.to_string()
            }
        }

        "delete" => {
            if let Some(path) = args.get(0) {
                match fs::remove_file(path) {
                    Ok(_) => r#"{"ok":true,"value":true}"#.to_string(),
                    Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
                }
            } else {
                r#"{"ok":false,"error":"missing path"}"#.to_string()
            }
        }

        _ => format!(r#"{{"ok":false,"error":"unknown method: {}"}}"#, method),
    }
}

fn extract_string(json: &str, key: &str) -> String {
    let pattern = format!("\"{}\":\"", key);
    if let Some(start) = json.find(&pattern) {
        let rest = &json[start + pattern.len()..];
        if let Some(end) = rest.find('"') {
            return rest[..end].to_string();
        }
    }
    String::new()
}

fn extract_args(json: &str) -> Vec<String> {
    let mut args = Vec::new();

    if let Some(start) = json.find("\"args\":[") {
        let rest = &json[start + 8..];
        if let Some(end) = rest.find(']') {
            let args_str = &rest[..end];

            // 简单解析字符串数组
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
                            current.push(next);
                        }
                    }
                    _ if in_string => {
                        current.push(c);
                    }
                    _ => {}
                }
            }
        }
    }

    args
}
