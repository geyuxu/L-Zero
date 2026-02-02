# Smoke test: Hello World
# Verifies: SETS, TEXEC PRINT (0x5000)
SETS 255, "Hello, L-Zero!"
TEXEC 0x5000, 255, 0
HALT
