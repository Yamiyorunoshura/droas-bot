## [Input]
  1. "{DEVNOTES}/{task_id}-dev-notes.md" --Development notes
  2. "{PLAN}/{task_id}-plan.md" --Implementation plan
  3. "{TMPL}/review-tmpl.yaml" --Review template

## [Output]
  1. "{REVIEW}/{task_id}-review.md"
  2. "{EPIC}"

## [Constraints]
  1. Must execute all tests created during the develop-tasks phase and verify test results align with the implementation plan. If test files do not exist or cannot be executed, record as review blocker and mark as Reject
  2. Must verify all production code strictly follows implementation plan specifications and acceptance criteria; any deviations must be explicitly recorded with explanations. "Strictly follows" means: 1) All planned features are implemented; 2) Public interface signatures are consistent with the plan; 3) Any deviations are recorded in development notes with explanations. Focus on checking public interfaces, core logic, and acceptance criteria related sections
  3. Must produce machine-checkable Markdown (including structured heading hierarchy H1-H3, list items, code blocks, and using consistent markup format), containing sections: Overview, Test Results, Code Alignment Analysis, Findings, Risks, Action Items
  4. Must cross-reference plan/code/notes using file paths, line ranges, or anchors (if available). Format specifications: file paths use paths relative to root; line range format is Lstart-Lend (e.g., L10-L20); anchor format is #anchor-id
  5. Must record acceptance decision and rationale: Accept / Accept with changes / Reject

## [Tools]
  1. **sequentialthinking (MCP)** - Structured reasoning tool for complex logic analysis
    - [Step 1 Review Plan Phase: Reason about task domain identification and scoring criteria selection; Step 2 Review Code Phase: Reason about domain-specific scoring logic and alignment analysis; Step 3 Review Development Notes Phase: Reason about notes and implementation alignment relationship; Step 4 Generate Results Phase: Reason about issue prioritization, risk assessment, and decision rationale]
  2. **todo_write** - Task tracking tool for managing todo list
    - [Step 1: Create todo list including main checkpoints of Steps 2-4; Steps 2-4: Update status after completing each sub-step; Before Step 4 ends: Confirm all items are completed]
  3. **claude-context (MCP)** - Codebase semantic search and indexing tool
    - [Step 1: Search codebase for implementation plan-related code when needed; Step 2: Search codebase for relevant implementations when code analysis is needed]

## [Steps]
  1. Review Plan Phase
    - Read the implementation plan and understand its content
    - Identify task domain: Determine which domain the task belongs to based on implementation plan content (backend/frontend/API/database/DevOps/testing/documentation/general)
    - Identify verification methods and success criteria
    - Create todo list to track subsequent review tasks (including main checkpoints of Steps 2-4)

  2. Review Code Phase
    - Read all production code and understand its implementation
    - Apply domain-specific review criteria: Based on the task domain identified in Step 1, review according to that domain's scoring dimensions
    - Execute all tests and record pass/fail status and alignment with plan (if test count >100 or estimated execution time >30 minutes, prioritize critical path tests and affected module tests)
    - if all tests pass then proceed to 2.1, else proceed to 2.2
      
      2.1. Tests Pass Path
        - Execute integration tests to confirm implementation does not affect existing code (scope: interface tests between new/modified modules and existing system, and end-to-end tests of critical business flows)
        - Verify test coverage
        - Verify code strictly aligns with architecture/design and acceptance criteria
        - Record domain-specific scores: Score each domain dimension and calculate overall score
      
      2.2. Tests Fail Path
        - Record failed test details (test name, error message, expected vs actual)
        - Mark review result as Reject or Accept with changes
        - Explicitly list necessary fix items
        - Do not execute subsequent coverage and alignment verification

  3. Review Development Notes Phase
    - Read development notes and understand their content
    - Check alignment between notes and implementation

  4. Generate Results Phase
    - Create review results using template, including test execution summary
    - Record test results and pass/fail status and alignment with plan; analyze code alignment and specific references
    - Save to "{REVIEW}/{task_id}-review.md"; if file exists, update it
    - if review result is Accept then proceed to 4.1, else if review result is Accept with changes then proceed to 4.2, else proceed to 4.3
      
      4.1. Accept Path
        - Update "{EPIC}" to mark task as completed
        - Record completion time and final score
      
      4.2. Accept with Changes Path
        - Record improvement suggestions and priorities
        - Update "{EPIC}" and note items to be improved
        - Mark task as "conditionally completed"
      
      4.3. Reject Path
        - Generate detailed review report
        - Do not update "{EPIC}"
        - Wait for brownfield-tasks to fix then review again

## [DoD]
  - [ ] Task domain has been identified and corresponding domain-specific review criteria have been applied
  - [ ] Scoring has been performed according to domain-specific scoring dimensions
  - [ ] All tests have been executed and results recorded
  - [ ] Integration tests have been executed to confirm implementation does not affect existing code and results recorded
  - [ ] Test results have been verified to align with the plan
  - [ ] Code alignment analysis is complete, including specific references to deviations from the plan
  - [ ] All necessary sections are present: Overview, Test Results, Code Alignment Analysis, Findings, Risks, Action Items
  - [ ] "{EPIC}" has been updated with completion status and score
  - [ ] Test failures and plan misalignments have been clearly identified and prioritized
  - [ ] Acceptance decision has been recorded with rationale based on test results and plan adherence
