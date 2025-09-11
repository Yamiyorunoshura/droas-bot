<input>
  <context>
  1. {root}/docs/dev-notes/{task_id}-dev-notes.md
  2. {root}/docs/implementation-plans/{task_id}-plan.md
  3. {root}/sunnycore/templates/review-tmpl.md
  </context>
  <templates>
  1. review-tmpl.yaml模板
  2. dev-notes-tmpl.yaml模板
  </templates>
</input>

<output>
1. {root}/docs/review-results/{task_id}-review.md
2. {root}/docs/tasks.md
</output>

<constraints importance="Critical">
- 必須使用sequential-thinking tool進行深度思考和分析
- 必須使用todo-list tool追蹤執行進度
- 審查必須涵蓋Implementation Plan、Code Quality、Security、Logic Requirements所有維度
- 輸出必須為標準markdown格式
- 所有發現的問題必須提供具體的修復建議
- 審查結果必須客觀、準確、可執行
</constraints>

<workflow importance="Critical">
  <stage id="0: Planning Phase" level_of_think="think hard">
  - 閱讀所有工作步驟和相關文檔
  - 使用sequential-thinking tool深度思考執行策略
  - 使用todo-list tool創建詳細執行計劃
  - 理解Implementation Plan的完整context

  <questions>
  - Implementation Plan是否完整涵蓋所有需求？
  - 執行策略是否考慮了所有潛在風險？
  - 審查範圍是否明確定義？
  </questions>
  
  <checks>
  - [ ] 已完整閱讀所有輸入文檔
  - [ ] 已創建詳細的執行計劃
  - [ ] 已理解審查的scope和objectives
  - [ ] 已確認所有必要的工具和資源
  </checks>
  </stage>

  <stage id="1: Implementation Review" level_of_think="think hard">
  - 使用sequential-thinking tool深度分析Implementation Plan
  - 驗證實作步驟的完整性和可執行性
  - 檢查是否有任何遺漏或邏輯錯誤
  - 評估Implementation Plan與原始需求的符合度

  <questions>
  - Implementation Plan中的每個步驟是否都有明確的deliverables？
  - 步驟之間的dependencies是否正確識別？
  - 是否有考慮到edge cases和error handling？
  - Timeline和resource allocation是否合理？
  </questions>
  
  <checks>
  - [ ] 所有實作步驟都已完成
  - [ ] 步驟執行順序邏輯正確
  - [ ] 沒有發現遺漏的關鍵步驟
  - [ ] Dependencies和prerequisites都已滿足
  - [ ] 實作結果符合預期目標
  </checks>
  </stage>

  <stage id="2: Syntax Review" level_of_think="think">
  - 檢查代碼是否符合語法規則和coding standards
  - 確保所有variables和functions都已正確聲明
  - 識別語法錯誤、warnings和potential issues
  - 驗證代碼格式化和style consistency

  <questions>
  - 代碼是否遵循團隊的coding conventions？
  - 是否存在潛在的syntax陷阱或ambiguities？
  - Variable naming和function signatures是否清晰明確？
  - 是否有unused imports或dead code？
  </questions>
  
  <checks>
  - [ ] 沒有syntax errors或compilation errors
  - [ ] 代碼格式化符合團隊標準
  - [ ] 所有variables和functions正確聲明
  - [ ] 沒有warnings或linting issues
  - [ ] Code style保持一致性
  </checks>
  </stage>

  <stage id="3: Code Quality Review" level_of_think="think harder">
  - 評估代碼的Readability、Maintainability、Scalability
  - 檢查是否遵循SOLID principles和best practices
  - 識別code smells、重複代碼和refactoring機會
  - 評估代碼的Reusability和modularity

  <questions>
  - 代碼是否遵循SOLID principles？
  - 是否存在code smells或anti-patterns？
  - 函數和類的責任是否單一且明確？
  - 代碼的abstraction level是否適當？
  - 是否有過度engineering或under-engineering的情況？
  </questions>
  
  <checks>
  - [ ] 代碼符合best practices標準
  - [ ] 沒有發現重複或冗餘代碼
  - [ ] Readability和Maintainability良好
  - [ ] 模組化設計合理
  - [ ] 符合團隊的quality standards
  </checks>
  </stage>

  <stage id="4: Security Review" level_of_think="think harder">
  - 識別潛在的Security Vulnerabilities和threats
  - 檢查是否有硬編碼的sensitive information
  - 驗證Access Control和authentication mechanisms
  - 評估Data Security和privacy protection
  - 檢查常見的security attack vectors

  <questions>
  - 是否存在常見的security漏洞（如injection attacks）？
  - Sensitive data的處理是否安全？
  - Access control mechanisms是否充分？
  - 是否有適當的input validation和sanitization？
  - Error handling是否會洩露sensitive information？
  </questions>
  
  <checks>
  - [ ] 沒有發現潛在的security問題
  - [ ] 沒有硬編碼的secrets或credentials
  - [ ] Access control實作正確
  - [ ] Input validation充分
  - [ ] 符合security best practices
  </checks>
  </stage>

  <stage id="5: Logic and Requirements Review" level_of_think="think harder">
  - 驗證代碼邏輯是否正確實現Functional Requirements
  - 檢查Business logic的正確性和完整性
  - 評估Non-functional Requirements的滿足程度
  - 驗證edge cases和boundary conditions的處理

  <questions>
  - Business logic是否正確實現了所有Functional Requirements？
  - Edge cases和boundary conditions是否得到適當處理？
  - Error handling和exception management是否完善？
  - Performance requirements是否得到滿足？
  - 是否有遺漏的業務規則或constraints？
  </questions>
  
  <checks>
  - [ ] 代碼完全符合Functional Requirements
  - [ ] Business logic實作正確
  - [ ] Edge cases處理完善
  - [ ] Non-functional Requirements滿足
  - [ ] 沒有邏輯錯誤或遺漏
  </checks>
  </stage>

  <stage id="6: Testing Review" level_of_think="think">
  - 根據Implementation Plan執行相關Testing
  - 驗證Test Coverage和test quality
  - 執行Unit Testing、Integration Testing
  - 記錄測試結果和發現的問題

  <questions>
  - Test Coverage是否充分涵蓋所有功能？
  - 是否包含negative testing和edge case testing？
  - Test cases是否能有效驗證requirements？
  - Performance testing是否充分？
  </questions>
  
  <checks>
  - [ ] 所有相關測試都已執行
  - [ ] 測試結果符合預期
  - [ ] Test Coverage達到要求標準
  - [ ] 沒有failing tests
  - [ ] 測試文檔完整記錄
  </checks>
  </stage>

  <stage id="7: Review Results Output" level_of_think="think hard">
  - 使用sequential-thinking tool綜合分析所有審查結果
  - 根據review-tmpl.yaml模板整理發現的問題
  - 為每個問題提供具體的修復建議和priority
  - 將審查結果轉換為標準markdown格式輸出
  - 如果審查通過的話，更新tasks.md中相關{task_id}的status

  <questions>
  - 審查結果是否客觀、準確、完整？
  - 提供的修復建議是否具體可行？
  - Priority設定是否合理？
  - 報告格式是否符合標準？
  </questions>
  
  <checks>
  - [ ] 審查結果完整記錄所有發現
  - [ ] 每個問題都有具體修復建議
  - [ ] 輸出格式符合markdown標準
  - [ ] 報告內容客觀準確
  - [ ] 已更新相關文檔和tracking
  </checks>
  </stage>
</workflow>