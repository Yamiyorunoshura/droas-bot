# DROAS Bot - Discord Rust歡迎機器人

[![CI](https://github.com/your-org/droas-bot/workflows/CI/badge.svg)](https://github.com/your-org/droas-bot/actions)
[![Security Audit](https://github.com/your-org/droas-bot/workflows/Security%20Audit/badge.svg)](https://github.com/your-org/droas-bot/actions)

一個使用Rust開發的Discord歡迎機器人，為新成員提供個性化歡迎體驗，包括自定義歡迎圖像和消息。

## 功能特色

- 🎨 **個性化歡迎圖像**: 自動生成包含新成員頭像和用戶名的歡迎圖像
- 🛠️ **每公會配置**: 支持每個Discord伺服器獨立配置歡迎頻道和背景圖像  
- ⚡ **高性能**: 使用Rust異步處理，支持高併發場景
- 🔒 **安全可靠**: 遵循Discord API最佳實踐，包含完整的錯誤處理和重試機制
- 📊 **豐富管理**: 提供管理員命令用於配置和預覽功能

## 系統需求

- Rust 1.70+
- PostgreSQL 13+ 或 SQLite 3.35+
- Discord Bot Token

## 快速開始

### 1. 環境配置

複製環境變數範例文件：
```bash
cp .env.example .env
```

編輯 `.env` 文件，設置必要的環境變數：
```env
DISCORD_BOT_TOKEN=your_bot_token_here
DISCORD_APPLICATION_ID=your_application_id_here
DATABASE_URL=sqlite://droas_bot.db
```

### 2. 安裝依賴

```bash
cargo build
```

### 3. 運行機器人

```bash
cargo run
```

## 開發指南

### 項目結構

```
src/
├── main.rs          # 應用程式入口
├── config.rs        # 配置管理
└── handlers/        # 事件處理器
    ├── mod.rs       # 模組定義
    └── welcome.rs   # 歡迎事件處理

tests/               # 測試文件
├── integration_tests.rs
├── config_tests.rs
└── ci_tests.rs

docs/                # 項目文檔
├── requirements/    # 需求規格
├── architecture/    # 架構設計
└── implementation-plan/  # 實施計劃
```

### 代碼品質

項目使用以下工具確保代碼品質：

- **rustfmt**: 代碼格式化
- **clippy**: 代碼檢查和建議
- **cargo test**: 單元和集成測試
- **cargo audit**: 安全漏洞掃描

運行所有檢查：
```bash
# 格式化代碼
cargo fmt

# 運行clippy檢查
cargo clippy -- -D warnings

# 運行測試
cargo test

# 安全審計
cargo audit
```

### 測試

項目采用測試驅動開發(TDD)方法：

```bash
# 運行所有測試
cargo test

# 運行特定測試模組
cargo test integration_tests

# 運行特定測試
cargo test test_config_load
```

## 部署

### Docker部署 (推薦)

```bash
docker build -t droas-bot .
docker run -d --name droas-bot --env-file .env droas-bot
```

### 直接部署

```bash
# 構建發布版本
cargo build --release

# 運行
./target/release/droas-bot
```

## 配置選項

| 環境變數 | 描述 | 默認值 | 必需 |
|---------|------|--------|------|
| `DISCORD_BOT_TOKEN` | Discord Bot Token | - | ✅ |
| `DISCORD_APPLICATION_ID` | Discord Application ID | - | ✅ |
| `DATABASE_URL` | 資料庫連接URL | `sqlite://droas_bot.db` | ❌ |
| `DATABASE_MAX_CONNECTIONS` | 資料庫最大連接數 | `5` | ❌ |
| `LOG_LEVEL` | 日志級別 | `info` | ❌ |
| `IMAGE_CACHE_DIR` | 圖像緩存目錄 | `./assets/cache` | ❌ |
| `MAX_IMAGE_SIZE_MB` | 最大圖像大小(MB) | `5` | ❌ |

## 貢獻指南

1. Fork 項目
2. 創建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 開啟 Pull Request

### 代碼規範

- 遵循Rust官方代碼風格指南
- 所有public API必須包含文檔註釋
- 新功能必須包含相應測試
- 確保CI檢查通過

## 許可證

本項目使用 MIT 許可證 - 詳見 [LICENSE](LICENSE) 文件

## 支持

- 📖 [完整文檔](docs/)
- 🐛 [問題追蹤](https://github.com/your-org/droas-bot/issues)
- 💬 [討論區](https://github.com/your-org/droas-bot/discussions)

---

**注意**: 此項目目前處於開發階段，API可能會發生變化。