## Non-functional requirements architecture

### Performance
- response-time: <= 3000ms for posting on member join (P95)
- throughput: up to 30 join events/min across ~100 guilds (MVP target)
- scalability-strategy:
  - non-blocking IO and bounded retries
  - cache fonts and reuse buffers
  - prefetch/cached avatars where feasible
- optimization-techniques:
  - image pipeline reuse and buffer pooling
  - minimize network calls; respect ETag/If-Modified-Since if available

### Security
- authentication:
  - method: Discord bot token from environment variable
  - password-policy: N/A
  - session-management: N/A (stateless gateway/HTTP interactions)
- authorization:
  - model: permission checks for admin-only commands (Manage Guild)
  - permissions: scoped per guild; minimal intents
- data-protection:
  - encryption-at-rest: host/disk-level (as available)
  - encryption-in-transit: TLS between bot and Discord
  - data-anonymization: avoid storing PII beyond guild IDs and config
- vulnerability-protection:
  - input-validation: strict URL/attachment validation for background
- sql-injection: parameterized queries for PostgreSQL
  - xss-protection: N/A (no web UI)
  - csrf-protection: N/A (no web UI)

### Availability
- uptime-target: 99.5% (MVP)
- disaster-recovery:
- backup-strategy: periodic backup of PostgreSQL database and assets
  - recovery-time-objective: < 4 hours
  - recovery-point-objective: < 1 hour
- monitoring:
  - health-checks: process heartbeat, reconnect-on-disconnect
  - alerting: basic logs; future: integrate simple alerts
  - logging: structured logs to stdout
- fault-tolerance:
  - redundancy: single instance (MVP); future multi-instance with queue
  - failover: process supervisor auto-restart
  - circuit-breaker: backoff and retry around Discord APIs

