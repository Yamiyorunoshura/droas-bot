## System overview

### Project context
- business-domain: community onboarding and branding within Discord guilds
- target-users: server administrators and moderators
- key-business-goals: improve onboarding experience, reinforce server identity, and reduce manual moderator workload

### Architecture principles
- scalability: event-driven processing with backoff and idempotency; lightweight horizontal scaling feasible later
- maintainability: modular services (event handling, image rendering, configuration); clean boundaries and contracts
- security: least-privilege intents/permissions; strict secret handling; never log sensitive tokens
- performance: image rendering optimized for p95 targets; async IO and backoff for API calls

