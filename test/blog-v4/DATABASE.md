# Blog Database Schema

## Posts Table

```sql
CREATE TABLE posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    content TEXT,
    author TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

## Sample Data

```sql
INSERT INTO posts (title, content, author) VALUES
    ('Welcome to L-0 Blog', 'This is a sample post demonstrating the L-0 VM blog system with SQLite database.', 'Admin'),
    ('Getting Started with L-0', 'L-0 is a custom virtual machine designed for AI agents. It uses assembly-like syntax.', 'Developer'),
    ('Database Operations', 'L-0 supports SQLite with full CRUD: Create Read Update Delete operations.', 'Admin');
```

## Notes

- Database file: `blog.db`
- Created automatically by `blog.asm` on first run via `DB_CREATE_TABLE` (0x9001)
- Sample posts inserted via `DB_INSERT` (0x9002)
