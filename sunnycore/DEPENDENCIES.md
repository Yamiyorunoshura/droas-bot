# MCP servers
- context7
- sequential-thinking
- playwright

# Json example
{
  "mcpServers": {
    "context7": {
      "type": "stdio",
      "command": "npx",
      "args": [
        "-y",
        "@upstash/context7-mcp"
      ],
      "env": {
        "CONTEXT7_API_KEY": "your-context7-api-key"
      }
    },
    "sequential-thinking": {
      "type": "stdio",
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-sequential-thinking"
      ],
      "env": {}
    },
    "playwright": {
      "type": "stdio",
      "command": "npx",
      "args": [
        "@playwright/mcp@latest"
      ],
      "env": {}
    },
    "claude-context": {
      "type": "stdio",
      "command": "npx",
      "args": [
        "@zilliz/claude-context-mcp@latest"
      ],
      "env": {
        "EMBEDDING_PROVIDER": "OpenAI",
        "EMBEDDING_MODEL": "Qwen/Qwen3-Embedding-8B",
        "OPENAI_API_KEY": "your-openai-api-key",
        "OPENAI_BASE_URL": "your-openai-base-url",
        "MILVUS_TOKEN": "your-milvus-token"
      }
    }
  }
}