# Full test: File I/O operations
# Verifies: FILE_WRITE (0x4001), FILE_READ (0x4000), FILE_EXISTS (0x4005)
# FILE_WRITE format: "path|content"

# Prepare write data: "path|content"
SETS 1, "/tmp/l0_test_file.txt"
SETS 2, "|"
SETS 3, "Hello from L-Zero file test!"
SCAT 4, 1, 2
SCAT 5, 4, 3              # r5 = "/tmp/l0_test_file.txt|Hello from L-Zero file test!"

# Write file
TEXEC 0x4001, 5, 0        # FILE_WRITE: arg=r5 (path|content)

# Check exists
TEXEC 0x4005, 1, 6        # FILE_EXISTS: arg=r1 (path), dest=r6
SETS 7, "true"
SCMP 6, 7
BEQ exists_ok
PANIC "FILE_EXISTS should return true"
exists_ok:

# Read file
TEXEC 0x4000, 1, 8        # FILE_READ: arg=r1 (path), dest=r8 (content)
SCMP 8, 3
BEQ read_ok
PANIC "FILE_READ content mismatch"
read_ok:

# Delete file - check if there's a FILE_DELETE tool
# For now, skip delete test since we don't have the tool ID confirmed
# Just verify the tests above passed

SETS 255, "File I/O tests passed"
TEXEC 0x5000, 255, 0
HALT
