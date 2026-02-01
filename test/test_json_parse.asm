# Test JSON_PARSE for body parsing

# Test 1: Direct parse of body-like JSON
SETS 1, "{\"title\":\"Test Body\"}|title"
TEXEC 0x6004, 1, 2
SETS 255, "Test 1 (direct): "
TEXEC 0x5000, 255, 99
TEXEC 0x5000, 2, 99

# Test 2: Simulate the todo_server flow
SETS 10, "{\"addr\":\"127.0.0.1\",\"body\":\"{\\\"title\\\":\\\"Hello\\\"}\",\"method\":\"POST\",\"path\":\"/api/todos\"}"
SETS 255, "Full request:"
TEXEC 0x5000, 255, 99
TEXEC 0x5000, 10, 99

# Extract body
SETS 11, "|body"
SCAT 12, 10, 11
TEXEC 0x6004, 12, 13
SETS 255, "Extracted body:"
TEXEC 0x5000, 255, 99
TEXEC 0x5000, 13, 99

# Extract title from body
SETS 14, "|title"
SCAT 15, 13, 14
TEXEC 0x6004, 15, 16
SETS 255, "Extracted title:"
TEXEC 0x5000, 255, 99
TEXEC 0x5000, 16, 99

HALT
