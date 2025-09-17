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
        "CONTEXT7_API_KEY": "ctx7sk-e8a45a51-402a-4420-9f7d-5f57f65ceb0c"
      }
    },
    "sequential-thinking": {
      "type": "stdio",a
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
        "OPENAI_API_KEY": "sk-ybskvxkencapuqiogkbydkklrhjisfvcnhegfznsoxlvbcoh",
        "OPENAI_BASE_URL": "https://api.siliconflow.cn/v1",
        "MILVUS_TOKEN": "b9a5f0b693757863ed655f0266b25a721642678944d38d709951954ce2dc07eaa265e1699bfaf2c68b6f7e374c4a74d02226152f"
      }
    }
  }
}
```