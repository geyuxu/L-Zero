# Todo List API Server (Fixed)
# GET /api/todos - List all todos
# POST /api/todos - Create new todo

# === DB Setup ===
SETS 255, "todo.db"
TEXEC 0x9000, 255, 99               # DB_INIT (R99 for unused results)

SETS 255, "todos|id INTEGER PRIMARY KEY AUTOINCREMENT,title TEXT,done INTEGER DEFAULT 0"
TEXEC 0x9001, 255, 99               # DB_CREATE_TABLE

SETS 255, "todos|{\"title\":\"Learn L-0\"}"
TEXEC 0x9002, 255, 99               # Insert sample data
SETS 255, "todos|{\"title\":\"Build server\"}"
TEXEC 0x9002, 255, 99

# === HTTP ===
SETS 255, "9091"
TEXEC 0x8000, 255, 99               # HTTP_INIT

SETS 255, "Todo Server: http://127.0.0.1:9091"
TEXEC 0x5000, 255, 99

# === Request Loop ===
RequestLoop:
    MARK 200                        # Memory watermark

    TEXEC 0x8008, 255, 10           # HTTP_LISTEN -> R10 = request JSON

    # Extract method using JSON_PARSE
    SETS 1, "|method"
    SCAT 2, 10, 1                   # R2 = request + "|method"
    TEXEC 0x6004, 2, 3              # R3 = method ("GET" or "POST")

    # Extract path
    SETS 4, "|path"
    SCAT 5, 10, 4                   # R5 = request + "|path"
    TEXEC 0x6004, 5, 6              # R6 = path

    # Extract body (for POST)
    SETS 7, "|body"
    SCAT 8, 10, 7
    TEXEC 0x6004, 8, 9              # R9 = body

    # Route by path length: /api/todos = 11 chars, / = 1 char
    HLEN 11, 6                      # R11 = path length
    SET 12, 5
    CMP 11, 12
    BGT HandleApi                   # > 5 means /api/...
    JMP ServeHtml

HandleApi:
    # Check method by length: GET=3, POST=4
    HLEN 13, 3                      # R13 = method length
    SET 14, 3
    CMP 13, 14
    BEQ HandleGet
    JMP HandlePost

HandleGet:
    SETS 255, "todos|1=1 ORDER BY id DESC"
    TEXEC 0x9003, 255, 20           # R20 = todos JSON
    TEXEC 0x8009, 20, 99            # HTTP_SEND
    JMP RequestDone

HandlePost:
    # Extract title from body: body is JSON like {"title":"..."}
    SETS 21, "|title"
    SCAT 22, 9, 21                  # R22 = body + "|title"
    TEXEC 0x6004, 22, 23            # R23 = title value

    # Insert into DB
    SETS 24, "todos|{\"title\":\""
    SCAT 25, 24, 23
    SETS 26, "\"}"
    SCAT 27, 25, 26                 # R27 = "todos|{\"title\":\"...\"}"
    TEXEC 0x9002, 27, 28            # R28 = inserted ID

    # Build response
    SETS 29, "{\"ok\":true,\"id\":"
    ITOA 30, 28
    SCAT 31, 29, 30
    SETS 32, "}"
    SCAT 33, 31, 32                 # R33 = {"ok":true,"id":N}
    TEXEC 0x8009, 33, 99            # HTTP_SEND
    JMP RequestDone

ServeHtml:
    SETS 40, "<!DOCTYPE html><html><body><h1>Todo</h1><ul id=\"l\"></ul>"
    SETS 41, "<form onsubmit=\"a(event)\"><input id=\"t\"><button>Add</button></form><script>"
    SETS 42, "async function f(){document.getElementById('l').innerHTML="
    SETS 43, "(await(await fetch('/api/todos')).json()).map(t=>`<li>${t.title}</li>`).join('')}"
    SETS 44, "async function a(e){e.preventDefault();await fetch('/api/todos',"
    SETS 45, "{method:'POST',headers:{'Content-Type':'application/json'},"
    SETS 46, "body:JSON.stringify({title:t.value})});t.value='';f()}f()</script></body></html>"

    SCAT 50, 40, 41
    SCAT 51, 50, 42
    SCAT 52, 51, 43
    SCAT 53, 52, 44
    SCAT 54, 53, 45
    SCAT 55, 54, 46
    TEXEC 0x8009, 55, 99            # HTTP_SEND
    JMP RequestDone

RequestDone:
    RESET 200                       # Reset memory watermark
    JMP RequestLoop
