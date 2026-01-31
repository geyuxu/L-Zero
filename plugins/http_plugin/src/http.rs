// L-0 HTTP Plugin v2.0
// 支持: 静态文件、表单处理、数据库 CRUD API
// 编译: rustc -O http_plugin.rs -o http_plugin

use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::time::Duration;
use std::process::{Command, Stdio};

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
            let port = args.get(0).map(|s| s.as_str()).unwrap_or("8080");
            let config = format!(r#"{{"port":{},"routes":{{}},"static_dir":"","db_path":""}}"#, port);
            if let Err(e) = fs::write(CONFIG_PATH, &config) {
                return format!(r#"{{"ok":false,"error":"{}"}}"#, e);
            }
            format!(r#"{{"ok":true,"value":"HTTP server configured on port {}"}}"#, port)
        }

        "static" => {
            // 设置静态文件目录
            let dir = args.get(0).map(|s| s.as_str()).unwrap_or("");
            let mut config = load_config();
            config.insert("static_dir".to_string(), dir.to_string());
            if let Err(e) = save_config(&config) {
                return format!(r#"{{"ok":false,"error":"{}"}}"#, e);
            }
            format!(r#"{{"ok":true,"value":"Static directory set to: {}"}}"#, dir)
        }

        "db" => {
            // 设置数据库路径
            let path = args.get(0).map(|s| s.as_str()).unwrap_or("");
            let mut config = load_config();
            config.insert("db_path".to_string(), path.to_string());
            if let Err(e) = save_config(&config) {
                return format!(r#"{{"ok":false,"error":"{}"}}"#, e);
            }
            format!(r#"{{"ok":true,"value":"Database path set to: {}"}}"#, path)
        }

        "route" => {
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
            let config = load_config();
            let port = get_port();

            match TcpListener::bind(format!("127.0.0.1:{}", port)) {
                Ok(listener) => {
                    listener.set_nonblocking(false).ok();
                    if let Ok((stream, addr)) = listener.accept() {
                        handle_http_request(stream, &config);
                        format!(r#"{{"ok":true,"value":"served request from {}"}}"#, addr)
                    } else {
                        r#"{"ok":false,"error":"no connection"}"#.to_string()
                    }
                }
                Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
            }
        }

        "request" => {
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
                .filter(|k| !["port", "static_dir", "db_path"].contains(&k.as_str()))
                .map(|k| format!("\"{}\"", escape_json(k)))
                .collect();
            format!(r#"{{"ok":true,"value":[{}]}}"#, routes.join(","))
        }

        _ => format!(r#"{{"ok":false,"error":"unknown method: {}"}}"#, method),
    }
}

fn handle_http_request(mut stream: TcpStream, config: &HashMap<String, String>) {
    let mut buffer = [0; 16384];
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let bytes_read = stream.read(&mut buffer).unwrap_or(0);
    let request = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

    let first_line = request.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    let (method, path) = if parts.len() >= 2 {
        (parts[0], parts[1])
    } else {
        ("GET", "/")
    };

    let static_dir = config.get("static_dir").map(|s| s.as_str()).unwrap_or("");
    let db_path = config.get("db_path").map(|s| s.as_str()).unwrap_or("");

    // 处理请求
    let (status, content_type, body) = route_request(method, path, &request, config, static_dir, db_path);

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status, content_type, body.len(), body
    );

    stream.write_all(response.as_bytes()).ok();
    stream.flush().ok();
}

fn route_request(method: &str, path: &str, request: &str, config: &HashMap<String, String>,
                 static_dir: &str, db_path: &str) -> (&'static str, &'static str, String) {

    // 1. API 路由 - /api/posts CRUD
    if path.starts_with("/api/") {
        return handle_api(method, path, request, db_path);
    }

    // 2. POST 表单处理
    if method == "POST" {
        if let Some(result) = handle_form_post(path, request, db_path) {
            return result;
        }
    }

    // 3. 已注册的静态路由
    let route_key = format!("{} {}", method, path);
    if let Some(content) = config.get(&route_key) {
        let ct = guess_content_type(content);
        return ("200 OK", ct, content.clone());
    }

    // 4. 静态文件服务
    if !static_dir.is_empty() && method == "GET" {
        let file_path = if path == "/" {
            format!("{}/index.html", static_dir)
        } else {
            format!("{}{}", static_dir, path)
        };

        if let Ok(content) = fs::read_to_string(&file_path) {
            let ct = guess_content_type_by_ext(&file_path);
            return ("200 OK", ct, content);
        }
    }

    // 5. 通配符路由
    let route_any = format!("* {}", path);
    if let Some(content) = config.get(&route_any) {
        let ct = guess_content_type(content);
        return ("200 OK", ct, content.clone());
    }

    if let Some(content) = config.get("* *") {
        let ct = guess_content_type(content);
        return ("200 OK", ct, content.clone());
    }

    ("404 Not Found", "text/plain", "404 Not Found".to_string())
}

fn handle_api(method: &str, path: &str, request: &str, db_path: &str) -> (&'static str, &'static str, String) {
    // /api/posts - 帖子 CRUD
    // /api/posts/1 - 单个帖子操作

    let parts: Vec<&str> = path.trim_start_matches("/api/").split('/').collect();
    let table = parts.get(0).unwrap_or(&"");
    let id = parts.get(1).unwrap_or(&"");

    if table.is_empty() {
        return ("400 Bad Request", "application/json", r#"{"error":"No table specified"}"#.to_string());
    }

    let result = match method {
        "GET" => {
            if id.is_empty() {
                // GET /api/posts - 获取所有
                call_db("select", table, db_path)
            } else {
                // GET /api/posts/1 - 获取单个
                let args = format!("{}|id={}", table, id);
                call_db("select", &args, db_path)
            }
        }
        "POST" => {
            // POST /api/posts - 创建
            let body = get_request_body(request);
            if let Some(json) = parse_json_body(&body) {
                let cols: Vec<&str> = json.keys().map(|s| s.as_str()).collect();
                let vals: Vec<&str> = json.values().map(|s| s.as_str()).collect();
                let args = format!("{}|{}|{}", table, cols.join(","), vals.join(","));
                call_db("insert", &args, db_path)
            } else {
                r#"{"ok":false,"error":"Invalid JSON body"}"#.to_string()
            }
        }
        "PUT" | "PATCH" => {
            // PUT /api/posts/1 - 更新
            if id.is_empty() {
                return ("400 Bad Request", "application/json", r#"{"error":"ID required for update"}"#.to_string());
            }
            let body = get_request_body(request);
            if let Some(json) = parse_json_body(&body) {
                let updates: Vec<String> = json.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
                let args = format!("{}|id={}|{}", table, id, updates.join(","));
                call_db("update", &args, db_path)
            } else {
                r#"{"ok":false,"error":"Invalid JSON body"}"#.to_string()
            }
        }
        "DELETE" => {
            // DELETE /api/posts/1 - 删除
            if id.is_empty() {
                return ("400 Bad Request", "application/json", r#"{"error":"ID required for delete"}"#.to_string());
            }
            let args = format!("{}|id={}", table, id);
            call_db("delete", &args, db_path)
        }
        _ => r#"{"ok":false,"error":"Method not allowed"}"#.to_string()
    };

    ("200 OK", "application/json", result)
}

fn handle_form_post(path: &str, request: &str, db_path: &str) -> Option<(&'static str, &'static str, String)> {
    let form = parse_form_data(request);

    match path {
        "/create" => {
            let title = form.get("title").map(|s| s.as_str()).unwrap_or("");
            let content = form.get("content").map(|s| s.as_str()).unwrap_or("");
            let author = form.get("author").map(|s| s.as_str()).unwrap_or("Anonymous");

            if title.is_empty() {
                return Some(("200 OK", "text/html", error_page("Title is required")));
            }

            let args = format!("posts|title,content,author|{},{},{}",
                sanitize(title), sanitize(content), sanitize(author));

            let result = call_db("insert", &args, db_path);
            if result.contains("\"ok\":true") {
                Some(("200 OK", "text/html", success_page("Post created!")))
            } else {
                Some(("200 OK", "text/html", error_page(&format!("Error: {}", result))))
            }
        }
        "/update" => {
            let id = form.get("id").map(|s| s.as_str()).unwrap_or("");
            let title = form.get("title").map(|s| s.as_str()).unwrap_or("");
            let content = form.get("content").map(|s| s.as_str()).unwrap_or("");

            if id.is_empty() {
                return Some(("200 OK", "text/html", error_page("Post ID is required")));
            }

            let mut updates = Vec::new();
            if !title.is_empty() { updates.push(format!("title={}", sanitize(title))); }
            if !content.is_empty() { updates.push(format!("content={}", sanitize(content))); }

            if updates.is_empty() {
                return Some(("200 OK", "text/html", error_page("Nothing to update")));
            }

            let args = format!("posts|id={}|{}", id, updates.join(","));
            let result = call_db("update", &args, db_path);
            if result.contains("\"ok\":true") {
                Some(("200 OK", "text/html", success_page("Post updated!")))
            } else {
                Some(("200 OK", "text/html", error_page(&format!("Error: {}", result))))
            }
        }
        "/delete" => {
            let id = form.get("id").map(|s| s.as_str()).unwrap_or("");

            if id.is_empty() {
                return Some(("200 OK", "text/html", error_page("Post ID is required")));
            }

            let args = format!("posts|id={}", id);
            let result = call_db("delete", &args, db_path);
            if result.contains("\"ok\":true") {
                Some(("200 OK", "text/html", success_page("Post deleted!")))
            } else {
                Some(("200 OK", "text/html", error_page(&format!("Error: {}", result))))
            }
        }
        _ => None
    }
}

fn call_db(method: &str, args: &str, db_path: &str) -> String {
    let db_plugin = "tools/db/target/release/db_plugin";

    // 确保数据库已初始化
    if !db_path.is_empty() {
        let init_req = format!(r#"{{"method":"init","args":["{}"]}}"#, db_path);
        if let Ok(mut child) = Command::new(db_plugin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(init_req.as_bytes()).ok();
                stdin.write_all(b"\n").ok();
            }
            child.wait().ok();
        }
    }

    let request = format!(r#"{{"method":"{}","args":["{}"]}}"#, method, args.replace('"', "\\\""));

    match Command::new(db_plugin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn() {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(request.as_bytes()).ok();
                stdin.write_all(b"\n").ok();
            }
            match child.wait_with_output() {
                Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
                Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
            }
        }
        Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
    }
}

fn sanitize(s: &str) -> String {
    s.replace('|', " ").replace(',', " ").replace('"', "'")
}

fn parse_form_data(request: &str) -> HashMap<String, String> {
    let mut data = HashMap::new();
    if let Some(idx) = request.find("\r\n\r\n") {
        let body = &request[idx + 4..];
        for pair in body.split('&') {
            if let Some(eq_idx) = pair.find('=') {
                let key = url_decode(&pair[..eq_idx]);
                let value = url_decode(&pair[eq_idx + 1..]);
                data.insert(key, value);
            }
        }
    }
    data
}

fn get_request_body(request: &str) -> String {
    if let Some(idx) = request.find("\r\n\r\n") {
        request[idx + 4..].to_string()
    } else {
        String::new()
    }
}

fn parse_json_body(body: &str) -> Option<HashMap<String, String>> {
    let mut map = HashMap::new();
    let body = body.trim();
    if !body.starts_with('{') || !body.ends_with('}') {
        return None;
    }

    let inner = &body[1..body.len()-1];
    for pair in inner.split(',') {
        let parts: Vec<&str> = pair.splitn(2, ':').collect();
        if parts.len() == 2 {
            let key = parts[0].trim().trim_matches('"');
            let value = parts[1].trim().trim_matches('"');
            map.insert(key.to_string(), value.to_string());
        }
    }
    Some(map)
}

fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '%' => {
                let hex: String = chars.by_ref().take(2).collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                }
            }
            '+' => result.push(' '),
            _ => result.push(c),
        }
    }
    result
}

fn success_page(msg: &str) -> String {
    format!(r#"<!DOCTYPE html><html><head><meta charset='UTF-8'><meta http-equiv='refresh' content='1;url=/'><title>Success</title>
<style>body{{font-family:sans-serif;display:flex;justify-content:center;align-items:center;height:100vh;background:linear-gradient(135deg,#667eea,#764ba2);margin:0}}
.box{{background:white;padding:40px;border-radius:16px;text-align:center;box-shadow:0 10px 40px rgba(0,0,0,0.2)}}
h1{{color:#27ae60}}p{{color:#666}}</style></head>
<body><div class='box'><h1>✓ Success</h1><p>{}</p></div></body></html>"#, msg)
}

fn error_page(msg: &str) -> String {
    format!(r#"<!DOCTYPE html><html><head><meta charset='UTF-8'><meta http-equiv='refresh' content='3;url=/'><title>Error</title>
<style>body{{font-family:sans-serif;display:flex;justify-content:center;align-items:center;height:100vh;background:linear-gradient(135deg,#667eea,#764ba2);margin:0}}
.box{{background:white;padding:40px;border-radius:16px;text-align:center;box-shadow:0 10px 40px rgba(0,0,0,0.2)}}
h1{{color:#e74c3c}}p{{color:#666}}</style></head>
<body><div class='box'><h1>✗ Error</h1><p>{}</p></div></body></html>"#, msg)
}

fn guess_content_type(content: &str) -> &'static str {
    if content.contains("<html") || content.contains("<!DOCTYPE") { "text/html" }
    else if content.starts_with("{") || content.starts_with("[") { "application/json" }
    else { "text/plain" }
}

fn guess_content_type_by_ext(path: &str) -> &'static str {
    if path.ends_with(".html") || path.ends_with(".htm") { "text/html" }
    else if path.ends_with(".css") { "text/css" }
    else if path.ends_with(".js") { "application/javascript" }
    else if path.ends_with(".json") { "application/json" }
    else if path.ends_with(".png") { "image/png" }
    else if path.ends_with(".jpg") || path.ends_with(".jpeg") { "image/jpeg" }
    else if path.ends_with(".svg") { "image/svg+xml" }
    else { "text/plain" }
}

fn http_request(url: &str, method: &str, body: &str) -> Result<String, String> {
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
        format!("{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", method, path, host_port)
    } else {
        format!("{} {} HTTP/1.1\r\nHost: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            method, path, host_port, body.len(), body)
    };

    stream.write_all(request.as_bytes()).map_err(|e| e.to_string())?;

    let mut response = String::new();
    stream.read_to_string(&mut response).map_err(|e| e.to_string())?;

    if let Some(idx) = response.find("\r\n\r\n") {
        Ok(response[idx+4..].to_string())
    } else {
        Ok(response)
    }
}

fn load_config() -> HashMap<String, String> {
    let content = fs::read_to_string(CONFIG_PATH)
        .unwrap_or_else(|_| r#"{"port":8080,"routes":{},"static_dir":"","db_path":""}"#.to_string());

    let mut config = HashMap::new();

    // 解析简单字段
    for key in ["port", "static_dir", "db_path"] {
        for pattern in [format!("\"{}\":\"", key), format!("\"{}\": \"", key)] {
            if let Some(start) = content.find(&pattern) {
                let rest = &content[start + pattern.len()..];
                if let Some(end) = rest.find('"') {
                    config.insert(key.to_string(), rest[..end].to_string());
                    break;
                }
            }
        }
        // 数字值 (port)
        if key == "port" && !config.contains_key(key) {
            if let Some(start) = content.find("\"port\":") {
                let rest = &content[start + 7..];
                let end = rest.find(|c: char| !c.is_numeric()).unwrap_or(rest.len());
                if end > 0 {
                    config.insert(key.to_string(), rest[..end].to_string());
                }
            }
        }
    }

    // 解析 routes
    if let Some(routes_start) = content.find("\"routes\":{") {
        let rest = &content[routes_start + 10..];
        let mut in_string = false;
        let mut escape_next = false;
        let mut brace_depth = 1;
        let mut end_pos = rest.len();

        for (i, c) in rest.char_indices() {
            if escape_next { escape_next = false; continue; }
            match c {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => brace_depth += 1,
                '}' if !in_string => {
                    brace_depth -= 1;
                    if brace_depth == 0 { end_pos = i; break; }
                }
                _ => {}
            }
        }

        let routes_str = &rest[..end_pos];
        let mut expecting_value = false;
        let mut in_string = false;
        let mut current_key = String::new();
        let mut current_value = String::new();
        let mut escape_next = false;

        for c in routes_str.chars() {
            if escape_next {
                if in_string {
                    let unescaped = match c {
                        'n' => '\n', 'r' => '\r', 't' => '\t',
                        '"' => '"', '\\' => '\\', _ => c,
                    };
                    if expecting_value { current_value.push(unescaped); }
                    else { current_key.push(unescaped); }
                }
                escape_next = false;
                continue;
            }
            match c {
                '\\' if in_string => escape_next = true,
                '"' if !in_string => in_string = true,
                '"' if in_string => {
                    in_string = false;
                    if expecting_value {
                        if !current_key.is_empty() {
                            config.insert(current_key.clone(), current_value.clone());
                        }
                        current_key.clear();
                        current_value.clear();
                        expecting_value = false;
                    }
                }
                ':' if !in_string && !expecting_value => expecting_value = true,
                ',' if !in_string => {}
                _ if in_string => {
                    if expecting_value { current_value.push(c); }
                    else { current_key.push(c); }
                }
                _ => {}
            }
        }
    }

    config
}

fn save_config(config: &HashMap<String, String>) -> Result<(), String> {
    let port = config.get("port").map(|s| s.as_str()).unwrap_or("8080");
    let static_dir = config.get("static_dir").map(|s| s.as_str()).unwrap_or("");
    let db_path = config.get("db_path").map(|s| s.as_str()).unwrap_or("");

    let routes: Vec<String> = config.iter()
        .filter(|(k, _)| !["port", "static_dir", "db_path"].contains(&k.as_str()))
        .map(|(k, v)| format!("\"{}\":\"{}\"", escape_json(k), escape_json(v)))
        .collect();

    let json = format!(
        r#"{{"port":{},"static_dir":"{}","db_path":"{}","routes":{{{}}}}}"#,
        port, escape_json(static_dir), escape_json(db_path), routes.join(",")
    );

    fs::write(CONFIG_PATH, &json).map_err(|e| e.to_string())
}

fn get_port() -> u16 {
    let config = load_config();
    config.get("port").and_then(|s| s.parse().ok()).unwrap_or(8080)
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
}

fn extract_string(json: &str, key: &str) -> String {
    for pattern in [format!("\"{}\": \"", key), format!("\"{}\":\"", key)] {
        if let Some(start) = json.find(&pattern) {
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
    let (start_opt, offset) = if let Some(start) = json.find("\"args\": [") {
        (Some(start), 9)
    } else if let Some(start) = json.find("\"args\":[") {
        (Some(start), 8)
    } else {
        (None, 0)
    };

    if let Some(start) = start_opt {
        let rest = &json[start + offset..];
        if let Some(end) = rest.find(']') {
            let args_str = &rest[..end];
            let mut in_string = false;
            let mut current = String::new();
            let mut chars = args_str.chars().peekable();

            while let Some(c) = chars.next() {
                match c {
                    '"' if !in_string => in_string = true,
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
                    _ if in_string => current.push(c),
                    _ => {}
                }
            }
        }
    }
    args
}
