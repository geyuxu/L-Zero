# Debug todo server - Fixed register usage
SETS 255, "todo.db"
TEXEC 0x9000, 255, 99               # Use R99 for unused results

SETS 255, "todos|id INTEGER PRIMARY KEY AUTOINCREMENT,title TEXT,done INTEGER DEFAULT 0"
TEXEC 0x9001, 255, 99

SETS 255, "todos|{\"title\":\"Learn L-0\"}"
TEXEC 0x9002, 255, 99

SETS 255, "9093"
TEXEC 0x8000, 255, 99

SETS 255, "Debug Todo Server on 9093"
TEXEC 0x5000, 255, 99

RequestLoop:
    TEXEC 0x8008, 255, 10               # R10 = request JSON

    SETS 255, "=== Raw request ==="
    TEXEC 0x5000, 255, 99
    TEXEC 0x5000, 10, 99                # Print request

    HLEN 11, 10
    SETS 255, "Request length:"
    TEXEC 0x5000, 255, 99
    ITOA 12, 11
    TEXEC 0x5000, 12, 99

    # Get method using JSON_PARSE (json|field format)
    SETS 1, "|method"
    SCAT 2, 10, 1                       # R2 = request + "|method"
    TEXEC 0x6004, 2, 3                  # R3 = method value (uses JSON_PARSE)

    SETS 255, "Method:"
    TEXEC 0x5000, 255, 99
    TEXEC 0x5000, 3, 99

    HLEN 4, 3                           # R4 = method length
    SETS 255, "Method length:"
    TEXEC 0x5000, 255, 99
    ITOA 5, 4
    TEXEC 0x5000, 5, 99

    # Check if GET (length 3)
    SET 6, 3
    CMP 4, 6
    BEQ IsGet
    JMP IsPost

IsGet:
    SETS 255, "-> Routing to GET handler"
    TEXEC 0x5000, 255, 99

    SETS 255, "todos|1=1 ORDER BY id DESC"
    TEXEC 0x9003, 255, 20
    TEXEC 0x8009, 20, 99
    JMP RequestLoop

IsPost:
    SETS 255, "-> Routing to POST handler"
    TEXEC 0x5000, 255, 99

    SETS 255, "{\"ok\":true,\"method\":\"not-get\"}"
    TEXEC 0x8009, 255, 99
    JMP RequestLoop
