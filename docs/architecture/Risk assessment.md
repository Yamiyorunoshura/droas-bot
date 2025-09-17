## Risk assessment

### Technical risks
- high-risk:
  - risk: Discord rate limits cause throttling during join bursts
  - impact: delayed welcome messages
  - mitigation: rate-limit-aware client, retries, and idempotency
- medium-risk:
  - risk: image rendering latency exceeds P95 targets under load
  - impact: degraded user experience
  - mitigation: caching, buffer reuse, reduced image complexity
- low-risk:
  - risk: local storage fills with background assets
  - impact: failures to save or load backgrounds
  - mitigation: size quotas and cleanup policy

### Operational risks
- dependencies:
  - external-service: Discord API & Gateway
  - risk-level: medium
  - contingency-plan: exponential backoff, reconnect logic, and partial functionality
- scalability-concerns:
  - bottleneck: single-process event handling and rendering
  - threshold: ~30 join events/min (MVP)
  - scaling-plan: move rendering to a worker pool; add queues

