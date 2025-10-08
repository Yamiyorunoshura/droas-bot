## Deployment Architecture


### Environments
**development**: 開發環境，用於日常開發和測試。本地 PostgreSQL 和 Redis；開發模式日誌
**staging**: 預發布環境，用於最終驗證。模擬生產環境配置；完整的功能測試
**production**: 生產環境，面向最終用戶。高可用性資料庫；Redis 叢集；監控和警報

### Infrastructure
Docker 容器化部署；Kubernetes 叢集管理；負載均衡；自動擴展

### CI/CD
GitHub Actions 自動化 CI/CD；自動化測試；容器構建；部署到多環境

### Scaling Strategy
水平擴展；無狀態服務設計；資料庫讀寫分離；快取層擴展

*source_refs: ["docs/architecture/Deployment Architecture.md", "Cargo.toml", "archive/0.2.4/cutover.md"]*

