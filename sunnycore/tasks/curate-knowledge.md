## [Input]
  1. "{REVIEW}/*.md" --Review reports
  2. "{DEVNOTES}/*.md" --Development notes
  3. "{KNOWLEDGE}/*.md" --Existing knowledge base(if exist)
  4. "{CUTOVER}" --Cutover report (required)
  5. "{root}/docs/cutover-dev-notes.md" --Cutover development notes (required)
  6. "{PROGRESS}" --Progress record

## [Output]
  1. "{KNOWLEDGE}/*.md" --Knowledge base (create directory first if it does not exist)
    - Document organization: May produce "best-practices.md", "errors.md" based on actual content, or subdivide by domain/type
    - Content allocation: One entry per platinum practice, one entry per error case

## [Rules]
  1. Must identify all practices and errors during development, but only define platinum-level practices as best practices and produce them
  2. Platinum level will be marked in review reports, only need to read marks without making own judgment
  3. If no platinum-level practices are found, should record "No sufficiently validated best practices in this phase" in the knowledge base and explain the reason
  4. If contradictory practice recommendations are found, should annotate conflicts and preserve all evidence sources for subsequent decision-making

## [Tools]
  1. **sequentialthinking (MCP)**
    - [Step 2: Reason about knowledge base organization structure]
  2. **claude-context (MCP)**
    - [Step 2: Search codebase for relevant implementations]
    - Usage scenario: When technical details mentioned in documentation need to be verified or supplemented from code
  3. **todo_write**
    - [Step 2: Create task list; Steps 3-4: Track task progress]
    - Usage scenario: Create todo list in preparation phase, track task progress

## [Steps]
  1. Preparation Phase
    - Read all development notes and review reports
    - Identify best practices marked as platinum level and record in temporary list
    - Identify all errors encountered during development process (including error type, occurrence context, solution) and record in temporary list
    - Create todo list to track subsequent knowledge base conception and production tasks

  2. Knowledge Base Design Phase
    - Conceive the structure and classification method of project knowledge base (classify by technical domain/error type/development phase based on actual content)
    - Evaluate consideration factors including content volume, technical domain distribution, and error type diversity
    - Decide organization structure for best practices and errors (e.g., API design/error handling/testing strategy)
    - Plan document naming conventions and semantic classification approach

  3. Produce Documents Phase
    - Create "{KNOWLEDGE}/" directory if it does not exist
    - Produce best practices to corresponding documents by classification (document naming: "best-practices-{domain}.md" or "best-practices.md")
    - Produce error cases to corresponding documents by classification (document naming: "errors-{type}.md" or "errors.md")
    - Document content format: Each knowledge point includes title, description, evidence source, applicable scenarios
    - Annotate evidence source for each knowledge point (annotation format: file path + relevant section, e.g., "docs/dev-notes/feature-x.md" [Error Handling] paragraph)

  4. Finalization Phase
    - Archive the "review-results/" and "dev-notes/" files to "{ARCHIVE}/{version_name}/"

## [DoD]
  - [ ] All files in "{REVIEW}/" and "{DEVNOTES}/" have been read
  - [ ] Knowledge base structure and classification method have been conceived
  - [ ] Knowledge base directory "{KNOWLEDGE}/" has been created
  - [ ] All platinum-level best practices have been identified and produced by classification
  - [ ] All errors during development process have been identified and recorded (including type, context, solution)
  - [ ] Each knowledge point has clear evidence source annotation

## [Example]
  **Knowledge Management Architecture**:
  
  1. **Document Types & Naming Convention**:
     - Best Practices: `best-practices-{semantic-topic}.md` (記錄 platinum 級別的開發實踐)
     - Errors: `errors-{semantic-topic}.md` (記錄已知錯誤和快速解決方案)
     - Problem Solving: `problem-solving-{semantic-topic}.md` (記錄複雜問題的完整解決過程)
  
  2. **Semantic Classification Examples**:
     - Best Practices topics: api-design, error-handling, testing-strategy, code-organization, dependency-management
     - Errors topics: authentication, database, api-integration, deployment, configuration
     - Problem Solving topics: performance, scalability, security, data-migration, integration
  
  3. **Document Format Templates**:
  
  **best-practices-{topic}.md**:
  ```markdown
  # {Topic} Best Practices
  
  ## {Practice Title}
  
  **Description**: {簡述最佳實踐}
  
  **Evidence Source**: {evidence_path}
  
  **Applicable Scenarios**: {適用場景}
  
  **Level**: Platinum
  ```
  
  **errors-{topic}.md**:
  ```markdown
  # {Topic} Errors
  
  ## {Error Title}
  
  **Error Type**: {錯誤類型}
  
  **Context**: {發生情境}
  
  **Solution**: {解決方案}
  
  **Source**: {evidence_path}
  ```
  
  **problem-solving-{topic}.md**:
  ```markdown
  # {Topic} Problem Solving
  
  ## {Problem Title} ({YYYY-MM-DD})
  
  **Problem**: {問題描述}
  
  **Investigation**: {調查過程}
  
  **Root Cause**: {根本原因分析}
  
  **Solution**: {解決方案}
  
  **Decision Rationale**: {決策理由}
  
  **Prevention**: {預防措施}
  
  **Source**: {evidence_path}
  ```
  
  4. **Concrete Examples**:
  
  **Example 1: best-practices-{topic}.md**
  ```markdown
  # {Topic} Best Practices
  
  ## {Practice Title A}
  
  **Description**: {具體描述該最佳實踐的內容和價值}
  
  **Evidence Source**: {evidence_path}
  
  **Applicable Scenarios**: {描述該實踐適用的場景和條件}
  
  **Level**: Platinum
  
  ## {Practice Title B}
  
  **Description**: {具體描述該最佳實踐的內容和價值}
  
  **Evidence Source**: {evidence_path}
  
  **Applicable Scenarios**: {描述該實踐適用的場景和條件}
  
  **Level**: Platinum
  ```
  
  **Example 2: errors-{topic}.md**
  ```markdown
  # {Topic} Errors
  
  ## {Error Title A}
  
  **Error Type**: {錯誤分類，如 Security Vulnerability / Logic Error / Configuration Error}
  
  **Context**: {描述錯誤發生的情境和觸發條件}
  
  **Solution**: {描述解決方案的具體步驟或方法}
  
  **Source**: {evidence_path}
  
  ## {Error Title B}
  
  **Error Type**: {錯誤分類}
  
  **Context**: {描述錯誤發生的情境和觸發條件}
  
  **Solution**: {描述解決方案的具體步驟或方法}
  
  **Source**: {evidence_path}
  ```
  
  **Example 3: problem-solving-{topic}.md**
  ```markdown
  # {Topic} Problem Solving
  
  ## {Problem Title} ({YYYY-MM-DD})
  
  **Problem**: {清楚描述遇到的問題現象和影響範圍}
  
  **Investigation**: 
  - {調查步驟1：使用的工具或方法，以及發現的線索}
  - {調查步驟2：進一步的分析過程和結果}
  - {調查步驟3：額外的數據收集或驗證}
  
  **Root Cause**: 
  1. {根本原因1：技術層面的具體原因}
  2. {根本原因2：設計或配置層面的原因}
  
  **Solution**: 
  1. {解決方案1：具體的修復措施}
  2. {解決方案2：相關的配置或代碼調整}
  3. {解決方案3：補充的優化措施}
  
  **Decision Rationale**: 
  - {決策理由1：為什麼選擇這個方案而非其他方案}
  - {決策理由2：該方案的優勢和權衡考量}
  - {決策理由3：預期效果和可接受的代價}
  
  **Prevention**: 
  - {預防措施1：流程或工具層面的改進}
  - {預防措施2：技術實踐的建議}
  - {預防措施3：代碼審查或測試的重點}
  
  **Source**: {evidence_path}
  ```
  
  5. **Classification Guidelines**:
     - 根據內容的語義和主題進行分類，而非技術層或模組
     - 同一 semantic topic 可能跨越多個技術層（如 authentication 可能涉及 frontend, backend, database）
     - 優先選擇對 LLM 和人類都友好的語義分類詞彙
     - 當單個文件內容過多（超過 50 個條目）時，考慮進一步細分語義主題
