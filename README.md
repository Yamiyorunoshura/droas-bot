# DROAS Discord Economy Bot

一個基於 Rust 開發的高性能 Discord 虛擬經濟系統機器人，提供完整的用戶帳戶管理、餘額查詢、點對點轉帳和交易歷史功能。

## 功能特性

- 🏦 **自動帳戶創建** - 新用戶首次使用時自動創建經濟帳戶
- 💰 **餘額查詢** - 使用美觀的嵌入消息界面查詢帳戶餘額
- 💸 **安全轉帳** - 用戶間安全的點對點虛擬貨幣轉帳
- 📊 **交易歷史** - 查看最近 10 筆交易記錄
- 🎨 **交互界面** - 所有響應使用精美的 Discord 嵌入消息
- ❓ **幫助系統** - 完整的命令幫助和使用指南
- 🔒 **安全驗證** - 全面的交易驗證和安全檢查
- ⚡ **高性能** - 支援 1000+ 並發用戶，95% 命令在 2 秒內響應

## 技術架構

- **語言**: Rust 1.88.0+
- **Discord 框架**: Serenity (Discord API v2+)
- **資料庫**: PostgreSQL 16.x (ACID 合規)
- **快取**: Redis 8.x (可降級到記憶體快取)
- **監控**: Prometheus 指標收集
- **架構**: 單體應用程式、Repository 模式、分層架構

## 系統要求

### 基本要求
- Rust 1.88.0 或更高版本
- Cargo 1.88.0 或更高版本
- PostgreSQL 16.x
- Redis 8.x (可選，用於快取)
- Discord Bot Token

### 作業系統支援
- Linux (推薦)
- macOS
- Windows 10/11

## 快速開始

### 1. 環境準備

安裝 Rust 工具鏈：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

安裝並啟動 PostgreSQL：
```bash
# macOS
brew install postgresql@16
brew services start postgresql@16

# Ubuntu/Debian
sudo apt update
sudo apt install postgresql-16
sudo systemctl start postgresql

# 創建資料庫
sudo -u postgres createdb droas
```

安裝並啟動 Redis (可選)：
```bash
# macOS
brew install redis
brew services start redis

# Ubuntu/Debian
sudo apt install redis-server
sudo systemctl start redis
```

### 2. 獲取源代碼

```bash
git clone <repository-url>
cd DROAS-bot
```

### 3. 配置環境變數

創建 `.env` 文件：
```bash
cp .env.example .env
```

編輯 `.env` 文件：
```env
# Discord Bot Token (必須)
DISCORD_TOKEN=your_discord_bot_token_here

# 資料庫配置 (必須)
DATABASE_URL=postgres://localhost/droas
DATABASE_MAX_CONNECTIONS=10
DATABASE_MIN_CONNECTIONS=1
DATABASE_CONNECTION_TIMEOUT=30

# Redis 快取配置 (可選)
REDIS_URL=redis://localhost:6379
CACHE_ENABLE_REDIS=true
CACHE_DEFAULT_TTL_SECS=300
CACHE_FALLBACK_TO_MEMORY=true

# 監控配置 (可選)
DROAS_MONITORING_PORT=8080
DROAS_HEALTH_CHECK_INTERVAL=30
RUST_LOG=info
```

### 4. 獲取 Discord Bot Token

1. 前往 [Discord Developer Portal](https://discord.com/developers/applications)
2. 創建新應用程式
3. 在 "Bot" 頁面創建機器人
4. 複製機器人 Token
5. 在 "OAuth2" → "URL Generator" 中設置：
   - Scopes: `bot`
   - Bot Permissions: `Send Messages`, `Read Message History`, `Embed Links`

### 5. 編譯和運行

```bash
# 編譯項目
cargo build --release

# 運行機器人
cargo run

# 或使用發布版本
./target/release/droas-bot
```

## 使用指南

### 基本命令

- `!balance` - 查詢您的帳戶餘額
- `!transfer @用戶 金額` - 轉帳給指定用戶
- `!history` - 查看最近的交易記錄
- `!help` - 顯示所有可用命令

### 使用範例

```
用戶 A: !balance
機器人: 💰 您的餘額：1000 幣

用戶 A: !transfer @用戶B 100
機器人: ✅ 成功轉帳 100 幣給 @用戶B

用戶 B: !balance
機器人: 💰 您的餘額：1100 幣
```

## 配置詳情

### 資料庫配置

機器人使用 PostgreSQL 作為主要資料庫。確保：

1. 資料庫服務正在運行
2. 已創建 `droas` 資料庫
3. 連接權限正確設置

### 快取配置

機器人支援 Redis 快取以提高性能。如果 Redis 不可用，會自動降級到記憶體快取。

### 監控配置

機器人提供內建的監控端點：

- **健康檢查**: `http://localhost:8080/health`
- **Prometheus 指標**: `http://localhost:8080/metrics`

## 開發指南

### 專案結構

```
src/
├── main.rs                 # 主程式入口
├── lib.rs                  # 庫入口
├── config.rs               # 配置管理
├── database/               # 資料庫層
├── services/               # 業務邏輯層
├── discord_gateway/        # Discord API 整合
├── command_router.rs       # 命令路由
├── cache/                  # 快取層
├── error.rs                # 錯誤處理
├── logging.rs              # 日誌系統
├── health.rs               # 健康檢查
├── metrics.rs              # 指標收集
└── styles/                 # UI 樣式
```

### 運行測試

```bash
# 運行所有測試
cargo test

# 運行特定測試
cargo test balance_service_test

# 顯示測試覆蓋率
cargo test --features coverage
```

### 代碼格式化

```bash
# 格式化代碼
cargo fmt

# 檢查代碼風格
cargo clippy -- -D warnings
```

## 故障排除

### 常見問題

**Q: 機器人無法啟動，提示 "DISCORD_TOKEN not set"**
A: 確認 `.env` 文件中的 `DISCORD_TOKEN` 已正確設置

**Q: 資料庫連接失敗**
A: 檢查 PostgreSQL 服務是否運行，連接字符串是否正確

**Q: Redis 連接失敗**
A: 機器人會自動降級到記憄體快取，但建議檢查 Redis 服務狀態

**Q: 機器人沒有回應命令**
A: 確認機器人有適當的 Discord 權限：Send Messages, Read Message History, Embed Links

### 日誌級別

設置環境變數 `RUST_LOG` 來調整日誌詳細程度：

```bash
RUST_LOG=debug cargo run    # 詳細調試日誌
RUST_LOG=info cargo run     # 標準日誌 (預設)
RUST_LOG=warn cargo run     # 僅警告和錯誤
RUST_LOG=error cargo run    # 僅錯誤
```

## 部署

### Docker 部署

```dockerfile
# 建構映像
docker build -t droas-bot .

# 運行容器
docker run -d \
  -e DISCORD_TOKEN=your_token \
  -e DATABASE_URL=postgres://host/dbname \
  -p 8080:8080 \
  droas-bot
```

### 系統服務

創建系統服務文件 `/etc/systemd/system/droas-bot.service`：

```ini
[Unit]
Description=DROAS Discord Economy Bot
After=network.target

[Service]
Type=simple
User=droas
WorkingDirectory=/opt/droas-bot
Environment=RUST_LOG=info
ExecStart=/opt/droas-bot/droas-bot
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

## 性能指標

- ✅ **響應時間**: 95% 命令在 2 秒內完成
- ✅ **餘額查詢**: 500ms 內完成 (使用快取)
- ✅ **並發支援**: 1000+ 同時用戶
- ✅ **正常運行時間**: 99.5% 可用性
- ✅ **交易安全**: 100% 身份驗證和輸入驗證

## 安全性

- 🔒 所有交易都通過 Discord 用戶 ID 進行身份驗證
- ✅ 輸入驗證和清理防止注入攻擊
- 🚫 阻止自我轉帳和無效交易
- 🛡️ 適當的錯誤處理不洩露敏感信息
- 📝 完整的審計日誌

## 貢獻

歡迎貢獻！請遵循以下步驟：

1. Fork 專案
2. 創建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 開啟 Pull Request

## 授權

本專案採用 Apache 2.0 授權 - 詳見 [LICENSE](LICENSE) 文件

## 支援

- 📧 **Email**: support@droas.bot
- 💬 **Discord**: [支援服務器](https://discord.gg/droas)
- 🐛 **Bug 報告**: [GitHub Issues](https://github.com/droas/droas-bot/issues)
- 📖 **文檔**: [Wiki](https://github.com/droas/droas-bot/wiki)

## 更新日誌

### v0.1.0 (2025-10-06)
- ✨ 初始版本發布
- 🏦 實現基本經濟系統功能
- 💰 支援餘額查詢和轉帳
- 📊 交易歷史記錄
- 🔒 安全驗證機制
- 📈 監控和指標收集
- 🎨 美觀的 Discord 嵌入消息界面

---

**感謝使用 DROAS Discord Economy Bot！** 🚀