<input>
  <context>
  1. {root}/docs/review-results/*.md - Review Records from development process
  2. {root}/docs/dev-notes/*.md - Development Notes from implementation
  3. {root}/sunnycore/templates/completion-report-tmpl.yaml - Completion Report Template
  </context>
  <templates>
  1. {root}/sunnycore/completion-report-tmpl.yaml - Project completion report template
  </templates>
</input>

<output>
1. {root}/docs/completion-report.md - Comprehensive Project Completion Report
</output>

<constraints, importance="Critical">
- 必須使用sequential thinking tool進行深度分析
- 必須整合所有review-results和dev-notes的內容
- 輸出格式必須為markdown
- 必須遵循模板結構
- 專業術語使用英文標準化表達
</constraints>

<workflow, importance="Critical">
  <stage id="0: Project Planning", level_of_think="think hard">
  - 識別所有Workflow Steps
  - 閱讀所有工作步驟下的無序列表項
  - 使用todo list tool為每一個無序列表項創建todo items

  <questions>
  - 所有必要的輸入文件是否都已識別？
  - 工作流程的邏輯順序是否合理？
  - 是否需要額外的工具或資源？
  </questions>
  
  <checks>
  - [ ] 已識別所有輸入文件路徑
  - [ ] 已理解輸出要求
  - [ ] 已制定詳細執行計劃
  - [ ] 已確認工具可用性
  </checks>
  </stage>

  <stage id="1: Review Results Summary", level_of_think="think harder">
  - 閱讀所有review-results/*.md文件
  - 使用sequential thinking tool深度思考並理解每個Review Record
  - 總結Development Process中遇到過的問題
  - 總結現有Project的Potential Issues
  - 總結現有Project的Unresolved Issues

  <questions>
  - 每個Review Record的核心發現是什麼？
  - 問題的嚴重程度如何分類？
  - 是否存在重複或相關的問題？
  - 哪些問題已解決，哪些仍待處理？
  </questions>

  <checks>
  - [ ] 已閱讀所有review-results文件
  - [ ] 已分類整理發現的問題
  - [ ] 已識別潛在風險
  - [ ] 已標記未解決問題
  - [ ] 已進行問題優先級排序
  </checks>
  </stage>

  <stage id="2: Development Notes Summary", level_of_think="think harder">
  - 閱讀所有dev-notes/*.md文件
  - 使用sequential thinking tool深度思考並理解每個Development Note
  - 總結Development Process中遇到過的問題
  - 總結Development Process中對問題的Solution
  - 提取Best Practices和Lessons Learned

  <questions>
  - 開發過程中最常見的問題類型是什麼？
  - 哪些解決方案最有效？
  - 是否有可重複使用的解決模式？
  - 團隊學到了哪些重要經驗？
  </questions>

  <checks>
  - [ ] 已閱讀所有dev-notes文件
  - [ ] 已整理問題解決方案
  - [ ] 已提取最佳實踐
  - [ ] 已記錄經驗教訓
  - [ ] 已識別可改進領域
  </checks>
  </stage>

  <stage id="3: Completion Report Generation", level_of_think="ultra think">
  - 根據Template Files填入Project Summary
  - 使用sequential thinking tool思考如何將內容轉換為markdown格式
  - 整合所有分析結果生成綜合報告
  - 將轉換為markdown格式的文件輸出到{root}/docs/completion-report.md

  <questions>
  - 報告結構是否完整且邏輯清晰？
  - 所有重要發現是否都已包含？
  - 內容是否平衡且客觀？
  - 報告是否為未來專案提供有價值的參考？
  </questions>

  <checks>
  - [ ] 已按模板結構組織內容
  - [ ] 已整合所有分析結果
  - [ ] 已確保markdown格式正確
  - [ ] 已進行內容完整性檢查
  - [ ] 已驗證輸出文件路徑
  - [ ] 已確保專業術語使用一致
  </checks>
  </stage>
</workflow>