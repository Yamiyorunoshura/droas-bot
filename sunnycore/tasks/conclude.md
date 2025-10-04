[輸入]
  1. {root}/docs/dev-notes/{task_id}-dev-notes.md --開發筆記（必要）
  2. {root}/docs/review_results/{task_id}-review.md --審查報告（必要）
  3. {root}/claude code/templates/completion-report-tmpl.yaml --總結報告模板（必要）
  4. {root}/docs/requirements/*.md --需求文件（可選）
  5. {root}/docs/tasks.md --任務清單（可選）

[輸出]
  1. 總結報告：{root}/docs/completion-report.md（Markdown 格式）

[約束]
  1. 你需要確保總結報告符合模板要求
  2. 總結報告必須包含以下核心內容：
    (1) 所有關鍵決策及其理由
    (2) 技術選擇與替代方案比較
    (3) 遇到的問題、根因分析及解決方案
    (4) 未來建議
    (5) DoD驗證證據（格式為「檔案路徑:行號」，如「src/main.py:L42-L56」）
  3. 若必要輸入檔案（dev-notes, review, template）缺失或格式錯誤，須在終端輸出缺失檔案清單並中止執行，等待用戶補件
  4. 若核心內容5項中任一項在開發筆記或審查報告中完全缺失，應在總結報告對應章節標註「待補充：缺少XX項目」並繼續執行

[工具]
  1. **todo_write**：建立和管理任務清單
    - [步驟1:創建待辦清單]
    - [步驟2:追蹤任務進度]
  2. **sequentialthinking**：進行結構化推理與驗證
    - [步驟2:構思報告的推理任務]
    - [步驟3:驗證DoD是否滿足]
  3. **claude-context**：搜尋程式碼庫以定位實作細節
    - [步驟2:尋找相關程式碼]
    - 使用前提：程式碼庫已通過 index_codebase 索引
    - 失敗處理：若未索引或搜尋失敗，改用 grep 工具進行關鍵字搜尋，或標註「無法定位程式碼證據」並繼續執行
  4. **grep**：關鍵字搜尋工具
    - [步驟3:精確關鍵字搜尋]
    - 使用場景：claude-context失敗時的備案工具

[工具指引]
  1. **todo_write**
    - 在準備階段創建待辦清單，包含所有主要任務
    - 每完成一個步驟即更新對應待辦項目狀態為 completed
  2. **sequentialthinking**
    - 簡單任務推理：1-3 totalThoughts
    - 中等任務推理：3-5 totalThoughts
    - 複雜任務推理：5-8 totalThoughts
    - 完成原本推理步數後依然有疑問：nextThoughtNeeded = true
    - 你必須完成所有設定的推理步數
    - 參數說明：
      * totalThoughts (number): 預估推理步數，可依實際需求調整
      * nextThoughtNeeded (boolean): 是否需要繼續推理，完成所有步驟後設為 false
    - 錯誤處理：若推理過程發現邏輯錯誤，使用 isRevision=true 修正先前思路
  3. **claude-context**
    - 主要用途：尋找相關程式碼片段（search_code）
    - 使用時機：需要定位特定實作細節或驗證技術決策時（指審查報告或開發筆記提及具體技術選擇但未標註「檔案路徑:行號」格式證據時）
    - 查詢技巧：使用自然語言描述要找的功能或模式
    - 產出要求：標註來源文件與行號（格式為「檔案路徑:行號」，如「src/main.py:L42-L56」），以利後續稽核（對應 evidence）

[步驟]
  1. 輸入驗證階段
    - 驗證所有輸入檔案存在性（dev-notes, review, template, requirements/*.md, tasks.md）
    - 若必要輸入檔案缺失（dev-notes, review, template），須在終端輸出缺失清單並中止執行；若可選檔案缺失（requirements, tasks.md），可標記並繼續執行
    - 根據實際任務創建todo list

  2. 資訊提取階段
    - 閱讀開發筆記與審查報告
    - 提取約束2前4項核心內容（關鍵決策及其理由、技術選擇與替代方案比較、遇到的問題及解決方案、未來建議）
    - 若關鍵資訊缺失，依約束2處理

  3. 結構對應階段
    - 閱讀 completion-report-tmpl.yaml 了解欄位結構
    - 將提取的資訊對應至模板各欄位：
      * 關鍵決策及其理由 → development_summary.key_decisions
      * 技術選擇與替代方案比較 → development_summary.technologies_used
      * 遇到的問題及解決方案 → development_summary.challenges_encountered
      * 未來建議 → recommendations.future_improvements
      * DoD驗證證據 → 報告末尾驗證結果區塊
    - 使用 claude-context 搜尋程式碼以補充實作細節（當報告提及具體技術選擇但未標註「檔案路徑:行號」格式證據時）

  4. 撰寫報告階段
    - 依據模板結構填寫各章節
    - 整合所有資訊成完整的總結報告
    - 寫入 {root}/docs/completion-report.md

  5. 品質檢查階段
    - 驗證報告符合模板要求（結構完整）
    - 驗證報告內容完整（逐項檢查約束2要求的5項核心內容是否在報告中明確呈現，並標註對應章節）
    - 檢查所有 DoD 驗證證據是否已包含
    - 若發現缺失應返回步驟4修正報告（最多迭代2次，若仍有缺失則標註「待補充」並繼續執行）

  6. DoD驗證階段
    - 逐項檢查所有DoD項目是否已滿足
    - 將DoD逐項驗證結果附於報告末尾
    - 確認所有待辦項目已完成

  7. 檔案歸檔階段
    - 驗證 requirements/*.md 和 tasks.md 存在性
    - 使用終端命令將檔案移至 {root}/docs/archive/{version_name}/（若資料夾不存在則先創建）
    - 建議命令：`mkdir -p {root}/docs/archive/{version_name} && mv -n {files} {root}/docs/archive/{version_name}/`（使用 -n 參數避免覆蓋已存在檔案）
    - 若 mv -n 因文件已存在而失敗，應記錄警告並標註「文件已存在於歸檔資料夾」，DoD 視為已完成
    - 執行驗證命令：`ls {root}/docs/archive/{version_name}` 確認檔案存在於目標資料夾，若目標資料夾中缺少預期檔案，輸出錯誤訊息並標註 DoD 未完成，等待用戶處理

[DoD]
  - [ ] 總結報告已產出且符合模板結構（包含模板中所有必要區塊）
  - [ ] 總結報告內容完整涵蓋約束2列出的5項核心內容
  - [ ] 若存在requirements/*.md和tasks.md，則已安全移至歸檔資料夾；若不存在則已記錄於報告
  - [ ] 所有待辦項目已完成（包含：輸入驗證、資訊提取、結構對應、撰寫報告、品質檢查、DoD驗證、檔案歸檔）