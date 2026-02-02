# Full test: HTTP client operations
# Verifies: HTTP_SEND (0x3001)
# Note: This test requires network access and external server

# Simple GET request (may fail without network)
# Uncomment to test:
# SETS 1, "GET|https://httpbin.org/get"
# TEXEC 0x3001, 2, 1
# SETS 255, "HTTP client test complete"
# TEXEC 0x5000, 255, 0

# For offline testing, just verify the test structure works
SETS 255, "HTTP client test skipped (requires network)"
TEXEC 0x5000, 255, 0
HALT
