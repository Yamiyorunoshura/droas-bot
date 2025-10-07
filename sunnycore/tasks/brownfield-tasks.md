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
  1. Must ensure fixed code runs properly
  2. Must ensure fixed code complies with architecture design
  3. Must ensure fixed code does not break existing functionality

## [Tools]
  1. **todo_write**
    - [Steps 2-4: Track task progress] (see Tool Guidelines for details)
  2. **sequentialthinking (MCP)**
    - [Step 1: All reasoning tasks; Step 2: Code fix reasoning tasks]
  3. **claude-context (MCP)**
    - [Step 1: Search codebase for relevant implementations]
    - [Step 2: Search codebase for fix-related code]

## [Steps]
  1. Preparation Phase
    - Read review report and understand the issues
    - Read architecture design to align with overall direction
    - Read development notes and actual code to think about where the issues are
    - Combine architecture and code to formulate atomic fix tasks

  2. Fix Phase
    - Create todo list to track fix progress (update according to state gate, see Tool Guidelines for details)
    - After each code fix, execute tests to ensure no new issues are introduced (e.g., pytest -q or make test; all must pass)
    - if tests pass (exit code=0) then proceed to 2.1, else proceed to 2.2
      
      2.1. Tests Pass Path
        - Continue with next fix task
        - Or enter integration testing
      
      2.2. Tests Fail Path
        - Use git reset or manually undo recent changes
        - Re-analyze failure reasons
        - Fix and re-execute tests
        - Repeat this process until tests pass
    - After completing all fixes, execute integration tests (such as: pytest tests/integration, make test-integration, or integration test commands for other programming languages or projects; all tests must also pass)

  3. Summary Phase
    - Summarize the fix process and results (should include document paths and line numbers/paragraph IDs as evidence; for version control projects, should include PR/commit links; corresponding to the evidence field of "Fix Summary" for auditing purposes)
    - Write or update the summary to the corresponding development notes according to the template

## [DoD]
  - [ ] All unit tests have passed
  - [ ] All integration tests have passed
  - [ ] Fix summary has been generated and includes changes/tests/evidence/risk/rollback
  - [ ] Fixed code complies with architecture design
  - [ ] Development notes have been updated and generated
