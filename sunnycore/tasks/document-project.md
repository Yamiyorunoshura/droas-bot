[輸入]
  1. {root}/docs/architecture/*.md --現有架構文件（作為上下文）
  2. {root}/sunnycore/templates/concluded-architecture-tmpl.yaml --架構文件模板

[輸出]
  1. {root}/docs/architecture/*.md --架構文件（Markdown 格式）
  2. JSON 格式的輸出結果（包含 files 陣列與 shards_created 計數）

[約束]
  1. 你必須產出完全符合 schema 的有效 JSON（additionalProperties=false）
  2. 產出最終交付物時，JSON 外不得包含任何文字說明
  3. 你應該使用 3 階段工作流程，並將檢查項目放在最後階段
  4. 每個產出的文件都必須對應至少 1 個來源參考（source_refs）
  5. 可以在 Markdown 文件中以 fenced code block 形式包含架構圖
  6. 所有文件路徑必須在 docs/architecture/ 下且副檔名為 .md

[工具]
  1. **todo_write**
    - [步驟1:創建任務清單]
    - [步驟2:追蹤任務進度]
    - [步驟3:追蹤任務進度]
  2. **sequentialthinking**
    - [步驟2:內容整合與文件撰寫的推理任務]

[工具指引]
  1. **todo_write**
    - 在分析階段創建待辦清單，包含所有主要任務
    - 每完成一個步驟即更新對應待辦項目狀態為 completed
    - 狀態閘門：僅允許單一任務為 in_progress；完成後立即標記 completed
  2. **sequentialthinking**
    - 簡單任務推理：1-3 totalThoughts
    - 中等任務推理：3-5 totalThoughts
    - 複雜任務推理：5-8 totalThoughts
    - 完成原本推理步數後依然有疑問：nextThoughtNeeded = true
    - 你必須完成所有設定的推理步數
    - 參數說明：
      * totalThoughts (number): 預估推理步數，可依實際需求調整
      * nextThoughtNeeded (boolean): 是否需要繼續推理，完成所有步驟後設為 false

[步驟]
  1. 分析與規劃階段
    - 閱讀所有輸入文件
    - 清點架構來源文件，記錄缺口與假設
    - 根據模板規劃目標文件的章節結構
    - 根據實際任務創建todo list
    - 交付物：任務清單與來源文件清單

  2. 撰寫架構文件階段
    - 整合並標準化內容，依照模板撰寫文件
    - 驗證文件完整性與內部一致性（逐一檢查：所有必要章節是否涵蓋、文件是否追溯至來源參考、文件路徑是否在 docs/architecture/ 下且為 .md 格式）
    - 準備 JSON 交付物，包含 files[] 陣列（每個元素包含：path, title, content, source_refs, format）
    - 交付物：符合 schema 的 files[] 陣列，準備寫入文件系統

  3. 分片與最終化階段
    - 執行 {root}/sunnycore/scripts/shard-architecture.py 並記錄 shards_created 數量
    - 驗證輸出 JSON 是否符合 schema；若有違規應立即修正
    - 僅輸出最終 JSON，不包含任何說明文字
    - 交付物：包含 shards_created 的最終 JSON 交付物

[DoD]
  - [ ] 已閱讀所有輸入文件
  - [ ] 已創建任務清單且所有項目已標記為 completed
  - [ ] 所有架構文件已依模板撰寫完成
  - [ ] 每個輸出文件都包含至少 1 個來源參考（source_refs）
  - [ ] 所有文件路徑在 docs/architecture/ 下且副檔名為 .md
  - [ ] 已執行 shard-architecture.py 並記錄 shards_created
  - [ ] 輸出 JSON 符合 schema（無 additionalProperties 違規）
  - [ ] 最終輸出僅包含 JSON，無額外說明文字
