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
                    Ok(content) => {
                        // Properly escape for JSON string
                        let escaped = content
                            .replace('\\', "\\\\")
                            .replace('"', "\\\"")
                            .replace('\n', "\\n")
                            .replace('\r', "\\r")
                            .replace('\t', "\\t");
                        format!(r#"{{"ok":true,"value":"{}"}}"#, escaped)
                    },
                    Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
                }
            } else {
                r#"{"ok":false,"error":"missing path"}"#.to_string()
            }
        }

        "write" => {
            // Format: either args[0]=path, args[1]=content OR args[0]="path|content"
            let (path, content) = if args.len() >= 2 {
                (args[0].clone(), args[1].clone())
            } else if let Some(arg) = args.get(0) {
                // Split on first "|" separator
                if let Some(pos) = arg.find('|') {
                    (arg[..pos].to_string(), arg[pos+1..].to_string())
                } else {
                    return r#"{"ok":false,"error":"missing content (use path|content format)"}"#.to_string();
                }
            } else {
                return r#"{"ok":false,"error":"missing path or content"}"#.to_string();
            };

            match fs::write(&path, &content) {
                Ok(_) => r#"{"ok":true,"value":true}"#.to_string(),
                Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
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
    // Try with space: "key": "value"
    let pattern_space = format!("\"{}\": \"", key);
    if let Some(start) = json.find(&pattern_space) {
        let rest = &json[start + pattern_space.len()..];
        if let Some(end) = rest.find('"') {
            return rest[..end].to_string();
        }
    }
    // Try without space: "key":"value"
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

    // Try with space: "args": [
    let (start_opt, offset) = if let Some(start) = json.find("\"args\": [") {
        (Some(start), 9)
    } else if let Some(start) = json.find("\"args\":[") {
        (Some(start), 8)
    } else {
        (None, 0)
    };

    if let Some(start) = start_opt {
        let rest = &json[start + offset..];

        // Parse args array, properly handling strings with escaped characters
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
                    // End of args array
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
