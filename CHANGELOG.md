# Changelog

## v0.0.2-preview (2026-02-02)

### New Instructions
- `SCMP` - String content comparison (sets flags like CMP but compares heap strings)
- `NEWR` - Dynamic-size heap allocation from register value

### Bug Fixes
- Fixed: String routing in REST API examples (CMP → SCMP)
- Fixed: AOT HLEN inconsistency (now matches VM behavior)
- Fixed: AOT UPPER/LOWER missing null terminator
- Fixed: GAS instruction AOT value (1000000 → 1000)
- Fixed: FILE_EXISTS documentation (returns "true"/"false")
- Fixed: JSON field extraction pattern (use JSON_PARSE 0x6004 with pipe format)
- Fixed: README instruction count unified to 56
- Fixed: stdlib patterns (sb_init, sb_append, sb_finish, array_new) use NEWR

### Documentation
- Updated ISA tables to include SCMP and NEWR
- Fixed Routing summary table (CMP → SCMP)
- Fixed JSON tool descriptions (JSON_GET vs JSON_PARSE)
- Added smoke tests: hello.asm, test_scmp.asm, test_newr.asm

### Known Issues
- Plugin processes spawned per TEXEC call (performance)
- HTTP dynamic mode uses file-based IPC
- See roadmap for planned improvements

## v0.0.1 (initial)
- 54-instruction ISA
- VM interpreter + AOT compiler
- 4 plugins: file, data, http, db
- Governance layer (LATCH/GUARD/TRAP/YIELD)
