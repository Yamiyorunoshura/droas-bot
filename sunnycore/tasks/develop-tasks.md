<input>
  <context>
  1. {root}/docs/implementation-plan/{task_id}-plan.md - Implementation Plan (實作計劃)
  2. {root}/sunnycore/templates/dev-notes-tmpl.md - Development Notes Template (開發筆記模板)
  3. {root}/docs/architecture/*.md
  </context>
  <templates>
  1. {root}/sunnycore/templates/dev-notes-tmpl.md - Development Notes Template
  </templates>
</input>

<output>
1. {root}/docs/dev-notes/{task_id}-dev-notes.md - 完整的Development Notes文檔
2. 高質量、經過測試的代碼實作
3. 完整的Test Coverage和Test Cases
4. 重構後符合最佳實踐的代碼
</output>

<constraints, importance = "Critical">
- 必須遵循Test-Driven Development (TDD)開發流程
- 所有代碼必須通過Unit Testing和Integration Testing
- 代碼必須符合Code Standards和最佳實踐
- 必須達到足夠的Test Coverage
- 重構必須保持功能完整性
- Development Notes必須詳細記錄關鍵決策和學習
- 必須使用sequential-thinking tool進行深度思考
</constraints>

<workflow, importance = "Critical">
  <stage id="0: 創建Implementation Plan", level_of_think = "think hard">
  - 閱讀並分析所有Implementation Plan文檔
  - 閱讀所有工作步驟中的無序列表項
  - 使用sequential thinking tool深度思考所有步驟的實現方式
  - 識別Dependencies和Parallel Tasks
  - 使用plan tool創建詳細的執行計劃

  <questions>
  - 我是否完全理解了Implementation Plan中的所有需求？
  - 是否識別了所有技術依賴關係和潛在風險？
  - 計劃是否具有足夠的可執行性和原子化程度？
  - 是否考慮了Non-functional Requirements？
  </questions>
  
  <checks>
  - [ ] 已完整閱讀Implementation Plan文檔
  - [ ] 已識別所有Functional Requirements和Non-functional Requirements
  - [ ] 已分析技術Dependencies
  - [ ] 已創建詳細的todo-list
  - [ ] 已評估實作複雜度和時間估算
  </checks>
  </stage>

  <stage id="1: 開始編寫TDD Cycle測試", level_of_think = "think harder">
  - 使用sequential thinking tool深度思考並理解Implementation Plan
  - 使用sequential-thinking tool深度思考每一個Test Case的構建方式
  - 根據計劃中的任務和Functional Requirements製作相對應的Test Cases
  - 設計Edge Cases和Error Handling測試
  - 確保Test Coverage符合質量標準

  <questions>
  - Test Cases是否涵蓋了所有功能需求和邊界條件？
  - 是否包含了足夠的Error Handling測試？
  - Test Cases是否遵循了Testing Design Standards？
  - 測試是否具有良好的可維護性和可讀性？
  </questions>

  <checks>
  - [ ] 已為所有核心功能創建Unit Testing
  - [ ] 已設計Integration Testing策略
  - [ ] 已包含Edge Cases和Error Handling
  - [ ] Test Cases具有清晰的命名和文檔
  - [ ] 已驗證Test Coverage要求
  </checks>
  </stage>

  <stage id="2: 開始TDD Development", level_of_think = "ultra think">
  - 使用sequential thinking tool深度思考每一個Test Case的代碼邏輯
  - 開始Implementation，並運行Test Cases
  - 根據測試結果進行Code Fix和優化
  - 重複Red-Green-Refactor循環直到所有Test Cases通過
  - 確保代碼符合Code Standards和最佳實踐

  <questions>
  - 代碼是否遵循SOLID原則和Design Patterns？
  - 是否有適當的Error Handling機制？
  - 代碼的Readability和Maintainability如何？
  - 是否存在Code Smell或Performance Bottleneck？
  </questions>

  <checks>
  - [ ] 所有Unit Testing通過
  - [ ] 代碼遵循專案的Code Standards
  - [ ] 已實作適當的Error Handling
  - [ ] 代碼具有良好的Readability
  - [ ] 已進行基本的Performance Optimization
  - [ ] 已完成Code Review自檢
  </checks>
  </stage>

  <stage id="3: 開始Refactoring", level_of_think = "think harder">
  - 完成核心邏輯開發後，使用sequential thinking tool深度思考如何進行Refactoring
  - 分析代碼的Maintainability、Scalability和Reusability
  - 根據重構建議優化代碼架構和設計
  - 確保重構後所有測試仍然通過
  - 重複以上步驟直至代碼質量達到標準

  <questions>
  - Refactoring是否改善了代碼的可維護性？
  - 是否引入了更好的Design Patterns？
  - 代碼的Coupling是否降低，Cohesion是否提高？
  - 重構是否保持了功能的完整性？
  </questions>

  <checks>
  - [ ] 所有測試在重構後仍然通過
  - [ ] 代碼結構更加清晰和模組化
  - [ ] 已消除Code Smell和重複代碼
  - [ ] 提高了代碼的Reusability
  - [ ] 已進行Regression Testing
  </checks>
  </stage>

  <stage id="4: 製作Development Notes", level_of_think = "think">
  - 根據dev-notes-tmpl.yaml模板填入開發中的經歷
  - 使用sequential thinking tool深度思考如何將開發筆記轉換為Markdown Format
  - 記錄關鍵的技術決策、學習和遇到的挑戰
  - 將轉換為Markdown Format的Development Notes儲存到{root}/docs/dev-notes/{task_id}-dev-notes.md
  - 確保文檔的完整性和可讀性

  <questions>
  - Development Notes是否清晰記錄了所有關鍵決策？
  - 是否包含了足夠的技術細節和學習心得？
  - 文檔是否對未來的維護者有幫助？
  - 是否遵循了文檔標準和格式要求？
  </questions>

  <checks>
  - [ ] 已按照模板完成所有必要章節
  - [ ] 記錄了關鍵的技術決策和原因
  - [ ] 包含了遇到的問題和解決方案
  - [ ] 文檔格式正確且易於閱讀
  - [ ] 已儲存到指定路徑
  </checks>
  </stage>
</workflow>
