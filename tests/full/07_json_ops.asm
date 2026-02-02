# Full test: JSON operations
# Verifies: JSON_PARSE (0x6004)
# Note: JSON_SET/JSON_GET (memory HashMap) skipped due to tool behavior
# TEXEC signature: TEXEC tool, arg, dest

# JSON_PARSE: parse JSON string and extract field
SETS 10, "{\"name\":\"Bob\",\"age\":30}"
SETS 11, "|name"
SCAT 12, 10, 11           # "json_str|field"
TEXEC 0x6004, 12, 13      # JSON_PARSE: arg=r12, dest=r13
SETS 14, "Bob"
SCMP 13, 14
BEQ json_parse_name_ok
PANIC "JSON_PARSE name extraction failed"
json_parse_name_ok:

# Extract another field
SETS 20, "{\"status\":\"ok\",\"count\":42}"
SETS 21, "|count"
SCAT 22, 20, 21
TEXEC 0x6004, 22, 23      # JSON_PARSE: arg=r22, dest=r23
SETS 24, "42"
SCMP 23, 24
BEQ json_parse_count_ok
PANIC "JSON_PARSE count extraction failed"
json_parse_count_ok:

# Test nested JSON
SETS 30, "{\"user\":{\"id\":123}}"
SETS 31, "|user"
SCAT 32, 30, 31
TEXEC 0x6004, 32, 33      # Extract user object
HLEN 34, 33
SET 35, 0
CMP 34, 35
BGT nested_ok
PANIC "JSON_PARSE nested extraction failed"
nested_ok:

SETS 255, "JSON operations tests passed"
TEXEC 0x5000, 255, 0
HALT
