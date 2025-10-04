[輸入]
  1. {root}/docs/requirements --規範需求來源
  2. {root}/docs/architecture/*.md --既有架構文件集
  3. {root}/sunnycore/scripts/shard-architecture.py --架構分片腳本
  4. {root}/sunnycore/templates/architecture-tmpl.yaml --架構模板

[輸出]
  1. {root}/docs/architecture/*.md --更新後的架構文件集（*.md 格式）

[約束]
  1. 必須在提議設計前徹底審查 {root}/docs/requirements 與 {root}/docs/architecture/*.md（證據：在架構草稿中以 `參考：{檔案路徑}#{章節標題}` 格式引用已審查內容）
  2. 必須保留既有契約；任何提議變更都必須包含明確的「影響分析」子章節
  3. 必須完全遵循 {root}/sunnycore/templates/architecture-tmpl.yaml 結構與章節順序
  4. 必須草擬至 {root}/docs/architecture.md，然後執行：uv run {root}/sunnycore/scripts/shard-architecture.py 進行分片
  5. 應使用清晰簡潔的英文與全文 2 空格縮排

[工具]
  1. **todo_write**
    - [步驟1:追蹤任務]
    - [步驟2:更新任務狀態]
    - [步驟3:追蹤任務]
  2. **sequentialthinking**
    - [步驟1:評估既有架構]
    - [步驟2:設計模組邊界與整合模式]
    - [步驟3:結構化草擬]
  3. **claude-context**
    - [步驟1:評估既有架構-處理大型文件集]

[工具指引]
  1. **todo_write**
    - 在評估階段創建待辦清單，包含所有主要任務
    - 每完成一個步驟即更新對應待辦項目狀態為 completed
    - 狀態閘門：僅允許單一任務為 in_progress；完成後立即標記 completed
  2. **sequentialthinking**
    - 簡單任務推理：1-3 totalThoughts
    - 中等任務推理：3-5 totalThoughts
    - 複雜任務推理：5-8 totalThoughts
    - 完成原本推理步數後依然有疑問：nextThoughtNeeded = true
    - 你必須完成所有設定的推理步數
  3. **claude-context**
    - 使用場景：處理大型既有架構文件集時
    - 可用於分段讀取與理解複雜架構
  4. **context7**
    - 使用場景：研究架構模式與最佳實踐時
    - 可使用 resolve-library-id 與 get-library-docs 查詢特定架構框架的設計模式與文件

[步驟]
  1. 評估既有架構階段
    - 檢視 {root}/docs/architecture/*.md 下的當前架構
    - 識別擴充點、約束條件與共享服務
    - 對應受影響的領域、限界上下文與依賴關係
    - 創建todo list追蹤後續設計與撰寫任務

  2. 設計新模組階段
    - 定義新模組的責任、邊界與介面
    - 指定與既有元件的資料流與互動方式
    - 評估非功能性需求（安全性、可觀測性、效能）與相容性
    - 為所有提議變更撰寫「影響分析」，說明對既有系統的潛在影響

  3. 撰寫與分片階段
    - 使用架構模板草擬 Markdown 格式的 {root}/docs/architecture.md
    - 確保章節強調新模組與整合影響
    - 執行分片腳本以分割文件：uv run {root}/sunnycore/scripts/shard-architecture.py
    - 驗證文件出現在 {root}/docs/architecture/ 下

  4. 最終化階段
    - 對照約束與指引問題進行交叉檢查；修正缺口與不一致
    - 確認所有影響分析已包含且完整
    - 驗證新舊模組間的相容性

[DoD]
  - [ ] 已徹底審查 {root}/docs/requirements 與 {root}/docs/architecture/*.md
  - [ ] 已識別擴充點、約束條件與受影響領域
  - [ ] 新模組已記錄邊界、介面與資料流
  - [ ] 所有提議變更都包含明確的「影響分析」子章節
  - [ ] 與既有契約的相容性已闡明（無破壞性變更）
  - [ ] {root}/docs/architecture.md 存在且遵循模板
  - [ ] 已執行 shard-architecture.py 並驗證分片文件生成
  - [ ] 所有待辦項目已完成
