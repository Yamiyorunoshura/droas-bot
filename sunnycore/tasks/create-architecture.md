[輸入]
  1. {root}/docs/requirements/*.md --權威專案需求
  2. {root}/sunnycore/scripts/shard-architecture.py --架構分片腳本
  3. {root}/sunnycore/templates/architecture-tmpl.yaml --規範架構模板

[輸出]
  1. {root}/docs/architecture/ 目錄下的架構文件集（*.md 格式）
  2. 最小預期文件範例（可根據專案複雜度產出更多分類文件）：
    - {root}/docs/architecture/overview.md
    - {root}/docs/architecture/components.md
    - {root}/docs/architecture/traceability_matrix.md

[約束]
  1. 必須在架構設計前驗證 {root}/docs/requirements/*.md 存在且完整；若發現需求不完整或存在衝突，應記錄問題並與需求制定者確認，不得自行臆測
  2. 必須創建明確的需求至架構對應關係，涵蓋功能性與非功能性需求（對應係指需求 ID 明確關聯至特定元件名稱或設計決策編號）
  3. 必須驗證每個需求都有對應的架構元件或設計決策
  4. 必須使用規範模板撰寫 {root}/docs/architecture.md；嚴格保留章節順序與 2 空格縮排；不得引入不存在的路徑
  5. 草擬完成後，必須執行 'uv run {root}/sunnycore/scripts/shard-architecture.py' 並驗證產出物出現在 {root}/docs/architecture/ 下

[工具]
  1. **todo_write**
    - [步驟1:追蹤與更新執行任務]
    - [步驟2:設計期間更新任務狀態]
    - [步驟3:追蹤撰寫進度與結果]
    - [步驟4:以最終狀態關閉任務]
  2. **sequentialthinking**
    - [步驟1:分解需求並識別架構模式]
    - [步驟2:架構系統元件並驗證設計決策]
    - [步驟3:結構化草擬與驗證步驟]
    - [步驟4:執行最終驗證序列]
  3. **context7**
    - [步驟2:取得外部套件與架構模式參考]

[工具指引]
  1. **todo_write**
    - 在需求分析階段創建待辦清單，包含所有主要任務
    - 每完成一個步驟即更新對應待辦項目狀態為 completed
    - 狀態閘門：僅允許單一任務為 in_progress；完成後立即標記 completed
  2. **sequentialthinking**
    - 簡單任務推理：1-3 totalThoughts
    - 中等任務推理：3-5 totalThoughts
    - 複雜任務推理：5-8 totalThoughts
    - 完成原本推理步數後依然有疑問：nextThoughtNeeded = true
    - 你必須完成所有設定的推理步數
  3. **context7**
    - 使用場景：當遇到不熟悉的技術棧、需要驗證設計模式適用性、或尋找特定領域的最佳實踐時
    - 可用於研究特定技術棧或設計模式

[步驟]
  1. 需求分析階段
    - 驗證 {root}/docs/requirements/*.md 下所有需求的完整性與一致性
    - 提取功能性/非功能性需求，並將非功能性需求轉換為架構約束
    - 創建對應矩陣（需求 ID → 元件/決策）並識別缺口或衝突
    - 創建todo list追蹤後續架構設計任務

  2. 架構設計階段
    - 基於需求分析劃定元件、邊界與規範資料流
    - 確保每個需求對應至架構元素；定義互動契約與資料模式
    - 記錄決策（建議使用 ADR 格式或表格形式）並包含需求可追溯性，處理跨領域關注點（安全性、可觀測性、效能）

  3. 撰寫階段
    - 使用 {root}/sunnycore/templates/architecture-tmpl.yaml 草擬 {root}/docs/architecture.md
    - 包含需求追溯矩陣並確保每個對應關係都已處理
    - 執行 'uv run {root}/sunnycore/scripts/shard-architecture.py' 並驗證產出物出現在 {root}/docs/architecture/ 下；若執行失敗，應檢查 architecture.md 格式是否符合模板規範，並修正後重新執行

  4. 最終驗證階段
    - 透過對應矩陣交叉驗證架構是否滿足所有需求
    - 修正印刷錯誤並標準化術語
    - 確認架構決策由需求證明合理

[異常處理]
  1. shard-architecture.py 執行失敗：檢查 architecture.md 格式是否符合模板規範，修正後重新執行
  2. 需求衝突無法調和：記錄衝突並與需求制定者確認，不得自行臆測
  3. 架構設計不可行：記錄技術限制並提出替代方案

[DoD]
  - [ ] 已驗證 {root}/docs/requirements/*.md 存在且完整
  - [ ] 已創建需求至架構對應矩陣（需求 ID → 元件/決策），完整準確（覆蓋率100%，無遺漏或錯誤對應）
  - [ ] 所有功能性需求已對應至特定架構元件
  - [ ] 非功能性需求已轉換為可衡量的架構約束
  - [ ] {root}/docs/architecture.md 存在且遵循模板
  - [ ] 已執行 shard-architecture.py 並驗證分片文件生成
  - [ ] 所有待辦項目已完成
