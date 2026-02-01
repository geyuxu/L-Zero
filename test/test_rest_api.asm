# Test: RESTful API Server (from README)
# Tests: Full CRUD with HTTP_LISTEN/HTTP_SEND + DB_* operations
# This is a simplified testable version of the README example

# === Database Setup ===
SETS 255, "test_rest_api.db"
TEXEC 0x9000, 255, 0                    # DB_INIT

SETS 255, "posts|id INTEGER PRIMARY KEY AUTOINCREMENT,title TEXT,content TEXT,created INTEGER"
TEXEC 0x9001, 255, 0                    # DB_CREATE_TABLE

# Insert sample data with server timestamp
TEXEC 0x5008, 255, 50                   # R50 = current timestamp
SETS 51, "posts|title,content,created|Hello World,Welcome to L-0 API,"
SCAT 52, 51, 50                         # Append timestamp
TEXEC 0x9002, 52, 0                     # DB_INSERT

# === HTTP Server ===
SETS 255, "18602"
TEXEC 0x8000, 255, 0                    # HTTP_INIT

SETS 255, "REST API server ready on port 18602"
TEXEC 0x5000, 255, 0

# === Handle Single Request (for testing) ===
# In production: use loop with MARK/RESET
MARK 200

TEXEC 0x8008, 255, 0                    # HTTP_LISTEN -> R0 = request JSON

# Extract method using JSON_GET
SETS 255, "method"
SCAT 1, 0, 255
TEXEC 0x6002, 1, 2                      # JSON_GET -> R2 = method

# Extract path
SETS 255, "path"
SCAT 3, 0, 255
TEXEC 0x6002, 3, 4                      # JSON_GET -> R4 = path

# For this test: always return posts list (simulating GET /api/posts)
# Real implementation would route based on method and path

# Query database
SETS 255, "posts"
TEXEC 0x9003, 255, 40                   # DB_SELECT -> R40 = all posts JSON

# Send response
TEXEC 0x8009, 40, 0                     # HTTP_SEND

RESET 200

SETS 255, "REST API test complete"
TEXEC 0x5000, 255, 0

# Cleanup: delete test database
SETS 255, "test_rest_api.db"
TEXEC 0x4003, 255, 0                    # FILE_DELETE

HALT
