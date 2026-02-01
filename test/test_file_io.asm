# Test: File I/O (from README)
# Tests: FILE_WRITE, FILE_READ, FILE_EXISTS, FILE_DELETE, UPPER

# Create test input file
SETS 255, "test_input.txt|Hello from L-0 VM!"
TEXEC 0x4001, 255, 0          # FILE_WRITE

# Check if file exists using HLEN trick
# "true" has length 4, "false" has length 5
SETS 255, "test_input.txt"
TEXEC 0x4005, 255, 1          # FILE_EXISTS -> R1 = "true" or "false"
HLEN 2, 1                      # R2 = length of result
SET 3, 4                       # "true" = 4 chars
CMP 2, 3
BEQ FileExists
JMP FileNotFound

FileExists:
# Read file content
SETS 255, "test_input.txt"
TEXEC 0x4000, 255, 10         # FILE_READ -> R10 = file content

# Print original content
SETS 255, "Original:"
TEXEC 0x5000, 255, 0
TEXEC 0x5000, 10, 0

# Transform: convert to uppercase
TEXEC 0x500C, 10, 11          # UPPER -> R11

# Print transformed content
SETS 255, "Transformed:"
TEXEC 0x5000, 255, 0
TEXEC 0x5000, 11, 0

# Write to output file
SETS 12, "test_output.txt|"
SCAT 13, 12, 11               # "test_output.txt|CONTENT"
TEXEC 0x4001, 13, 0           # FILE_WRITE

# Verify output file
SETS 255, "test_output.txt"
TEXEC 0x4000, 255, 20         # FILE_READ -> R20
SETS 255, "Output file content:"
TEXEC 0x5000, 255, 0
TEXEC 0x5000, 20, 0

# Cleanup: delete test files
SETS 255, "test_input.txt"
TEXEC 0x4003, 255, 0          # FILE_DELETE
SETS 255, "test_output.txt"
TEXEC 0x4003, 255, 0          # FILE_DELETE

SETS 255, "File I/O test complete"
TEXEC 0x5000, 255, 0
HALT

FileNotFound:
SETS 255, "Error: test file not found"
TEXEC 0x5000, 255, 0
HALT
