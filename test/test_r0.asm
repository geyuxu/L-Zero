# Test R0 as TEXEC destination

SETS 255, "9095"
TEXEC 0x8000, 255, 0                # R0 will be overwritten

SETS 1, "Server on 9095"
TEXEC 0x5000, 1, 2

# Wait for request, result to R0
TEXEC 0x8008, 255, 0

SETS 1, "=== R0 after HTTP_LISTEN ==="
TEXEC 0x5000, 1, 2
TEXEC 0x5000, 0, 2                  # Print R0

HLEN 1, 0
SETS 2, "R0 length: "
TEXEC 0x5000, 2, 3
ITOA 3, 1
TEXEC 0x5000, 3, 4

# Send response
SETS 1, "OK"
TEXEC 0x8009, 1, 2

HALT
