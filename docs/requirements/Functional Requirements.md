## Functional Requirements

```yaml
functional-requirements:
  - id: "F-001"
    title: "Post welcome message on member join"
    description: "When a new member joins a guild, the bot posts a welcome message to a configured channel."
    priority: "High"
    user-story: "As a server admin, I want an automatic welcome message so new members feel acknowledged."
    acceptance-criteria:
      - "Given a configured welcome channel, when a member joins, then a welcome message is posted within 3 seconds"
      - "If no welcome channel is configured, then log a warning and skip posting"
      - "Message includes the member's mention and brief greeting text"
    business-rules: []
    dependencies: []
    effort-estimate: "M"
    notes: "Channel configuration handled per guild"

  - id: "F-002"
    title: "Render welcome image"
    description: "Generate an image that includes the member's avatar, username, and a guild-specific background image."
    priority: "High"
    user-story: "As a server admin, I want a personalized image so the welcome feels branded."
    acceptance-criteria:
      - "Image resolution is 1024x512 pixels (landscape)"
      - "Avatar is fetched and rendered as a circle with anti-aliased edges"
      - "Username is rendered with a readable font and sufficient contrast"
      - "If avatar is unavailable, use a generic placeholder silhouette"
      - "If background is not configured, use a default background"
    business-rules: []
    dependencies: []
    effort-estimate: "M"
    notes: "Ensure font licensing permits redistribution"

  - id: "F-003"
    title: "Admin command to set background"
    description: "Provide a command restricted to administrators to set or update the per-guild welcome background image by attachment or URL."
    priority: "High"
    user-story: "As an admin, I want to set the welcome image background so we can reflect our server branding."
    acceptance-criteria:
      - "Command available to users with Manage Guild permission"
      - "Supports image via message attachment or HTTPS URL"
      - "Validates file type (PNG or JPEG) and max size (<= 5 MB)"
      - "Persists the background per guild and confirms on success"
      - "On invalid input, returns a helpful error message"
    business-rules:
      - "Only one active background per guild"
    dependencies: []
    effort-estimate: "M"
    notes: "Store config in a persistent store; consider file naming by guild ID"

  - id: "F-004"
    title: "Preview welcome image"
    description: "Allow admins to preview the rendered welcome image without requiring an actual member join."
    priority: "Medium"
    user-story: "As an admin, I want to preview the welcome image so I can fine-tune the background."
    acceptance-criteria:
      - "Command generates an image using the invoker's avatar and username"
      - "Returns the image as an attachment within 3 seconds on P95"
      - "Errors are logged and messaged with a short explanation"
    business-rules: []
    dependencies: ["F-002", "F-003"]
    effort-estimate: "S"
    notes: "Reuse the same rendering pipeline"

  - id: "F-005"
    title: "Per-guild configuration management"
    description: "Persist and load per-guild settings including welcome channel and background image reference."
    priority: "High"
    user-story: "As an admin, I want per-guild settings so the bot behaves consistently across restarts."
    acceptance-criteria:
      - "Settings are keyed by guild ID"
      - "On startup, the bot loads settings before processing events"
      - "Changes are durable and survive restarts"
    business-rules: []
    dependencies: []
    effort-estimate: "M"
    notes: "Choose a lightweight store first (e.g., file or SQLite)"

  - id: "F-006"
    title: "Rate limit and error handling"
    description: "The bot respects Discord rate limits and handles transient failures with retries."
    priority: "High"
    user-story: "As a maintainer, I want resilience so the bot remains reliable under load."
    acceptance-criteria:
      - "HTTP 429 responses trigger backoff and retry respecting retry-after"
      - "Network errors retry up to 3 times with exponential backoff"
      - "Duplicate join events within 5 minutes do not post duplicate messages"
    business-rules: []
    dependencies: []
    effort-estimate: "M"
    notes: "Idempotency via member+timestamp window"
```

