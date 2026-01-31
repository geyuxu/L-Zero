# L-0 Blog Web Application v4.0
# =============================
# Full CRUD blog with external templates
# Run from project root:
#   ./target/release/l0asm examples/blog-v4/blog.asm > examples/blog-v4/blog.l0
#   ./target/release/l0vm examples/blog-v4/blog.l0

# === 1. Initialize Database ===
SETS 255, "=== L-0 Blog v2.0 ==="
TEXEC 0x5000, 255, 255

SETS 0, "examples/blog-v4/blog.db"
TEXEC 0x9000, 0, 255
SETS 255, "[DB] SQLite initialized"
TEXEC 0x5000, 255, 255

# Create posts table
SETS 0, "posts|id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, content TEXT, author TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP"
TEXEC 0x9001, 0, 255
SETS 255, "[DB] Table ready"
TEXEC 0x5000, 255, 255

# Insert sample posts if empty (using raw SQL to check)
SETS 0, "SELECT COUNT(*) as cnt FROM posts"
TEXEC 0x900A, 0, 50
# For simplicity, always insert samples (duplicates OK for demo)
SETS 0, "posts|title,content,author|Welcome to L-0 Blog,This is a sample post demonstrating the L-0 VM blog system with SQLite database.,Admin"
TEXEC 0x9002, 0, 255
SETS 0, "posts|title,content,author|Getting Started with L-0,L-0 is a custom virtual machine designed for AI agents. It uses assembly-like syntax.,Developer"
TEXEC 0x9002, 0, 255
SETS 0, "posts|title,content,author|Database Operations,L-0 supports SQLite with full CRUD: Create Read Update Delete operations.,Admin"
TEXEC 0x9002, 0, 255
SETS 255, "[DB] Sample posts ready"
TEXEC 0x5000, 255, 255

# === 2. Load Templates ===
SETS 255, "[TPL] Loading templates..."
TEXEC 0x5000, 255, 255

# Load header template
SETS 0, "examples/blog-v4/templates/header.html"
TEXEC 0x4000, 0, 10            # FILE_READ -> R10

# Load form template
SETS 0, "examples/blog-v4/templates/form_create.html"
TEXEC 0x4000, 0, 11            # -> R11

# Load posts header
SETS 0, "examples/blog-v4/templates/posts_header.html"
TEXEC 0x4000, 0, 12            # -> R12

# Load posts footer
SETS 0, "examples/blog-v4/templates/posts_footer.html"
TEXEC 0x4000, 0, 13            # -> R13

# Load footer template
SETS 0, "examples/blog-v4/templates/footer.html"
TEXEC 0x4000, 0, 14            # -> R14

SETS 255, "[TPL] Templates loaded"
TEXEC 0x5000, 255, 255

# === 3. Query Posts ===
SETS 0, "posts"
TEXEC 0x9003, 0, 20            # DB_SELECT -> R20 = posts JSON

# === 4. Build Home Page HTML ===
# Start with header
MOV 30, 10                     # R30 = header

# Add create form
SCAT 30, 30, 11

# Add posts header
SCAT 30, 30, 12

# Add posts data as hidden JSON for JavaScript to render
SETS 31, "<script type='application/json' id='posts-data'>"
SCAT 30, 30, 31
SCAT 30, 30, 20                # Posts JSON
SETS 31, "</script><div id='posts-container'></div>"
SCAT 30, 30, 31

# Add posts footer (with update/delete forms)
SCAT 30, 30, 13

# Add page footer
SCAT 30, 30, 14

# === 5. Setup HTTP Server ===
SETS 0, "8080"
TEXEC 0x8000, 0, 255
SETS 255, "[HTTP] Port 8080"
TEXEC 0x5000, 255, 255

# Set database path for CRUD operations
SETS 0, "examples/blog-v4/blog.db"
TEXEC 0x8007, 0, 255
SETS 255, "[HTTP] DB path set"
TEXEC 0x5000, 255, 255

# === 6. Write index.html and Register Routes ===

# Write the built page to index.html
SETS 0, "examples/blog-v4/index.html|"
SCAT 0, 0, 30
TEXEC 0x4001, 0, 255
SETS 255, "[HTTP] Wrote index.html"
TEXEC 0x5000, 255, 255

# Set static directory
SETS 0, "examples/blog-v4"
TEXEC 0x8006, 0, 255
SETS 255, "[HTTP] Static dir: examples/blog-v4/"
TEXEC 0x5000, 255, 255

# POST routes are handled dynamically by http_plugin
SETS 255, "[HTTP] CRUD: /create, /update, /delete (dynamic)"
TEXEC 0x5000, 255, 255

# === 7. Start Server ===
SETS 255, "================================"
TEXEC 0x5000, 255, 255
SETS 255, "http://localhost:8080"
TEXEC 0x5000, 255, 255
SETS 255, "Press Ctrl+C to stop"
TEXEC 0x5000, 255, 255
SETS 255, "================================"
TEXEC 0x5000, 255, 255

SETS 0, "1000"
TEXEC 0x8002, 0, 255           # HTTP_SERVE

HALT
