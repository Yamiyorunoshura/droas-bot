## Database design

### Primary database
- type: relational (PostgreSQL)
- justification: managed, scalable relational store; strong concurrency and reliability

### Caching strategy
- levels: in-memory caches for fonts and recent avatars
- ttl-policies: time-based expiry (e.g., minutes)

### Data partitioning
- strategy: none for MVP
- criteria: N/A

