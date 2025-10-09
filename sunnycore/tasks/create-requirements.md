**GOAL**: Create detailed functional and non-functional requirements documentation.

## [Input]
  1. "{TMPL}/requirement-tmpl.yaml" --Requirement template
  2. User-provided initial ideas and requirement descriptions

## [Output]
  1. "{root}/docs/requirements.md" (temporary requiremenmt file. Will be sharded after running "shard-requirements.py")
  2. "{REQ}/*.md" --Complete requirement specifications (including functional requirements, non-functional requirements, and acceptance criteria)

## [Constraints]
  1. Do not create vague or unmeasurable requirements
  2. Do not deviate from template structure and field names
  3. Do not include sensitive or personal data in examples
  4. Do not skip executing shard-requirements.py

## [Tools]
  1. **todo_write**
    - [Step 1: Create task list; Steps 2-4: Track task progress]
  2. **sequential-thinking (MCP)**
    - [Step 1: Structured decomposition of user stories and system capabilities]
    - [Step 2: Systematically decompose complex functional requirements into atomic requirements]
    - [Step 3: Transform abstract quality attributes into quantifiable non-functional requirements]
    - [Step 4: Validate logical completeness of Given-When-Then acceptance criteria]
  3. **playwright (MCP)**
    - [Step 2: When need to research competitor features or collect UI flow examples]
    - Use scenarios: Crawl competitor website feature flows, error handling approaches, UX design references
  4. **claude-context (MCP)**
    - [Step 3: Search codebase for similar requirement implementations and related modules]
    - Query examples: "How is similar functionality implemented?" "What are the related modules?"

## [Steps]
  1. Initialization
  - Task: Understand workflow and requirements gathering scope
  - Expected outcome: Progress tracking mechanism established

  2. Functional Requirements
  - Task: Extract and organize functional requirements
  - Expected outcome: Complete, deduplicated, atomic functional requirements organized by user stories

  3. Non-Functional Requirements
  - Task: Define quantified non-functional requirements
  - Expected outcome: Measurable requirements across all quality attributes with clear targets

  4. Acceptance Criteria
  - Task: Create testable acceptance criteria for all requirements
  - Expected outcome: All requirements have Given-When-Then acceptance criteria with binary outcomes

  5. Finalization
  - Task: Obtain user approval and shard requirements
  - Expected outcome: Requirements approved and successfully sharded to "{REQ}/" using shard-requirements.py

## [Requirements-Analysis-Guidelines]
  1. **Verifiability First**
    - Each requirement must be testable and measurable with clear success/failure criteria
    - Avoid vague wording; use concrete, quantifiable metrics
  
  2. **Structured Classification**
    - Separate functional (what system does) from non-functional (quality attributes: performance, security, reliability)
    - Use Given-When-Then format for all acceptance criteria
  
  3. **Quantify Quality Attributes**
    - Non-functional requirements need specific metrics (e.g., "P95 latency < 200ms" not "fast response")
    - Transform abstract qualities into monitorable signals
  
  4. **Traceability & Dependencies**
    - Map requirement relationships and priorities
    - Identify preconditions and impact scope for each requirement

## [DoD]
  - [ ] Complete functional and non-functional requirements with quantified metrics and Given-When-Then acceptance criteria
  - [ ] Requirements successfully sharded to "{REQ}/*.md" files using shard-requirements.py
  - [ ] Final requirement set approved by user

## [Example]

### Example 1: E-commerce Product Search
[Input]
User idea: "I want customers to search products by category, price range, and ratings"

[Decision]
- Extract 3 functional requirements: REQ-001 (category filter), REQ-002 (price range filter), REQ-003 (rating filter)
- Define non-functional requirement: NFR-001 (search response time < 500ms)
- Create Given-When-Then acceptance criteria for each

[Expected Outcome]
- docs/requirements/functional.md with REQ-001, REQ-002, REQ-003
- docs/requirements/non-functional.md with NFR-001 (performance target)
- Each requirement has measurable acceptance criteria

### Example 2: Mobile Push Notification System
[Input]
User idea: "Send real-time notifications to users on iOS and Android when events occur"

[Decision]
- Functional requirements: REQ-001 (event subscription), REQ-002 (cross-platform delivery), REQ-003 (notification templates)
- Non-functional: NFR-001 (delivery latency < 2s), NFR-002 (99.9% delivery success rate)
- Acceptance criteria with specific metrics

[Expected Outcome]
- docs/requirements/functional.md with event-based notification requirements
- docs/requirements/non-functional.md with quantified reliability and performance targets
- Verifiable acceptance criteria using Given-When-Then format

### Example 3: Healthcare Patient Record Access
[Input]
User idea: "Doctors need secure access to patient medical history with audit trails"

[Decision]
- Functional: REQ-001 (role-based access), REQ-002 (audit logging), REQ-003 (search patient records)
- Non-functional: NFR-001 (HIPAA compliance), NFR-002 (audit log retention 7 years)
- Security-focused acceptance criteria

[Expected Outcome]
- docs/requirements/functional.md with access control and audit requirements
- docs/requirements/non-functional.md with compliance and data retention policies
- Acceptance criteria aligned with healthcare regulations
