# DROAS Bot 開發指南

本文檔提供DROAS Bot項目的詳細開發指南，包括環境設置、開發工作流程和最佳實踐。

## 開發環境設置

### 1. 系統要求

- **Rust**: 1.70 或更高版本
- **PostgreSQL**: 13+ (可選，可使用SQLite進行本地開發)
- **Git**: 版本控制
- **IDE**: 推薦使用VS Code配合rust-analyzer擴展

### 2. Rust工具鏈安裝

```bash
# 安裝Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 添加必要組件
rustup component add rustfmt clippy

# 安裝useful cargo擴展
cargo install cargo-watch cargo-audit cargo-expand
```

### 3. 項目設置

```bash
# 克隆項目
git clone https://github.com/your-org/droas-bot.git
cd droas-bot

# 複製環境配置
cp .env.example .env

# 安裝依賴
cargo build
```

### 4. 環境變數配置

編輯 `.env` 文件：

```env
# Discord配置 (開發時可使用測試機器人)
DISCORD_BOT_TOKEN=your_dev_bot_token
DISCORD_APPLICATION_ID=your_dev_application_id

# 數據庫配置 (開發時推薦使用SQLite)
DATABASE_URL=sqlite://dev.db
DATABASE_MAX_CONNECTIONS=5
DATABASE_MIN_CONNECTIONS=1

# 應用配置
LOG_LEVEL=debug
IMAGE_CACHE_DIR=./dev_assets/cache
MAX_IMAGE_SIZE_MB=5
```

## 開發工作流程

### 1. 分支策略

```bash
# 主分支
main        # 生產就緒代碼
develop     # 開發分支

# 功能分支
feature/功能名稱    # 新功能開發
bugfix/錯誤名稱     # 錯誤修復
hotfix/緊急修復名稱 # 緊急修復
```

### 2. 開發流程

1. **創建功能分支**
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/your-feature-name
   ```

2. **開發和測試**
   ```bash
   # 監視模式開發 (自動重新構建和測試)
   cargo watch -x check -x test -x run
   
   # 運行特定測試
   cargo test test_name
   
   # 格式化代碼
   cargo fmt
   
   # 運行clippy檢查
   cargo clippy
   ```

3. **提交代碼**
   ```bash
   # 確保所有檢查通過
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   
   # 提交
   git add .
   git commit -m "feat: add welcome image generation"
   git push origin feature/your-feature-name
   ```

4. **創建Pull Request**
   - 在GitHub上創建PR從feature分支到develop分支
   - 確保CI檢查通過
   - 請求代碼審查

### 3. 測試驅動開發(TDD)

項目採用TDD方法，開發流程：

1. **Red**: 寫一個失敗的測試
2. **Green**: 寫最小可行代碼讓測試通過
3. **Refactor**: 重構代碼提高品質

示例：
```rust
// 1. 先寫測試 (tests/welcome_tests.rs)
#[tokio::test]
async fn test_welcome_message_generation() {
    let handler = WelcomeHandler::new();
    let result = handler.generate_message(123, 456).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("歡迎"));
}

// 2. 運行測試（應該失敗）
// cargo test test_welcome_message_generation

// 3. 實現最小可行代碼讓測試通過
impl WelcomeHandler {
    pub async fn generate_message(&self, guild_id: u64, user_id: u64) -> Result<String> {
        Ok("歡迎新成員！".to_string()) // 最小實現
    }
}

// 4. 重構並添加更多功能
```

### 4. 代碼品質檢查

所有代碼必須通過以下檢查：

```bash
# 格式化檢查
cargo fmt --check

# Clippy檢查
cargo clippy --all-targets --all-features -- -D warnings

# 測試覆蓋率
cargo test

# 安全審計
cargo audit

# 完整CI檢查
./.github/scripts/ci-local.sh  # 如果存在
```

## 項目架構

### 模組結構

```
src/
├── main.rs              # 應用入口
├── config.rs            # 配置管理
├── lib.rs              # 庫入口（如果需要）
├── handlers/           # 事件處理器
│   ├── mod.rs
│   ├── welcome.rs      # 歡迎事件
│   └── admin.rs        # 管理命令
├── services/           # 業務邏輯服務
│   ├── mod.rs
│   ├── image.rs        # 圖像生成
│   └── database.rs     # 數據庫操作
└── utils/              # 工具函數
    ├── mod.rs
    └── error.rs        # 錯誤處理
```

### 命名規範

- **文件名**: snake_case (例如: `welcome_handler.rs`)
- **模組名**: snake_case (例如: `image_service`)
- **結構體**: PascalCase (例如: `WelcomeHandler`)
- **函數**: snake_case (例如: `generate_welcome_image`)
- **常量**: SCREAMING_SNAKE_CASE (例如: `MAX_IMAGE_SIZE`)
- **環境變數**: SCREAMING_SNAKE_CASE (例如: `DISCORD_BOT_TOKEN`)

## 調試和測試

### 1. 本地調試

```bash
# 設置調試級別日誌
export LOG_LEVEL=debug

# 運行機器人
cargo run

# 使用調試器 (VS Code)
# 在.vscode/launch.json中配置調試設置
```

### 2. 測試策略

- **單元測試**: 測試個別函數和方法
- **集成測試**: 測試模組間交互
- **端到端測試**: 測試完整用戶場景

```bash
# 運行所有測試
cargo test

# 運行特定測試文件
cargo test --test integration_tests

# 運行帶輸出的測試
cargo test -- --nocapture

# 運行單個測試
cargo test test_config_load -- --exact
```

### 3. 性能測試

```bash
# 基準測試
cargo bench

# 性能分析
cargo flamegraph --bin droas-bot  # 需要安裝cargo-flamegraph
```

## 部署和發布

### 1. 本地構建

```bash
# 開發構建
cargo build

# 發布構建（優化）
cargo build --release
```

### 2. Docker構建

```bash
# 構建Docker鏡像
docker build -t droas-bot:latest .

# 運行容器
docker run --env-file .env droas-bot:latest
```

### 3. 版本發布

```bash
# 更新版本號 (Cargo.toml)
# 創建發布標籤
git tag v1.0.0
git push origin v1.0.0
```

## 故障排除

### 常見問題

1. **編譯錯誤**: 檢查Rust版本和依賴版本
2. **數據庫連接失敗**: 驗證DATABASE_URL和數據庫服務狀態
3. **Discord API錯誤**: 檢查bot token和權限設置

### 日誌分析

```bash
# 查看詳細日誌
RUST_LOG=debug cargo run

# 結構化日誌輸出
RUST_LOG=droas_bot=trace cargo run
```

## 貢獻指南

1. 遵循現有代碼風格和命名規範
2. 為新功能添加測試
3. 更新相關文檔
4. 確保CI檢查通過
5. 請求代碼審查

## 資源連結

- [Rust官方文檔](https://doc.rust-lang.org/)
- [Serenity文檔](https://docs.rs/serenity/)
- [SQLx文檔](https://docs.rs/sqlx/)
- [Discord開發者文檔](https://discord.com/developers/docs)