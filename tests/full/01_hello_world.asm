# Full test: Hello World variations
# Verifies: SETS, TEXEC PRINT (0x5000), PRINTN (0x5001)

SETS 1, "Hello, World!"
TEXEC 0x5000, 1, 0

SETS 2, "Line 1"
SETS 3, "Line 2"
TEXEC 0x5001, 2, 0
TEXEC 0x5001, 3, 0

SETS 255, "\nHello World test complete"
TEXEC 0x5000, 255, 0
HALT
