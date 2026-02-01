// L-0 HTTP Plugin v3.0
// 支持: 静态文件、静态路由、动态响应模式 (HTTP_LISTEN/HTTP_SEND)
// 编译: rustc -O http_plugin.rs -o http_plugin

use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

extern crate libc;

// 配置文件路径
const CONFIG_PATH: &str = ".l0_http_config.json";
// 动态模式 - 请求/响应通过文件传递
const PENDING_REQUEST_PATH: &str = ".l0_http_pending_request.json";
const PENDING_RESPONSE_PATH: &str = ".l0_http_pending_response.json";

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
            // Load existing config to preserve routes, or create new if not exists
            let mut config = load_config();
            config.insert("port".to_string(), port.to_string());
            if let Err(e) = save_config(&config) {
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

        "route" => {
            let (route_spec, response_content) = if args.len() >= 2 {
                (args[0].clone(), args[1].clone())
            } else if let Some(arg) = args.get(0) {
                if let Some(idx) = arg.find('|') {
                    (arg[..idx].to_string(), arg[idx+1..].to_string())
                } else {
                    return r#"{"ok":false,"error":"Missing '|' separator in route","usage":"METHOD /path|response_content","examples":["GET /api/data|{\"result\":\"ok\"}","GET /|<html>Welcome</html>"]}"#.to_string();
                }
            } else {
                return r#"{"ok":false,"error":"Missing route argument","usage":"METHOD /path|response_content","examples":["GET /api/data|{\"result\":\"ok\"}","POST /submit|{\"status\":\"received\"}"]}"#.to_string();
            };

            if !route_spec.is_empty() {
                // Validate route format
                if !route_spec.contains(' ') {
                    return format!(r#"{{"ok":false,"error":"Invalid route format - missing space between METHOD and path","got":"{}","usage":"METHOD /path|content","examples":["GET /api/test|ok","POST /data|received"]}}"#, escape_json(&route_spec));
                }
                let mut config = load_config();
                config.insert(route_spec.clone(), response_content.clone());
                if let Err(e) = save_config(&config) {
                    return format!(r#"{{"ok":false,"error":"Failed to save route: {}"}}"#, e);
                }
                format!(r#"{{"ok":true,"value":"route {} registered"}}"#, escape_json(&route_spec))
            } else {
                r#"{"ok":false,"error":"Empty route specification","usage":"METHOD /path|response_content"}"#.to_string()
            }
        }

        "serve" => {
            let max_requests: usize = args.get(0)
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);

            let config = Arc::new(load_config());
            let port = get_port();

            match TcpListener::bind(format!("127.0.0.1:{}", port)) {
                Ok(listener) => {
                    let count = Arc::new(AtomicUsize::new(0));
                    let mut handles = Vec::new();

                    for stream in listener.incoming() {
                        let current = count.load(Ordering::SeqCst);
                        if current >= max_requests {
                            break;
                        }

                        if let Ok(stream) = stream {
                            let config_clone = Arc::clone(&config);
                            let count_clone = Arc::clone(&count);

                            // Spawn thread to handle request concurrently
                            let handle = thread::spawn(move || {
                                handle_http_request_arc(stream, &config_clone);
                                count_clone.fetch_add(1, Ordering::SeqCst);
                            });
                            handles.push(handle);
                        }
                    }

                    // Wait for all active handlers to complete
                    for handle in handles {
                        handle.join().ok();
                    }

                    let final_count = count.load(Ordering::SeqCst);
                    format!(r#"{{"ok":true,"value":"served {} requests"}}"#, final_count)
                }
                Err(e) => {
                    let hint = if e.to_string().contains("Address already in use") {
                        format!(r#","hint":"Port {} is busy. Wait ~30s for TCP TIME_WAIT or use HTTP_INIT to change port.""#, port)
                    } else {
                        String::new()
                    };
                    format!(r#"{{"ok":false,"error":"Failed to bind port {}: {}"{}}}"#, port, e, hint)
                }
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
                .filter(|k| !["port", "static_dir"].contains(&k.as_str()))
                .map(|k| format!("\"{}\"", escape_json(k)))
                .collect();
            format!(r#"{{"ok":true,"value":[{}]}}"#, routes.join(","))
        }

        // ========== Dynamic Response Mode ==========
        // Architecture: Daemon server + file-based IPC
        // 1. HTTP_LISTEN starts daemon if needed, then polls for requests
        // 2. Daemon: accepts connection -> writes request file -> waits for response file -> sends
        // 3. HTTP_SEND writes response file for daemon to pick up

        "listen" => {
            let port = get_port();

            // Don't clean up response file here - the daemon needs to read it!
            // Only clean up old request files
            let _ = fs::remove_file(PENDING_REQUEST_PATH);
            // Note: PENDING_RESPONSE_PATH is cleaned by daemon after sending

            // Start daemon server if not running
            // PID file is per-port to support multiple servers
            let pid_file = format!("/tmp/l0_http_daemon_{}.pid", port);
            let daemon_running = if let Ok(pid_str) = fs::read_to_string(&pid_file) {
                // Check if process is actually running
                if let Ok(pid) = pid_str.trim().parse::<i32>() {
                    unsafe { libc::kill(pid, 0) == 0 }
                } else {
                    false
                }
            } else {
                false
            };

            if !daemon_running {
                // Start daemon: this process forks and the child runs the server loop
                match unsafe { libc::fork() } {
                    -1 => return r#"{"ok":false,"error":"Fork failed"}"#.to_string(),
                    0 => {
                        // Child process - become daemon
                        unsafe { libc::setsid() }; // New session

                        // Write our PID
                        let pid = unsafe { libc::getpid() };
                        let _ = fs::write(pid_file, pid.to_string());

                        // Start server loop
                        if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
                            listener.set_nonblocking(false).ok();

                            // Debug: write to a log file
                            let _ = fs::write("/tmp/l0_http_daemon.log", format!("Daemon started on port {}\n", port));

                            loop {
                                match listener.accept() {
                                    Ok((mut stream, addr)) => {
                                        let _ = fs::OpenOptions::new().append(true).open("/tmp/l0_http_daemon.log")
                                            .and_then(|mut f| std::io::Write::write_all(&mut f, format!("Accepted from {}\n", addr).as_bytes()));

                                        // Read request
                                        let mut buffer = [0; 16384];
                                        stream.set_read_timeout(Some(Duration::from_secs(30))).ok();

                                        let bytes_read = stream.read(&mut buffer).unwrap_or(0);
                                        let _ = fs::OpenOptions::new().append(true).open("/tmp/l0_http_daemon.log")
                                            .and_then(|mut f| std::io::Write::write_all(&mut f, format!("Read {} bytes\n", bytes_read).as_bytes()));

                                        let request = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

                                        // Parse request
                                        let first_line = request.lines().next().unwrap_or("");
                                        let parts: Vec<&str> = first_line.split_whitespace().collect();
                                        let (method, path) = if parts.len() >= 2 {
                                            (parts[0], parts[1])
                                        } else {
                                            ("GET", "/")
                                        };

                                        let body = get_request_body(&request);

                                        // Debug: log body
                                        let _ = fs::OpenOptions::new().append(true).open("/tmp/l0_http_daemon.log")
                                            .and_then(|mut f| std::io::Write::write_all(&mut f, format!("Body extracted: '{}' (len={})\n", body, body.len()).as_bytes()));

                                        // Write request info for VM to read
                                        let request_json = format!(
                                            r#"{{"method":"{}","path":"{}","body":"{}","addr":"{}"}}"#,
                                            method,
                                            escape_json(path),
                                            escape_json(&body),
                                            addr
                                        );

                                        let _ = fs::OpenOptions::new().append(true).open("/tmp/l0_http_daemon.log")
                                            .and_then(|mut f| std::io::Write::write_all(&mut f, format!("Writing to: {}\n", PENDING_REQUEST_PATH).as_bytes()));

                                        match fs::write(PENDING_REQUEST_PATH, &request_json) {
                                            Ok(_) => {
                                                let _ = fs::OpenOptions::new().append(true).open("/tmp/l0_http_daemon.log")
                                                    .and_then(|mut f| std::io::Write::write_all(&mut f, b"Wrote request file OK\n"));
                                            }
                                            Err(e) => {
                                                let _ = fs::OpenOptions::new().append(true).open("/tmp/l0_http_daemon.log")
                                                    .and_then(|mut f| std::io::Write::write_all(&mut f, format!("Write failed: {}\n", e).as_bytes()));
                                            }
                                        }

                                        // Wait for response (up to 60 seconds)
                                        let mut responded = false;
                                        let _ = fs::OpenOptions::new().append(true).open("/tmp/l0_http_daemon.log")
                                            .and_then(|mut f| std::io::Write::write_all(&mut f, format!("Waiting for response at: {}\n", PENDING_RESPONSE_PATH).as_bytes()));

                                        for i in 0..600 {
                                            if let Ok(content) = fs::read_to_string(PENDING_RESPONSE_PATH) {
                                                let _ = fs::OpenOptions::new().append(true).open("/tmp/l0_http_daemon.log")
                                                    .and_then(|mut f| std::io::Write::write_all(&mut f, format!("Found response after {} iterations: {} bytes\n", i, content.len()).as_bytes()));
                                                let content_type = if content.starts_with('[') || content.starts_with('{') {
                                                    "application/json"
                                                } else if content.contains("<html") {
                                                    "text/html"
                                                } else {
                                                    "text/plain"
                                                };

                                                let http_response = format!(
                                                    "HTTP/1.1 200 OK\r\nContent-Type: {}; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                                    content_type, content.len(), content
                                                );

                                                let _ = stream.write_all(http_response.as_bytes());
                                                let _ = stream.flush();
                                                let _ = fs::remove_file(PENDING_RESPONSE_PATH);
                                                responded = true;
                                                break;
                                            }
                                            std::thread::sleep(Duration::from_millis(100));
                                        }

                                        if !responded {
                                            let timeout_response = "HTTP/1.1 504 Gateway Timeout\r\nContent-Length: 15\r\n\r\nRequest timeout";
                                            let _ = stream.write_all(timeout_response.as_bytes());
                                        }

                                        // Clean up request file
                                        let _ = fs::remove_file(PENDING_REQUEST_PATH);
                                    }
                                    Err(_) => {
                                        std::thread::sleep(Duration::from_millis(100));
                                    }
                                }
                            }
                        }
                        std::process::exit(0);
                    }
                    _ => {
                        // Parent - give daemon time to start
                        std::thread::sleep(Duration::from_millis(200));
                    }
                }
            }

            // Poll for pending request (up to 60 seconds)
            for _ in 0..600 {
                if let Ok(request_json) = fs::read_to_string(PENDING_REQUEST_PATH) {
                    return format!(r#"{{"ok":true,"value":{}}}"#, request_json);
                }
                std::thread::sleep(Duration::from_millis(100));
            }

            r#"{"ok":false,"error":"Timeout waiting for request"}"#.to_string()
        }

        "send" => {
            // Send response for the pending request
            let content = args.get(0).map(|s| s.as_str()).unwrap_or("");

            // Debug logging
            let cwd = std::env::current_dir().map(|p| p.display().to_string()).unwrap_or("unknown".to_string());
            let _ = fs::OpenOptions::new().create(true).append(true).open("/tmp/l0_http_daemon.log")
                .and_then(|mut f| std::io::Write::write_all(&mut f, format!("HTTP_SEND called, cwd={}, content_len={}\n", cwd, content.len()).as_bytes()));

            if content.is_empty() {
                return r#"{"ok":false,"error":"Missing response content","usage":"send content"}"#.to_string();
            }

            // Write response to file for the daemon to pick up
            let full_path = format!("{}/{}", cwd, PENDING_RESPONSE_PATH);
            let _ = fs::OpenOptions::new().create(true).append(true).open("/tmp/l0_http_daemon.log")
                .and_then(|mut f| std::io::Write::write_all(&mut f, format!("Writing to: {}\n", full_path).as_bytes()));

            match fs::write(PENDING_RESPONSE_PATH, content) {
                Ok(_) => {
                    let _ = fs::OpenOptions::new().create(true).append(true).open("/tmp/l0_http_daemon.log")
                        .and_then(|mut f| std::io::Write::write_all(&mut f, b"HTTP_SEND wrote response OK\n"));
                    r#"{"ok":true,"value":"response sent"}"#.to_string()
                }
                Err(e) => {
                    let _ = fs::OpenOptions::new().create(true).append(true).open("/tmp/l0_http_daemon.log")
                        .and_then(|mut f| std::io::Write::write_all(&mut f, format!("HTTP_SEND write failed: {}\n", e).as_bytes()));
                    format!(r#"{{"ok":false,"error":"Failed to send: {}"}}"#, e)
                }
            }
        }

        "stop_dynamic" => {
            // Stop the dynamic server daemon
            let port = get_port();
            let pid_file = format!("/tmp/l0_http_daemon_{}.pid", port);
            if let Ok(pid_str) = fs::read_to_string(&pid_file) {
                if let Ok(pid) = pid_str.trim().parse::<i32>() {
                    unsafe { libc::kill(pid, libc::SIGTERM) };
                    let _ = fs::remove_file(&pid_file);
                    return r#"{"ok":true,"value":"daemon stopped"}"#.to_string();
                }
            }
            r#"{"ok":true,"value":"no daemon running"}"#.to_string()
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

    // 处理请求
    let (status, content_type, body) = route_request(method, path, &request, config, static_dir);

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status, content_type, body.len(), body
    );

    stream.write_all(response.as_bytes()).ok();
    stream.flush().ok();
}

// Thread-safe version for multi-threaded serve
fn handle_http_request_arc(stream: TcpStream, config: &Arc<HashMap<String, String>>) {
    handle_http_request(stream, config.as_ref());
}

fn route_request(method: &str, path: &str, _request: &str, config: &HashMap<String, String>,
                 static_dir: &str) -> (&'static str, &'static str, String) {

    // 1. 已注册的静态路由 (优先级最高)
    let route_key = format!("{} {}", method, path);
    if let Some(content) = config.get(&route_key) {
        let ct = guess_content_type(content);
        return ("200 OK", ct, content.clone());
    }

    // 2. 静态文件服务
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

fn get_request_body(request: &str) -> String {
    if let Some(idx) = request.find("\r\n\r\n") {
        request[idx + 4..].to_string()
    } else {
        String::new()
    }
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
        .unwrap_or_else(|_| r#"{"port":8080,"routes":{},"static_dir":""}"#.to_string());

    let mut config = HashMap::new();

    // 解析简单字段
    for key in ["port", "static_dir"] {
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

    let routes: Vec<String> = config.iter()
        .filter(|(k, _)| !["port", "static_dir"].contains(&k.as_str()))
        .map(|(k, v)| format!("\"{}\":\"{}\"", escape_json(k), escape_json(v)))
        .collect();

    let json = format!(
        r#"{{"port":{},"static_dir":"{}","routes":{{{}}}}}"#,
        port, escape_json(static_dir), routes.join(",")
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
