// L-0 HTTP Plugin
// 简单的 HTTP 服务器：init, route, serve, stop, request
// 编译: rustc -O http_plugin.rs -o http_plugin

use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::time::Duration;

// 配置文件路径
const CONFIG_PATH: &str = ".l0_http_config.json";

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
            // 初始化 HTTP 服务器配置
            // args[0] = 端口号
            let port = args.get(0).map(|s| s.as_str()).unwrap_or("8080");

            let config = format!(r#"{{"port":{},"routes":{{}}}}"#, port);
            if let Err(e) = fs::write(CONFIG_PATH, &config) {
                return format!(r#"{{"ok":false,"error":"{}"}}"#, e);
            }

            format!(r#"{{"ok":true,"value":"HTTP server configured on port {}"}}"#, port)
        }

        "route" => {
            // 注册路由
            // Format: "METHOD /path|response_content" (pipe-separated)
            // Or: args[0] = "METHOD /path", args[1] = response_content
            let (route_spec, response_content) = if args.len() >= 2 {
                (args[0].clone(), args[1].clone())
            } else if let Some(arg) = args.get(0) {
                if let Some(idx) = arg.find('|') {
                    (arg[..idx].to_string(), arg[idx+1..].to_string())
                } else {
                    return r#"{"ok":false,"error":"usage: route 'METHOD /path|response_content'"}"#.to_string();
                }
            } else {
                return r#"{"ok":false,"error":"usage: route 'METHOD /path|response_content'"}"#.to_string();
            };

            if !route_spec.is_empty() {

                let mut config = load_config();
                config.insert(route_spec.clone(), response_content.clone());

                if let Err(e) = save_config(&config) {
                    return format!(r#"{{"ok":false,"error":"{}"}}"#, e);
                }

                format!(r#"{{"ok":true,"value":"route {} registered"}}"#, escape_json(&route_spec))
            } else {
                r#"{"ok":false,"error":"usage: route 'METHOD /path' response_content"}"#.to_string()
            }
        }

        "serve" => {
            // 启动服务器（阻塞模式，用于测试）
            // args[0] = 请求数量限制（可选，默认1）
            let max_requests: usize = args.get(0)
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);

            let config = load_config();
            let port = get_port();

            match TcpListener::bind(format!("127.0.0.1:{}", port)) {
                Ok(listener) => {
                    let mut count = 0;
                    for stream in listener.incoming() {
                        if count >= max_requests {
                            break;
                        }
                        if let Ok(stream) = stream {
                            handle_http_request(stream, &config);
                            count += 1;
                        }
                    }
                    format!(r#"{{"ok":true,"value":"served {} requests"}}"#, count)
                }
                Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
            }
        }

        "serve_once" => {
            // 启动服务器，处理一个请求后退出
            let config = load_config();
            let port = get_port();

            match TcpListener::bind(format!("127.0.0.1:{}", port)) {
                Ok(listener) => {
                    listener.set_nonblocking(false).ok();
                    if let Ok((stream, addr)) = listener.accept() {
                        let req_info = handle_http_request(stream, &config);
                        format!(r#"{{"ok":true,"value":"served request from {}"}}"#, addr)
                    } else {
                        r#"{"ok":false,"error":"no connection"}"#.to_string()
                    }
                }
                Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
            }
        }

        "request" => {
            // 发送 HTTP 请求（客户端模式）
            // args[0] = URL (如 "http://localhost:8080/hello")
            // args[1] = 方法 (GET/POST，默认 GET)
            // args[2] = 请求体（可选）
            if let Some(url) = args.get(0) {
                let method = args.get(1).map(|s| s.as_str()).unwrap_or("GET");
                let body = args.get(2).map(|s| s.as_str()).unwrap_or("");

                match http_request(url, method, body) {
                    Ok(response) => format!(r#"{{"ok":true,"value":"{}"}}"#, escape_json(&response)),
                    Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
                }
            } else {
                r#"{"ok":false,"error":"usage: request url [method] [body]"}"#.to_string()
            }
        }

        "list_routes" => {
            let config = load_config();
            let routes: Vec<String> = config.keys()
                .filter(|k| *k != "port")
                .map(|k| format!("\"{}\"", escape_json(k)))
                .collect();
            format!(r#"{{"ok":true,"value":[{}]}}"#, routes.join(","))
        }

        "get_port" => {
            let port = get_port();
            format!(r#"{{"ok":true,"value":"{}"}}"#, port)
        }

        _ => format!(r#"{{"ok":false,"error":"unknown method: {}"}}"#, method),
    }
}

fn handle_http_request(mut stream: TcpStream, routes: &HashMap<String, String>) -> String {
    let mut buffer = [0; 4096];
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let bytes_read = stream.read(&mut buffer).unwrap_or(0);
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);

    // 解析请求行: "GET /path HTTP/1.1"
    let first_line = request.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    let (method, path) = if parts.len() >= 2 {
        (parts[0], parts[1])
    } else {
        ("GET", "/")
    };

    let route_key = format!("{} {}", method, path);
    let route_key_any = format!("* {}", path);

    // 查找路由
    let response_body = routes.get(&route_key)
        .or_else(|| routes.get(&route_key_any))
        .or_else(|| routes.get("* *"))  // 默认路由
        .map(|s| s.as_str())
        .unwrap_or("404 Not Found");

    // 检查是否是文件路径
    let body = if response_body.starts_with("file:") {
        let file_path = &response_body[5..];
        fs::read_to_string(file_path).unwrap_or_else(|_| "404 File Not Found".to_string())
    } else {
        response_body.to_string()
    };

    let content_type = if body.contains("<html") || body.contains("<!DOCTYPE") {
        "text/html"
    } else if body.starts_with("{") || body.starts_with("[") {
        "application/json"
    } else {
        "text/plain"
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        content_type,
        body.len(),
        body
    );

    stream.write_all(response.as_bytes()).ok();
    stream.flush().ok();

    route_key
}

fn http_request(url: &str, method: &str, body: &str) -> Result<String, String> {
    // 简单解析 URL: http://host:port/path
    let url = url.trim_start_matches("http://");
    let (host_port, path) = if let Some(idx) = url.find('/') {
        (&url[..idx], &url[idx..])
    } else {
        (url, "/")
    };

    let (host, port) = if let Some(idx) = host_port.find(':') {
        (&host_port[..idx], host_port[idx+1..].parse().unwrap_or(80))
    } else {
        (host_port, 80)
    };

    let addr = format!("{}:{}", host, port);
    let mut stream = TcpStream::connect(&addr).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();

    let request = if body.is_empty() {
        format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            method, path, host_port
        )
    } else {
        format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            method, path, host_port, body.len(), body
        )
    };

    stream.write_all(request.as_bytes()).map_err(|e| e.to_string())?;

    let mut response = String::new();
    stream.read_to_string(&mut response).map_err(|e| e.to_string())?;

    // 提取响应体
    if let Some(idx) = response.find("\r\n\r\n") {
        Ok(response[idx+4..].to_string())
    } else {
        Ok(response)
    }
}

fn load_config() -> HashMap<String, String> {
    let content = fs::read_to_string(CONFIG_PATH)
        .unwrap_or_else(|_| r#"{"port":8080,"routes":{}}"#.to_string());

    let mut config = HashMap::new();

    // 解析 port
    if let Some(port_start) = content.find("\"port\":") {
        let rest = &content[port_start + 7..];
        let end = rest.find(|c: char| !c.is_numeric()).unwrap_or(rest.len());
        config.insert("port".to_string(), rest[..end].to_string());
    }

    // 解析 routes - Fixed state machine
    if let Some(routes_start) = content.find("\"routes\":{") {
        let rest = &content[routes_start + 10..];

        // Find the closing } that matches the opening {, accounting for strings
        let mut in_string = false;
        let mut escape_next = false;
        let mut brace_depth = 1;
        let mut end_pos = rest.len();

        for (i, c) in rest.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }
            match c {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => brace_depth += 1,
                '}' if !in_string => {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        end_pos = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        let routes_str = &rest[..end_pos];

        // Parse key-value pairs with proper state tracking
        let mut expecting_value = false;
        let mut in_string = false;
        let mut current_key = String::new();
        let mut current_value = String::new();
        let mut escape_next = false;

        for c in routes_str.chars() {
            if escape_next {
                if in_string {
                    if expecting_value {
                        current_value.push(c);
                    } else {
                        current_key.push(c);
                    }
                }
                escape_next = false;
                continue;
            }

            match c {
                '\\' if in_string => escape_next = true,
                '"' if !in_string => {
                    in_string = true;
                }
                '"' if in_string => {
                    in_string = false;
                    if expecting_value {
                        // End of value - store the pair
                        if !current_key.is_empty() {
                            config.insert(current_key.clone(), current_value.clone());
                        }
                        current_key.clear();
                        current_value.clear();
                        expecting_value = false;
                    }
                    // If not expecting_value, we just finished reading a key
                }
                ':' if !in_string && !expecting_value => {
                    expecting_value = true;
                }
                ',' if !in_string => {
                    // Reset for next pair (expecting_value should already be false)
                }
                _ if in_string => {
                    if expecting_value {
                        current_value.push(c);
                    } else {
                        current_key.push(c);
                    }
                }
                _ => {}
            }
        }
    }

    config
}

fn save_config(config: &HashMap<String, String>) -> Result<(), String> {
    let port = config.get("port").map(|s| s.as_str()).unwrap_or("8080");

    let routes: Vec<String> = config.iter()
        .filter(|(k, _)| *k != "port")
        .map(|(k, v)| format!("\"{}\":\"{}\"", escape_json(k), escape_json(v)))
        .collect();

    let json = format!(r#"{{"port":{},"routes":{{{}}}}}"#, port, routes.join(","));

    fs::write(CONFIG_PATH, &json).map_err(|e| e.to_string())
}

fn get_port() -> u16 {
    let config = load_config();
    config.get("port")
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080)
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
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

        // Now parse the strings within the array
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
