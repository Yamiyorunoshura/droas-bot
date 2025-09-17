## Technical stack

### Frontend
- framework: N/A (no end-user web UI; Discord client apps act as the presentation layer)
- language: N/A
- styling: N/A

### Backend
- framework: Rust application using serenity (Discord API v10)
- language: Rust (stable)
- runtime: native binary

### Database
- primary: PostgreSQL for per-guild configuration
- cache: in-memory caches (fonts, background metadata)
- search: N/A

### Infrastructure
- cloud-provider: none required for MVP (local or low-cost host)
- container: optional Docker image for deployment; not required for local dev
- ci-cd: GitHub Actions (lint, test, build)
- monitoring: basic logs; optional future integration with a log aggregator

### External services
- authentication: bot token (Discord)
- payment: N/A
- notifications: Discord channels via bot messages

