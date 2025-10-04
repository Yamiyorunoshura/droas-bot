[輸入]
  1. {root}/sunnycore/templates/dev-notes-tmpl.yaml --開發筆記模板
  2. {root}/docs/dev-notes/{task_id}-dev-notes.md --開發筆記
  3. {root}/docs/review-results/{task_id}-review.md --審查報告
  4. {root}/docs/architecture/*.md --架構設計

[輸出]
  1. 修復過後可正常運行的程式碼
  2. 修復總結（建議使用 MARKDOWN 呈現，包含：changes、tests、evidence、risk、rollback；建議格式，可根據專案需求調整）
  3. 更新後的開發筆記{root}/docs/dev-notes/{task_id}-dev-notes.md

[約束]
  1. 你需要確保修復過後的程式碼可以正常運行
  2. 你需要確保修復過後的程式碼符合架構設計
  3. 你需要確保修復過後的程式碼不破壞原有功能

[工具]
  1. **todo_write**
    - [步驟2:追蹤任務進度]（詳見工具指引）
  2. **sequentialthinking**
    - [步驟1:所有推理任務]
    - [步驟2:程式碼修復推理任務]
  3. **claude-context**
    - [步驟1:尋找相關程式碼]
    - [步驟2:尋找相關程式碼]

[工具指引]
  1. **sequentialthinking**
    - 簡單任務推理：1-3 totalThoughts
    - 中等任務推理：3-5 totalThoughts
    - 複雜任務推理：5-8 totalThoughts
    - 完成原本推理步數後依然有疑問：nextThoughtNeeded = true
    - 你必須完成所有設定的推理步數
  2. **claude-context**
    - 尋找相關程式碼：search_code
    - 產出時標註來源文件與行號，以利後續稽核（對應 evidence）
  3. **todo_write**
    - 狀態閘門：僅允許單一任務為 in_progress；完成後立即標記 completed；取消需註記理由
    - 更新節點：每完成可驗收步驟（如建立檔案、通過測試）即時更新狀態

[步驟]
  1. 準備階段
    - 閱讀並理解審查報告
    - 閱讀架構設計並對齊整體方向
    - 閱讀開發筆記與實際程式碼，思考問題所在
    - 結合架構與程式碼，制定原子化的修復任務

  2. 修復階段
    - 創建todo list以追蹤修復進度（依狀態閘門更新，詳見工具指引）
    - 每次程式碼在修復過後都應該運行測試確保沒有引入新的問題（例如：pytest -q 或 make test；以全部通過為準）
    - 測試通過條件：指令退出碼必須為 0；若失敗（退出碼≠0）應使用 git reset 或手動撤銷最近變更並修復後重試；可附覆蓋率/性能指標
    - 在完成所有修復後應該進行整合測試（如：pytest tests/integration, make test-integration, 或其他程式語言或專案的整合測試指令；同樣以測試全部通過為準）

  3. 總結階段
    - 總結修復過程與結果（應附帶文件路徑與行號/段落ID作為證據；若為版控專案，應附PR/commit連結；對應『修復總結』的 evidence 欄位，便於稽核）
    - 將總結依照模板寫入或更新到對應開發筆記

[DoD]
  - [ ] 所有單元測試已通過
  - [ ] 所有整合測試已通過
  - [ ] 開發筆記已更新並輸出