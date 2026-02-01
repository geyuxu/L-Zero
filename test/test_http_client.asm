# Test: HTTP Client (from README)
# Tests: HTTP_REQUEST (GET and POST)
# Note: HTTP_REQUEST only supports HTTP, not HTTPS

SETS 255, "Testing HTTP Client..."
TEXEC 0x5000, 255, 0

# GET request (use http, not https - plugin doesn't support TLS)
SETS 1, "http://httpbin.org/get"
TEXEC 0x8004, 1, 10           # HTTP_REQUEST -> R10 = response

# Check if we got a response (at least 10 chars)
HLEN 2, 10
SET 3, 10
CMP 2, 3
BLT GetFailed

SETS 4, "GET request succeeded"
TEXEC 0x5000, 4, 0

# POST request with body
SETS 5, "http://httpbin.org/post|POST|{\"test\":\"L0\"}"
TEXEC 0x8004, 5, 20           # HTTP_REQUEST -> R20 = response

# Check if we got a response
HLEN 6, 20
SET 7, 10
CMP 6, 7
BLT PostFailed

SETS 8, "POST request succeeded"
TEXEC 0x5000, 8, 0

SETS 255, "HTTP Client test complete"
TEXEC 0x5000, 255, 0
HALT

GetFailed:
SETS 255, "GET request failed or empty response"
TEXEC 0x5000, 255, 0
HALT

PostFailed:
SETS 255, "POST request failed or empty response"
TEXEC 0x5000, 255, 0
HALT
