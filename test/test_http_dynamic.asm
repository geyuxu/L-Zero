# Test: Dynamic HTTP Server (from README)
# Tests: HTTP_INIT, HTTP_LISTEN, HTTP_SEND, TIME

# Initialize server
SETS 255, "18601"
TEXEC 0x8000, 255, 0   # HTTP_INIT

SETS 255, "Dynamic server ready on port 18601"
TEXEC 0x5000, 255, 0

# Handle exactly 1 request for testing (README uses infinite loop)
# In production: JMP RequestLoop creates infinite loop

# Wait for request
TEXEC 0x8008, 255, 0   # HTTP_LISTEN
# R0 = {"method":"GET","path":"/api","body":"...","addr":"..."}

# Build dynamic response (same as README)
SETS 10, "{\"status\":\"ok\",\"timestamp\":"
TEXEC 0x5008, 255, 11  # TIME -> R11
SCAT 14, 10, 11
SETS 15, "}"
SCAT 16, 14, 15        # R16 = full JSON response

# Send response
TEXEC 0x8009, 16, 0    # HTTP_SEND

SETS 255, "Dynamic server test complete"
TEXEC 0x5000, 255, 0
HALT
