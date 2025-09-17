## Functional requirements architecture

### Authentication system
- components:
  - bot-token-management
  - permission-intents (minimal required)
  - secure-config-loader (env var)
- design-pattern: token-based app authentication (Discord bot)
- security-measures:
  - secret-from-env only; no token in source or logs
  - least-privilege intents
  - rotate tokens if compromise suspected
- data-flow:
  - process-startup -> read $DISCORD_BOT_TOKEN -> connect gateway -> handle events

### Data processing pipeline
- components:
  - join-event-ingestion
  - rendering-pipeline (avatar fetch, composition, text overlay)
  - output-delivery (message + attachment)
- design-pattern: event-driven pipeline
- scalability-approach:
  - exponential backoff on transient failures
  - rate-limit-aware client (respect retry-after)
  - optional batching for previews in the future

### API endpoints
- design-pattern: Discord application commands and gateway events (no public HTTP API)
- endpoints:
  - slash-commands: /set-background, /preview
  - events: GUILD_MEMBER_ADD
- documentation: inline command help and repository docs
- versioning-strategy: semantic versioning for bot releases

