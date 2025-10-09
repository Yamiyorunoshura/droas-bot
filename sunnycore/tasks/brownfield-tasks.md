**GOAL**: Re-develop tasks that failed review based on review feedback.

## [Input]
  1. "{TMPL}/dev-notes-tmpl.yaml" --Development notes template
  2. "{DEVNOTES}/{task_id}-dev-notes.md" --Development notes
  3. "{REVIEW}/{task_id}-review.md" --Review report
  4. "{ARCH}/*.md" --Architecture design
  5. "{KNOWLEDGE}/*.md" --Project knowledge (if exist)

## [Output]
  1. Fixed code that runs properly
  2. Fix summary (recommend presenting in MARKDOWN, including: changes, tests, evidence, risk, rollback; suggested format, can be adjusted according to project needs)
  3. Updated development notes "{DEVNOTES}/{task_id}-dev-notes.md"

## [Constraints]
  1. Do not deliver fixes that fail to run properly
  2. Do not deviate from architecture design specifications
  3. Do not break existing functionality during fixes

## [Tools]
  1. **todo_write**
    - [Steps 2-4: Track task progress]
  2. **sequential-thinking (MCP)**
    - [Step 1: Analyze problems and architecture context]
    - [Step 2: Reason about code fix strategies]
  3. **claude-context (MCP)**
    - [Step 1: Search existing implementations to understand codebase structure]
    - [Step 2: Search fix-related code]
    - Query examples: "What is the existing implementation?" "How to integrate with existing system?"
  4. **context7 (MCP)**
    - [Step 2: Fix Phase - When fixes involve external API calls or integration issues]
    - When to use: When need to verify correct API usage or find official troubleshooting guides

## [Steps]
  1. Preparation
  - Task: Understand issues from review report and architecture context
  - Expected outcome: Atomic fix tasks formulated based on analysis

  2. Fix Implementation
  - Task: Implement fixes following TDD cycle (RED → GREEN → REFACTOR)
  - Expected outcome: All unit tests and integration tests passing

  3. Summary
  - Task: Generate fix summary and update development notes
  - Expected outcome: Comprehensive fix summary with evidence and updated dev notes

## [Development-Guidelines]
  1. **TDD Practice (Mandatory)**
    - **RED Phase**: Write failing tests from acceptance criteria before any code; verify tests fail for the right reason
    - **GREEN Phase**: Implement minimal code to pass tests (exit code 0); follow architecture mapping strictly
    - **REFACTOR Phase**: Improve code quality while maintaining green tests; integrate real APIs/services; apply patterns and eliminate duplication
    - Iterate RED→GREEN→REFACTOR until all acceptance criteria met; rollback immediately if tests fail during refactoring
  
  2. **Code Quality Standards**
    - Apply SOLID principles (Single Responsibility, Open-Closed, Dependency Inversion)
    - Use meaningful names; keep functions ≤50 lines; avoid duplication (DRY)
    - Statically typed languages must compile successfully
  
  3. **Testing Requirements**
    - Minimum 80% coverage; critical logic requires 100%
    - Cover unit, integration, and E2E levels appropriately
    - Execute full test suite after every change; rollback on failures (exit code ≠ 0)
  
  4. **Documentation & Risk Management**
    - Record technical decisions, deviations, and rationale in dev notes
    - Link requirement IDs and architecture references
    - Identify risks (technical, dependency, timeline); document mitigation and rollback strategies

## [DoD]
  - [ ] All unit and integration tests passed
  - [ ] Fix summary generated with changes/tests/evidence/risk/rollback sections
  - [ ] Development notes updated

## [Example]

### Example 1: Failed Authentication Logic
[Input]
- Review report: docs/review/1-review.md (Reject: token validation fails for expired tokens)
- Dev notes: docs/dev-notes/1-dev-notes.md (original implementation)
- Architecture: JWT authentication with Redis token store

[Decision]
- Issue: Token expiry check missing (test_expired_token_rejection failed)
- Fix: Add token expiry validation before Redis lookup
- TDD: RED (write test for expiry check) → GREEN (add expiry logic) → REFACTOR (optimize Redis queries)

[Expected Outcome]
- Fixed code: src/auth/TokenValidator.js with expiry check (L25-L30)
- Fix summary: Changes (added expiry validation), Tests (test_expired_token_rejection now passes), Evidence (all auth tests pass), Risk (low), Rollback (revert commit abc123)
- Updated docs/dev-notes/1-dev-notes.md with fix details

### Example 2: Performance Issue in Query
[Input]
- Review report: docs/review/3-review.md (Accept with changes: NFR-001 violated, query takes 3s instead of < 500ms)
- Dev notes: docs/dev-notes/3-dev-notes.md (missing database index)
- Architecture: PostgreSQL with search functionality

[Decision]
- Issue: Missing index on search column (performance test failed)
- Fix: Add index on products.name column, optimize query
- TDD: RED (performance test < 500ms fails) → GREEN (add index, query passes) → REFACTOR (add query explain logging)

[Expected Outcome]
- Fixed code: migrations/add_product_name_index.sql, src/repositories/ProductRepository.js (optimized query)
- Fix summary: Changes (added index, rewrote query), Tests (performance test now passes at 120ms), Evidence (test output shows timing), Risk (medium - index creation on large table), Rollback (drop index migration)
- Updated docs/dev-notes/3-dev-notes.md
