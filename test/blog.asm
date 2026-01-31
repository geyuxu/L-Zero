# L-0 Blog Web Application
# =========================
# Blog with SQLite + HTTP - Run: l0asm blog.asm > blog.l0 && l0vm blog.l0

# === 1. Initialize Database ===
SETS 255, "=== L-0 Blog System ==="
TEXEC 0x5000, 255, 255

SETS 0, "blog.db"
TEXEC 0x9000, 0, 255
SETS 255, "[DB] SQLite initialized"
TEXEC 0x5000, 255, 255

# Create posts table
SETS 0, "posts|id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT, content TEXT, author TEXT"
TEXEC 0x9001, 0, 255
SETS 255, "[DB] Table ready"
TEXEC 0x5000, 255, 255

# Insert sample posts
SETS 0, "posts|title,content,author|Welcome to L-0,This blog runs on L-0 VM with SQLite,Admin"
TEXEC 0x9002, 0, 255
SETS 0, "posts|title,content,author|Getting Started,L-0 uses assembly syntax for AI agents,Dev"
TEXEC 0x9002, 0, 255
SETS 0, "posts|title,content,author|Database Guide,Full CRUD with SQLite plugin,Admin"
TEXEC 0x9002, 0, 255
SETS 255, "[DB] Posts inserted"
TEXEC 0x5000, 255, 255

# === 2. Query Posts ===
SETS 0, "posts"
TEXEC 0x9003, 0, 20

# === 3. Build HTML ===
SETS 30, "<!DOCTYPE html><html><head><meta charset=UTF-8><title>L-0 Blog</title>"
SETS 31, "<style>body{font-family:system-ui;max-width:800px;margin:40px auto;padding:20px;background:#f5f5f5}"
SCAT 30, 30, 31
SETS 31, "h1{color:#1a73e8}.post{background:#fff;padding:20px;margin:15px 0;border-radius:8px;box-shadow:0 2px 4px rgba(0,0,0,.1)}"
SCAT 30, 30, 31
SETS 31, "pre{background:#e8e8e8;padding:15px;border-radius:4px;overflow:auto}</style></head><body>"
SCAT 30, 30, 31
SETS 31, "<h1>L-0 Blog</h1><p>Powered by L-0 VM + SQLite + HTTP</p>"
SCAT 30, 30, 31
SETS 31, "<div class=post><h2>Posts from Database</h2><pre>"
SCAT 30, 30, 31
SCAT 30, 30, 20
SETS 31, "</pre></div>"
SCAT 30, 30, 31
SETS 31, "<div class=post><h2>Tech Stack</h2><ul><li>L-0 VM (Rust)</li><li>SQLite Database</li><li>HTTP Server</li></ul></div>"
SCAT 30, 30, 31
SETS 31, "<footer>L-0 VM v1.0</footer></body></html>"
SCAT 30, 30, 31

# === 4. Setup HTTP ===
SETS 0, "8080"
TEXEC 0x8000, 0, 255
SETS 255, "[HTTP] Port 8080"
TEXEC 0x5000, 255, 255

# Register route with HTML content
SETS 0, "GET /|"
SCAT 0, 0, 30
TEXEC 0x8001, 0, 255
SETS 255, "[HTTP] Route GET / registered"
TEXEC 0x5000, 255, 255

# Catch-all
SETS 0, "* *|"
SCAT 0, 0, 30
TEXEC 0x8001, 0, 255

# === 5. Start Server ===
SETS 255, "================================"
TEXEC 0x5000, 255, 255
SETS 255, "http://localhost:8080"
TEXEC 0x5000, 255, 255
SETS 255, "Press Ctrl+C to stop"
TEXEC 0x5000, 255, 255
SETS 255, "================================"
TEXEC 0x5000, 255, 255

SETS 0, "100"
TEXEC 0x8002, 0, 255

HALT
