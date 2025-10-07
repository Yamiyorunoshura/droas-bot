## API Documentation


### Internal APIs

#### Balance Service API
**endpoint**: BalanceService
**method**: adjust_balance_by_admin
**description**: 管理員調整用戶餘額
**parameters**:
- admin_user_id: i64
- admin_users: &[i64]
- target_user_id: i64
- amount: BigDecimal
- reason: String
**response**: BalanceResponse
**authentication**: Admin permission verification

#### Admin Service API
**endpoint**: AdminService
**method**: verify_admin_permission
**description**: 驗證用戶管理員權限
**parameters**:
- user_id: i64
**response**: Result<bool, DiscordError>
**authentication**: Discord native permissions or authorized list

#### Admin Audit Service API
**endpoint**: AdminAuditService
**method**: log_admin_operation
**description**: 記錄管理員操作到審計日誌
**parameters**:
- record: AdminAuditRecord
**response**: Result<(), DiscordError>
**authentication**: Internal service call

### External APIs

#### Discord API
**library**: Serenity 0.12
**version**: v2+
**documentation-url**: https://docs.rs/serenity/latest/serenity/
**usage-context**: 主要用戶界面和命令處理

#### PostgreSQL
**library**: SQLx 0.7
**version**: 16.x
**documentation-url**: https://docs.rs/sqlx/latest/sqlx/
**usage-context**: 資料持久化和查詢

#### Redis
**library**: redis-rs 0.23
**version**: 8.x
**documentation-url**: https://docs.rs/redis/latest/redis/
**usage-context**: 快取和性能優化

#### Warp
**library**: Warp 0.3
**version**: 0.3
**documentation-url**: https://docs.rs/warp/latest/warp/
**usage-context**: HTTP 監控端點

### API Standards
**versioning**: Semantic Versioning (v0.2.0)
**authentication**: Discord OAuth2 + Admin Permission Lists + Native Discord Permissions
**error-handling**: Centralized error handling with DiscordError enum
**rate-limiting**: Discord API rate limits + internal rate limiting for admin operations

*source_refs: ["docs/architecture/API Documentation.md", "src/services/*.rs"]*

