# Test: Static HTTP Server (from README)
# Tests: HTTP_INIT, HTTP_ROUTE, HTTP_SERVE_ONCE

# Initialize server on test port
SETS 255, "18600"
TEXEC 0x8000, 255, 0   # HTTP_INIT

# Register route (same as README example)
SETS 255, "GET /|<html><body>Hello World</body></html>"
TEXEC 0x8001, 255, 0   # HTTP_ROUTE

# Register JSON API route
SETS 255, "GET /api/status|{\"ok\":true}"
TEXEC 0x8001, 255, 0   # HTTP_ROUTE

# Print ready message
SETS 255, "HTTP server ready on port 18600"
TEXEC 0x5000, 255, 0

# Serve one request (for testing, README uses 10)
SETS 255, "1"
TEXEC 0x8002, 255, 0   # HTTP_SERVE

SETS 255, "Static server test complete"
TEXEC 0x5000, 255, 0
HALT
