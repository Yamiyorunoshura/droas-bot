## Deployment architecture

### Infrastructure design
- environment-strategy:
  - development: local run via cargo run
  - staging: optional dockerized bot in a sandbox guild
  - production: single instance on a small host or container
- containerization:
  - strategy: optional Docker image; keep binary small
  - scaling: manual scale-out if needed
- ci-cd-pipeline:
  - version-control: git (feature branches)
  - testing: unit and integration tests in CI
  - deployment: manual trigger (MVP); automate later
- monitoring-and-logging:
  - application-monitoring: basic metrics/logs (future enhancement)
  - infrastructure-monitoring: host-level metrics (if applicable)
  - log-aggregation: optional future centralization

