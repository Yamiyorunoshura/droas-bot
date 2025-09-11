<input>
  <context>
  1. {root}/docs/review-results/*.md - Review Results文件，包含Code Review結果和Practice Level評估
  2. {root}/docs/dev-notes/*.md - Development Notes文件，記錄開發過程中的Error Cases修正和解決方案
  </context>
  <templates>
  1. {root}/sunnycore/project-knowledge-tmpl.yaml - 專案知識庫結構模板
  </templates>
</input>

<output>
1. {root}/docs/project-knowledge.md - 整合後的Project Knowledge Base，包含Error Cases和Best Practices的詳細總結
</output>

<constraints importance="Important">
- 必須完整提取所有Error Cases及其修復方案
- 只納入被評定為Platinum Level的Best Practices
- 保持知識條目的結構化和可檢索性
- 確保Markdown格式的正確性和Readability
- 遵循專案的文檔標準和命名規範
</constraints>

<workflow importance="Important">
  <stage id="1: Implementation Planning" level_of_think="think hard">
  - 識別所有Workflow步驟和依賴關係
  - 閱讀所有工作步驟下的無序列表項
  - 使用todo list tool為每一個無序列表項創建todo items

  <questions>
  - 所有相關的Review Results和Development Notes文件都已識別了嗎？
  - todo list是否涵蓋了所有必要的步驟？
  - 是否考慮了文件間的依賴關係和處理順序？
  </questions>
  
  <checks>
  - [ ] 已完整掃描{root}/docs/review-results/目錄
  - [ ] 已完整掃描{root}/docs/dev-notes/目錄
  - [ ] 已確認project-knowledge-tmpl.md的存在和可讀性
  - [ ] todo list已創建並包含所有必要的步驟
  </checks>
  </stage>

  <stage id="2: Error Case Analysis" level_of_think="think harder">
  - 閱讀Review Results的error log欄目以及其修復的方式
  - 閱讀Development Notes中關於Error Cases修正的方式
  - 分析錯誤模式和根本原因
  - 將這些錯誤整理為詳細的Error Cases總結

  <questions>
  - 是否已提取所有類型的錯誤案例（編譯錯誤、運行時錯誤、邏輯錯誤等）？
  - 每個Error Case是否都包含了完整的修復方案？
  - 是否識別了錯誤的根本原因和預防措施？
  </questions>

  <checks>
  - [ ] 已系統性分析所有Review Results中的error log
  - [ ] 已提取Development Notes中的所有錯誤修正記錄
  - [ ] 每個Error Case都包含問題描述、原因分析和解決方法
  - [ ] 已按錯誤類型進行分類整理
  </checks>
  </stage>

  <stage id="3: Best Practice Curation" level_of_think="think harder">
  - 查找Review Results中被評定為Platinum Level的實踐
  - 分析這些Best Practices的核心價值和適用場景
  - 將這些實踐整理為詳細的總結，包含實施指導

  <questions>
  - Platinum Level的評定標準是什麼？是否理解正確？
  - 這些Best Practices是否具有普遍適用性？
  - 是否提供了足夠的實施細節和指導？
  </questions>

  <checks>
  - [ ] 已識別所有Platinum Level的實踐項目
  - [ ] 每個Best Practice都包含詳細的實施指導
  - [ ] 已分析實踐的適用場景和限制條件
  - [ ] 已按實踐類型進行分類整理
  </checks>
  </stage>

  <stage id="4: Knowledge Base Creation" level_of_think="ultra think">
  - 基於project-knowledge-tmpl.md製作或更新Project Knowledge Base
  - 將整理好的Error Cases和Best Practices添加到知識庫中
  - 使用Sequential Thinking Tool思考如何將格式轉換為Markdown
  - 確保知識庫的結構清晰、內容完整、格式正確
  - 將轉換為Markdown的專案知識輸出到對應位置

  <questions>
  - Project Knowledge Base的結構是否邏輯清晰、易於導航？
  - Error Cases和Best Practices的整合是否平衡且互補？
  - Markdown格式是否符合專案標準？
  - 知識庫是否具備良好的可維護性和擴展性？
  </questions>

  <checks>
  - [ ] 已基於模板創建完整的Project Knowledge Base結構
  - [ ] 所有Error Cases已正確整合到知識庫中
  - [ ] 所有Best Practices已正確整合到知識庫中
  - [ ] Markdown格式正確，無語法錯誤
  - [ ] 知識庫已輸出到指定位置{root}/docs/project-knowledge.md
  - [ ] 已驗證輸出文件的完整性和可讀性
  </checks>
  </stage>
</workflow>