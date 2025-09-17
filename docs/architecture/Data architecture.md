## Data architecture

### Data model
- entities:
  - guild_config: guild_id, welcome_channel_id, background_ref, updated_at
  - background_asset: file_path or URL, media_type, created_at
- relationships:
  - guild_config (1) -> background_asset (1)
- data-flow:
  - ingestion: join-event -> load guild_config -> fetch avatar/background
  - processing: compose image -> generate bytes -> attach to message
  - output: send message + image -> log result

