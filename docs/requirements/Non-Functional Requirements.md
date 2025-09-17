## Non-Functional Requirements

```yaml
non-functional-requirements:
  - id: "NFR-P-001"
    type: "performance"
    description: "Welcome image rendering latency"
    metric: "p95-latency-ms"
    target-value: "<=1000"
    test-method: "load-test: 20 concurrent previews for 5 minutes"
    risk-level: "Medium"
    mitigation-approach: "Cache fonts, reuse buffers, prefetch avatars when feasible"

  - id: "NFR-P-002"
    type: "performance"
    description: "Message post time"
    metric: "p95-latency-ms"
    target-value: "<=3000"
    test-method: "integration-test against Discord sandbox guild"
    risk-level: "Medium"
    mitigation-approach: "Asynchronous posting and rate-limit aware client"

  - id: "NFR-S-001"
    type: "security"
    description: "Bot token handling"
    compliance-standard: "Secret management best practices"
    risk-level: "High"
    mitigation-approach: "Read token from environment variable; never log secrets"
    test-method: "static analysis and config review"

  - id: "NFR-R-001"
    type: "reliability"
    description: "Operational availability"
    availability-target: "99.5%"
    recovery-time: "15m"
    recovery-point: "5m"
    backup-frequency: "daily"
    test-method: "chaos test: inject network failures and verify recovery"

  - id: "NFR-U-001"
    type: "usability"
    description: "Image readability"
    user-experience-goal: "Username and greeting text are readable on typical mobile and desktop themes"
    accessibility-level: "WCAG 2.1 AA"
    device-compatibility: ["desktop", "mobile"]
    test-method: "visual checks against light/dark backgrounds"

  - id: "NFR-SC-001"
    type: "scalability"
    description: "Target scale"
    capacity-requirement: "Up to 100 guilds and 30 join events/min"
    growth-projection: "2x in 6 months"
    test-method: "event replay harness"
```

