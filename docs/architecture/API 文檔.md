## API 文檔


### 內部 API

#### UserService API
```
create_user(user_id: u64, guild_id: u64) -> Result<User>
get_user(user_id: u64, guild_id: u64) -> Result<Option<User>>
ensure_user_exists(user_id: u64, guild_id: u64) -> Result<User>
```

#### BalanceService API
```
get_balance(user_id: u64, guild_id: u64) -> Result<Balance>
update_balance(user_id: u64, guild_id: u64, amount: i64) -> Result<Balance>
transfer_balance(from_user: u64, to_user: u64, guild_id: u64, amount: u64) -> Result<Transaction>
```

#### TransferService API
```
execute_transfer(request: TransferRequest) -> Result<Transaction>
validate_transfer(request: &TransferRequest) -> Result<()>
generate_transaction_id() -> String
```

#### AdminService API
```
adjust_balance(admin_id: u64, target_user: u64, guild_id: u64, operation: Operation, amount: i64, reason: String) -> Result<Transaction>
has_admin_permission(user_id: u64, guild_id: u64) -> Result<bool>
```

#### AuditService API
```
log_transaction(transaction: &Transaction) -> Result<()>
get_audit_logs(guild_id: u64, limit: Option<u32>) -> Result<Vec<AuditEntry>>
log_admin_action(action: &AdminAction) -> Result<()>
```

#### HelpService API
```
get_help_overview() -> Result<String>
get_command_help(command: &str) -> Result<String>
format_help_message(content: &str) -> Result<String>
```

#### LeaderboardService API
```
get_balance_leaderboard(guild_id: u64, limit: Option<u32>) -> Result<Vec<LeaderboardEntry>>
get_transfer_count_leaderboard(guild_id: u64, limit: Option<u32>) -> Result<Vec<LeaderboardEntry>>
format_leaderboard(entries: Vec<LeaderboardEntry>, metric: LeaderboardMetric) -> Result<String>
```

#### ConfigService API
```
get_server_config(guild_id: u64) -> Result<ServerConfig>
update_currency_symbol(guild_id: u64, symbol: &str, admin_id: u64) -> Result<ServerConfig>
update_currency_name(guild_id: u64, name: &str, admin_id: u64) -> Result<ServerConfig>
update_thousands_separator(guild_id: u64, enabled: bool, admin_id: u64) -> Result<ServerConfig>
format_currency(amount: i64, config: &ServerConfig) -> Result<String>
```

### 外部 API

#### Discord Slash Commands
- `/balance [user]`: 查詢餘額
- `/transfer <user> <amount> [reason]`: 轉帳
- `/adjust <user> <operation> <amount> <reason>`: 管理調整
- `/audit [limit]`: 稽核查詢
- `/help [command]`: 指令說明
- `/leaderboard [metric] [limit]`: 排行榜

#### 資料庫 API
- PostgreSQL 連接和查詢
- 事務管理
- 連接池管理

#### Redis API
- 快取操作
- 會話存儲
- 排行榜快取

### API 標準
- **錯誤處理**: 統一的 Result<T, Error> 模式
- **日誌記錄**: 結構化日誌格式
- **性能監控**: 所有 API 調用都記錄執行時間
- **權限檢查**: 統一的權限驗證中間件

