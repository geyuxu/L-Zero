# Test TEXEC result handling

SETS 255, "9094"
TEXEC 0x8000, 255, 0

SETS 255, "Server on 9094"
TEXEC 0x5000, 255, 0

# Wait for request
TEXEC 0x8008, 255, 10               # R10 = result

SETS 255, "=== Result in R10 ==="
TEXEC 0x5000, 255, 0
TEXEC 0x5000, 10, 0

HLEN 11, 10
SETS 255, "Result length:"
TEXEC 0x5000, 255, 0
ITOA 12, 11
TEXEC 0x5000, 12, 0

# Send response
SETS 255, "OK"
TEXEC 0x8009, 255, 0

HALT
