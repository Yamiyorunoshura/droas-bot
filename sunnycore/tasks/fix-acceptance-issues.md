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
  1. Must address all issues documented in cutover report
  2. Must follow TDD cycle for fixes: write/update tests first (RED), implement fix (GREEN), then refactor (REFACTOR)
  3. Must verify fixes resolve the reported issues
  4. Development notes must preserve the indentation and numbering style used in the template
  5. Must re-run acceptance tests after fixes to ensure issues are resolved
  6. Must not introduce new issues or break existing functionality

## [Tools]
  1. **sequentialthinking (MCP)**: Perform structured reasoning and analysis
    - [Step 2: Analyze root causes; Step 3: Plan fix strategies; Step 6: Evaluate fix results]
  2. **todo_write**: Create and manage task list
    - [Step 1: Create todo list; Steps 2-7: Track task progress]
  3. **claude-context (MCP)**: Search codebase for relevant code
    - [Step 2: Locate code related to issues]

## [Steps]
  1. Preparation Phase
    - Read cutover report to understand all reported issues
    - Verify existence of required input files
    - Prioritize issues by severity and business impact
    - Create todo list based on issues to fix

  2. Root Cause Analysis Phase
    - Check if "{PRD}" exists
    - For each reported issue, analyze error messages and reproduction steps
    - Locate relevant code using search tools and identify root cause of the issue
    - Document analysis findings and determine appropriate fix strategy for each issue
    - if "PRD.md" exists then proceed to 2.1, else proceed to 2.2
      
      2.1. PRD-based Project
        - Read "{PRD}"
        - Extract requirements from PRD requirements section
        - Extract architecture information from PRD architecture section
        - Use PRD as the primary context for understanding system design
      
      2.2. Traditional Project Structure
        - Read requirements from "{REQ}/*.md" (if available)
        - Read architecture from "{ARCH}/*.md" (if available)

  3. Fix Planning Phase
    - Plan fix approach for each issue
    - Identify which components need to be modified
    - Determine if new tests are needed
    - Assess risk of fixes and potential side effects
    - Document planned changes

  4. RED Phase: Test First
    - For each issue, write or update tests to reproduce the problem
    - Ensure tests fail as expected, verifying the issue exists
    - Execute test suite to confirm RED status
    - If tests don't fail, refine tests until they properly catch the issue

  5. GREEN Phase: Implement Fixes
    - Implement minimal code changes to fix each issue
    - Follow architecture patterns and coding standards
    - Execute tests to verify fixes work
    - If tests still fail, analyze and adjust fix
    - Repeat until all tests pass

  6. REFACTOR Phase: Improve and Verify
    - Refactor code while keeping tests green
    - Improve code quality and maintainability
    - Execute full test suite to ensure no regressions
    - Re-run acceptance tests from cutover to verify issues are resolved
    - If any tests fail, rollback and reassess

  7. Documentation Phase
    - Generate development notes according to template structure
    - Document each issue fixed with before/after comparison
    - Document technical decisions and rationale
    - Record any risks or follow-up items
    - Save to "{root}/docs/dev-notes/cutover-fixes-dev-notes.md"

## [DoD]
  - [ ] Cutover report has been read and all issues identified
  - [ ] Root cause analysis completed for all issues
  - [ ] Fix strategy planned for each issue
  - [ ] Tests written/updated to catch all issues (RED phase)
  - [ ] All issues fixed and tests pass (GREEN phase)
  - [ ] Code refactored and quality improved (REFACTOR phase)
  - [ ] Full test suite passes with no regressions
  - [ ] Acceptance tests re-run and all issues resolved
  - [ ] Development notes generated with complete documentation

