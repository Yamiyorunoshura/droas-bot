**GOAL**: Review task development outcomes, ensuring compliance with quality standards and requirements.

## [Input]
  1. "{DEVNOTES}/{task_id}-dev-notes.md" --Development notes
  2. "{PLAN}/{task_id}-plan.md" --Implementation plan
  3. "{TMPL}/review-tmpl.yaml" --Review template

## [Output]
  1. "{REVIEW}/{task_id}-review.md"
  2. "{EPIC}"

## [Constraints]
  1. Do not skip test execution (if tests missing or fail to run, mark as Reject)
  2. Do not omit acceptance decision (Accept/Accept with changes/Reject)
  3. Do not produce review report that deviates from template structure
  4. Do not update epic with incorrect status or score

## [Tools]
  1. **sequential-thinking (MCP)** - Structured reasoning tool for complex logic analysis
    - [Step 1: Identify task domain and select review criteria]
    - [Step 2-4: Reason about domain-specific scoring logic, issue prioritization, and decision rationale]
    - When to use: When need complex code quality assessment or architecture alignment analysis
  2. **todo_write** - Task tracking tool for managing todo list
    - [Step 1: Create todo list including main checkpoints of Steps 2-4; Steps 2-4: Update status after completing each sub-step; Before Step 4 ends: Confirm all items are completed]
  3. **claude-context (MCP)** - Codebase semantic search and indexing tool
    - [Step 1-2: Search codebase for implementation plan-related code (if needed)]
    - [Step 2: Search for relevant test cases and implementation patterns]
  4. **context7 (MCP)**
    - [Step 2: Code and Test Review Phase - Verify external API usage correctness]
    - When to use: When reviewing code that integrates with external services or SDKs

## [Steps]
  1. Review Planning
  - Task: Understand implementation plan and identify task domain
  - Expected outcome: Domain-specific review criteria determined with progress tracking established

  2. Code and Test Review
  - Task: Execute all tests and apply domain-specific scoring criteria
  - Expected outcome: Test results recorded, test coverage and code alignment verified

  3. Development Notes Review
  - Task: Review development notes alignment with implementation
  - Expected outcome: Notes verified against actual implementation

  4. Results Generation and Decision
  - Task: Generate review report and make acceptance decision
  - Expected outcome: Complete review report at "{REVIEW}/{task_id}-review.md" with decision and "{EPIC}" updated

## [Domain-Specific-Review-Guidelines]
  
  ### **Domain-Based Review Process**
  Identify task domain first, then apply appropriate domain dimensions below. Use General Review if domain is unclear.

  ### **Scoring System (All Domains)**
  - **Platinum (4.0)**: Fully compliant, no issues
  - **Gold (3.0)**: Meets standards, 1-2 minor issues
  - **Silver (2.0)**: Minimum standards, 3-4 issues needing improvement
  - **Bronze (1.0)**: Below standards, serious issues or critical gaps
  - Calculate: overall_score = mean of dimension scores, round to 2 decimals

  ### **Decision Rules**
  - **Accept**: All dimensions ≥ 2.0, no critical issues
  - **Accept with Changes**: 1-2 dimensions ≥ 1.5 with clear improvement plan
  - **Reject**: 3+ dimensions < 2.0, or critical security/functional issues
  - **Risk**: Low (all ≥2.5), Medium (1-2 between 2.0-2.4), High (any <2.0 or security issues)

## [DoD]
  - [ ] All tests executed with results recorded and verified against implementation plan
  - [ ] Complete review report exists at "{REVIEW}/{task_id}-review.md" with scoring and acceptance decision
  - [ ] "{EPIC}" updated with task completion status and score

## [Example]

### Example 1: Backend API - User Authentication
[Input]
- Development notes: docs/dev-notes/1-dev-notes.md (Task-1: Implement JWT authentication)
- Implementation plan: docs/plans/1-plan.md (specified JWT, bcrypt, token expiry)
- Template: review-tmpl.yaml

[Decision]
- Domain: Backend (apply backend scoring dimensions)
- Execute tests: npm test (all 15 tests pass, coverage 92%)
- Code alignment: Verify JWT implementation matches plan (src/auth/AuthService.js:L10-L45)
- Score: API Design (4.0), Security (4.0), Error Handling (3.0), Testing (4.0) → Overall 3.75
- Decision: Accept (all dimensions ≥ 2.0, no critical issues)

[Expected Outcome]
- docs/review/1-review.md with test results, code alignment analysis, score 3.75
- docs/epic.md updated: Task-1 marked completed with score
- Acceptance decision: Accept with rationale (strong implementation, minor error handling improvements suggested)

### Example 2: Frontend - Dashboard Widget
[Input]
- Development notes: docs/dev-notes/2-dev-notes.md (Task-2: Build analytics widget)
- Implementation plan: docs/plans/2-plan.md (React component with chart.js)
- Template: review-tmpl.yaml

[Decision]
- Domain: Frontend (apply frontend scoring dimensions)
- Execute tests: npm test (12/15 tests pass, 3 accessibility tests fail)
- Code alignment: Missing responsive design from plan (src/components/AnalyticsWidget.jsx)
- Score: UI/UX (3.0), State Management (4.0), Performance (3.0), Accessibility (2.0), Testing (2.0) → Overall 2.8
- Decision: Accept with changes (accessibility and responsive design fixes needed)

[Expected Outcome]
- docs/review/2-review.md with failed test details (L42-L58), alignment gaps
- Action items: Fix accessibility (aria-labels, keyboard nav), add responsive breakpoints
- docs/epic.md updated with score 2.8, status: needs revision

### Example 3: Database Migration - Schema Update
[Input]
- Development notes: docs/dev-notes/3-dev-notes.md (Task-3: Add user preferences table)
- Implementation plan: docs/plans/3-plan.md (migration script, indexes, foreign keys)
- Template: review-tmpl.yaml

[Decision]
- Domain: Database (apply database scoring dimensions)
- Execute tests: npm run test:db (migration fails rollback test, missing index on user_id)
- Code alignment: Foreign key constraint missing from plan (migrations/003_user_preferences.sql:L15)
- Score: Schema Design (3.0), Indexing (2.0), Migration (1.0), Performance (2.0) → Overall 2.0
- Decision: Reject (migration script has critical issue, rollback fails)

[Expected Outcome]
- docs/review/3-review.md with critical findings: migration rollback failure
- Risk: High (data integrity risk if deployed)
- Action items: Fix rollback script, add missing index, re-test migration
- docs/epic.md: Task-3 status = failed review, requires rework
