**GOAL**: Fix issues identified during acceptance phase.

## [Input]
  1. "{CUTOVER}" --Cutover report (required)
  2. (Conditional) "{PRD}" --Product Requirements Document (if exists, used as primary requirement and architecture source)
  3. (Conditional) "{REQ}/*.md" --Requirement documents (optional, used if "PRD.md" does not exist)
  4. (Conditional) "{ARCH}/*.md" --Architecture documents (optional, used if "PRD.md" does not exist)
  5. "{TMPL}/dev-notes-tmpl.yaml" --Development notes template (required)

## [Output]
  1. "{root}/docs/cutover-fixes-dev-notes.md" --Development notes for fixes
  2. Fixed code implementation
  3. Updated documentation if needed

## [Constraints]
  1. Do not leave any cutover issues unaddressed
  2. Do not skip TDD cycle for fixes (RED → GREEN → REFACTOR)
  3. Do not skip re-running acceptance tests after fixes
  4. Do not introduce new issues or break existing functionality

## [Tools]
  1. **sequential-thinking (MCP)**: Perform structured reasoning and analysis
    - [Step 1: Structured analysis of problem root causes]
    - [Step 2: Reason about fix strategies and evaluate multiple approaches]
    - When to use: When problems are complex or need to evaluate different fix approaches
  2. **todo_write**: Create and manage task list
    - [Step 1: Create todo list; Steps 2-7: Track task progress]
  3. **claude-context (MCP)**: Search codebase for relevant code
    - [Step 2: Search code and dependencies related to failing functionality]
    - Query examples: "Where is the failing functionality?" "What are the dependencies?"
  4. **context7 (MCP)**
    - [Step 2-6: When fixes involve external service integration or configuration]
    - When to use: When need official documentation for service configuration or API troubleshooting

## [Steps]
  1. Preparation
  - Task: Understand and prioritize all reported issues from cutover report
  - Expected outcome: Issues prioritized by severity and business impact

  2. Root Cause Analysis
  - Task: Analyze root causes for all issues
  - Expected outcome: Complete root cause analysis with documented fix strategies

  3. Fix Planning
  - Task: Create comprehensive fix plan for all issues
  - Expected outcome: Fix plan complete with risk assessment and component identification

  4. RED Phase - Test First
  - Task: Write/update tests to reproduce all issues
  - Expected outcome: Failing tests properly reproducing all issues (RED status confirmed)

  5. GREEN Phase - Implement Fixes
  - Task: Implement minimal fixes to pass all tests
  - Expected outcome: All tests passing with fixes following architecture patterns

  6. REFACTOR Phase - Improve and Verify
  - Task: Improve code quality and re-run acceptance tests
  - Expected outcome: Improved code quality with all acceptance tests confirming issues resolved

  7. Documentation
  - Task: Generate complete development notes
  - Expected outcome: Development notes at "{root}/docs/cutover-fixes-dev-notes.md" with all fixes documented

## [Fix-Development-Guidelines]
  1. **TDD Fix Cycle (Mandatory)**
    - **RED Phase**: Write/update tests to reproduce each reported issue; verify tests fail correctly
    - **GREEN Phase**: Implement minimal fixes to pass tests (exit code 0); follow architecture patterns
    - **REFACTOR Phase**: Improve code quality while maintaining green tests; ensure no regressions
    - Rollback immediately if tests fail during refactoring; re-run acceptance tests after all fixes
  
  2. **Root Cause Analysis First**
    - Identify root cause for each issue before implementing fixes
    - Document fix strategy and risk assessment
    - Prioritize fixes by severity and business impact
  
  3. **Testing & Verification**
    - Minimum 80% coverage; critical logic requires 100%
    - Execute full test suite after every change; rollback on failures
    - Re-run all acceptance tests to confirm issues resolved
    - Ensure no new issues or regressions introduced

## [DoD]
  - [ ] All issues resolved through full TDD cycle (RED → GREEN → REFACTOR) with all tests passing
  - [ ] Acceptance tests re-executed with all issues confirmed resolved
  - [ ] Complete development notes at "{root}/docs/cutover-fixes-dev-notes.md"

## [Example]

### Example 1: Push Notification Failure
[Input]
- Cutover report: docs/cutover-report.md (Critical: push notifications fail, missing FCM server key)
- PRD: docs/PRD.md (REQ-002: push notification delivery)
- Template: dev-notes-tmpl.yaml

[Decision]
- Root cause: Firebase config incomplete (no server key in .env)
- Fix strategy: Add FCM_SERVER_KEY to .env, update notification service
- RED: Write test for notification delivery (test_send_notification fails)
- GREEN: Add FCM config, implement send logic (test passes)
- REFACTOR: Add error handling, retry mechanism

[Expected Outcome]
- Fixed code: src/services/NotificationService.js with FCM integration, .env.example with FCM_SERVER_KEY
- docs/cutover-fixes-dev-notes.md documenting root cause, fix, test results
- Re-run acceptance test: Push notifications now work (✓)

### Example 2: Data Export Timeout
[Input]
- Cutover report: docs/cutover-report.md (High: CSV export times out for > 1000 rows)
- Requirements: docs/requirements/functional.md (REQ-003: export up to 10K rows)
- Architecture: docs/architecture/components.md (Export Service with in-memory processing)
- Template: dev-notes-tmpl.yaml

[Decision]
- Root cause: In-memory processing causes timeout for large datasets
- Fix strategy: Implement streaming export instead of loading all data
- RED: Write test for 10K row export with < 30s timeout (test fails)
- GREEN: Implement streaming CSV writer (test passes at 12s)
- REFACTOR: Add progress indicator, optimize database query

[Expected Outcome]
- Fixed code: src/services/ExportService.js with streaming implementation
- docs/cutover-fixes-dev-notes.md with performance comparison (before: timeout, after: 12s)
- Re-run acceptance: Export 10K rows successfully (✓)

