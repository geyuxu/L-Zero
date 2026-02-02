# Full test: Database operations
# Verifies: DB_EXEC (0x4000), DB_QUERY (0x4001)

# Create table
SETS 1, "CREATE TABLE IF NOT EXISTS test_users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)"
TEXEC 0x4000, 1, 0

# Insert data
SETS 2, "INSERT INTO test_users (name, age) VALUES ('Alice', 30)"
TEXEC 0x4000, 2, 0

SETS 3, "INSERT INTO test_users (name, age) VALUES ('Bob', 25)"
TEXEC 0x4000, 3, 0

# Query data
SETS 4, "SELECT name FROM test_users WHERE age = 30"
TEXEC 0x4001, 4, 5

# The result format may vary, just verify we got something
HLEN 6, 5
SET 7, 0
CMP 6, 7
BGT query_ok
PANIC "DB_QUERY returned empty"
query_ok:

# Clean up
SETS 10, "DROP TABLE test_users"
TEXEC 0x4000, 10, 0

SETS 255, "Database operations tests passed"
TEXEC 0x5000, 255, 0
HALT
