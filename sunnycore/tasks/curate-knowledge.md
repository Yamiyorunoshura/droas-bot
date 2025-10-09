**GOAL**: Organize project knowledge accumulated during development, including best practices, lessons learned, and bug fix records.

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

## [Constraints]
  1. Do not include non-platinum practices as best practices (only use marked platinum from reviews)
  2. Do not make independent judgments on practice levels (trust review markings)
  3. Do not force resolution of conflicting practices (preserve all evidence for context-based decisions)

## [Tools]
  1. **sequential-thinking (MCP)**
    - [Step 2: Reason about knowledge base organization structure and semantic classification]
    - When to use: When need to design knowledge categorization scheme or resolve conflicting practices
  2. **claude-context (MCP)**
    - [Step 2: Search codebase for relevant implementations]
    - When to use: When technical details mentioned in documentation need to be verified or supplemented from code
  3. **todo_write**
    - [Step 2: Create task list; Steps 3-4: Track task progress]
  4. **context7 (MCP)**
    - [Step 2: Knowledge Base Design Phase - Verify if practices align with official guidelines]
    - When to use: When validating best practices against official documentation

## [Steps]
  1. Preparation
  - Task: Identify all platinum-level best practices and development errors
  - Expected outcome: Progress tracking established for knowledge curation

  2. Knowledge Base Design
  - Task: Design knowledge base organization and semantic classification
  - Expected outcome: Well-structured organization scheme with proper semantic classification

  3. Document Production
  - Task: Generate knowledge base documents under "{KNOWLEDGE}/"
  - Expected outcome: Complete documents with proper evidence sources and semantic naming

  4. Finalization
  - Task: Archive review and development notes
  - Expected outcome: Documents archived to "{ARCHIVE}/{version_name}/"

## [Knowledge-Curation-Guidelines]
  1. **Platinum-Level Practices Only**
    - Include only platinum-level practices marked in review reports (no self-judgment needed)
    - If none exist, record "No sufficiently validated best practices at this stage" with explanation
    - Each practice must include: title, description, applicable scenarios, evidence source
  
  2. **Error Documentation**
    - Record all development errors with: type, context, solution, prevention
    - Each error case must be reproducible and verifiable
    - Annotate evidence sources (format: "file_path [Section Name]")
  
  3. **Semantic Organization**
    - Classify by semantic topics (e.g., api-design, error-handling, authentication)
    - Use semantic naming: best-practices-{topic}.md, errors-{topic}.md, problem-solving-{topic}.md
    - Ensure clear structure for easy search and maintenance
  
  4. **Conflict Handling**
    - When contradictory practices found, preserve all evidence without forcing choices
    - Annotate conflict points and applicable scenarios for each approach
    - Support context-based decision-making with complete information

## [DoD]
  - [ ] Knowledge base exists at "{KNOWLEDGE}/" with platinum-level best practices and errors documented
  - [ ] All knowledge points have evidence source annotations (format: file_path [Section Name])
  - [ ] Review and development notes archived to "{ARCHIVE}/{version_name}/"

## [Example]

### Example 1: API Development Project
[Input]
- Review reports: docs/review/1-review.md (platinum: JWT rotation pattern), docs/review/3-review.md (platinum: circuit breaker)
- Dev notes: docs/dev-notes/2-dev-notes.md (error: race condition in cache), docs/dev-notes/4-dev-notes.md (error: SQL injection vulnerability)
- Cutover report: docs/cutover-report.md (best practice: API versioning strategy)

[Decision]
- Platinum practices found: 3 (JWT rotation, circuit breaker, API versioning)
- Errors found: 2 (race condition, SQL injection)
- Classify by semantic topics: best-practices-security.md, best-practices-api-design.md, errors-concurrency.md, errors-security.md

[Expected Outcome]
- docs/knowledge/best-practices-security.md (JWT rotation with evidence source: docs/review/1-review.md)
- docs/knowledge/best-practices-api-design.md (circuit breaker, API versioning)
- docs/knowledge/errors-concurrency.md (race condition case with solution)
- docs/knowledge/errors-security.md (SQL injection case with prevention)

### Example 2: Data Pipeline Development
[Input]
- Review reports: docs/review/2-review.md (gold: error handling, not platinum), docs/review/5-review.md (platinum: idempotent data processing)
- Dev notes: docs/dev-notes/1-dev-notes.md (error: memory leak in streaming), docs/dev-notes/3-dev-notes.md (platinum: backpressure handling)
- Progress: docs/progress.md (critical decision: Kafka vs RabbitMQ)

[Decision]
- Platinum practices: 2 (idempotent processing, backpressure handling)
- Gold practices: Excluded (not platinum level)
- Errors: 1 (memory leak)
- Classification: best-practices-data-processing.md, errors-performance.md

[Expected Outcome]
- docs/knowledge/best-practices-data-processing.md (2 entries: idempotent processing from review/5, backpressure handling from dev-notes/3)
- docs/knowledge/errors-performance.md (memory leak case with investigation and solution)
- Each entry has evidence source annotation (file_path [Section Name])

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
