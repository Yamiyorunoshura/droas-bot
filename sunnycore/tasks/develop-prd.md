[Input]
  1. "{root}/docs/PRD.md" --Product Requirements Document
  2. "{root}/sunnycore/templates/dev-notes-tmpl.yaml" --Development notes template

[Output]
  1. "{root}/docs/dev-notes/prd-dev-notes.md" --Complete development notes document
  2. High-quality, tested code implementation for all PRD tasks
  3. Complete test coverage and test cases

[Constraints]
  1. Must complete all tasks defined in the PRD in a single execution
  2. Must follow TDD cycle for each task: implement tests first (RED), minimal code (GREEN), then refactor (REFACTOR)
  3. Development notes must preserve the indentation and numbering style used in the template
  4. Must comply with acceptance criteria and architecture mapping defined in the PRD
  5. If modification of files outside the PRD scope is needed, must record in development notes and explain the reason

[Tools]
  1. **todo_write**
    - [Step 1: Create task list from PRD; Steps 2-4: Track implementation progress]
  2. **sequentialthinking (MCP)**
    - [Step 1: Analyze PRD complexity; Steps 2-4: Reason about implementation strategies for each task]
  3. **claude-context (MCP)**
    - [Step 1: Process large PRD documents in segments]

[Steps]
  1. Preparation Phase
    - Read "{root}/docs/PRD.md" completely
    - Extract all tasks from the PRD tasks section
    - Understand requirements, architecture, and task dependencies
    - Create todo list based on all tasks, respecting dependency order
    - Plan execution sequence considering task dependencies

  2. Iterative TDD Development Phase
    - For each task in dependency order:
      
      2.1. RED Phase: Implement Tests
        - Read task's acceptance criteria and requirements
        - Implement all test cases for the task
        - Execute test suite to confirm all newly added test cases have status FAILED
        - Verify error messages match expectations
        - if all new tests fail correctly then proceed to 2.2, else fix tests and retry
      
      2.2. GREEN Phase: Minimal Implementation
        - Implement minimal code to make tests pass
        - Follow architecture mapping specified in the PRD
        - Execute all tests
        - if all tests pass then proceed to 2.3, else fix code and retry
      
      2.3. REFACTOR Phase: Refactoring and Optimization
        - Perform refactoring while keeping all tests green
        - Apply best practices and improve code quality
        - Execute tests after each refactoring
        - if tests fail after refactoring then rollback and re-evaluate
        - if all tests pass then mark task completed and proceed to next task
    
    - Repeat for all tasks until all PRD tasks are completed

  3. Integration Testing Phase
    - Execute complete test suite for all implemented tasks
    - Verify all acceptance criteria from PRD are met
    - Test integration points between tasks
    - Ensure all functional and non-functional requirements are satisfied
    - if any tests fail then identify failing task and return to Step 2 for that task

  4. Documentation Phase
    - Generate comprehensive development notes according to template structure
    - Include implementation summary for all tasks
    - Document technical decisions made during development
    - Record any deviations from PRD and rationale
    - Document risk considerations and mitigation strategies
    - Include test results and coverage metrics
    - Generate document to "{root}/docs/dev-notes/prd-dev-notes.md"

  5. Finalization Phase
    - Cross-verify implementation against all PRD acceptance criteria
    - Ensure all requirements have been satisfied
    - Verify code quality meets standards (SOLID principles, DRY, etc.)
    - Confirm test coverage meets minimum requirements (≥80%)
    - Check all DoD items one by one to ensure they are met
    - Confirm all todo items are completed

[Error-Handling]
  1. PRD file not found: Report error and halt execution
  2. Test failures in RED phase that don't fail correctly: Fix test logic and retry
  3. Test failures in GREEN phase: Analyze failure, fix implementation, retry
  4. Test failures in REFACTOR phase: Rollback refactoring, re-evaluate strategy
  5. Integration test failures: Identify root cause, fix specific task, re-run integration tests

[DoD]
  - [ ] PRD has been read and all tasks extracted
  - [ ] Todo list has been created with all tasks in dependency order
  - [ ] All tasks have been implemented following TDD cycle (RED → GREEN → REFACTOR)
  - [ ] All tests pass for all tasks
  - [ ] All acceptance criteria in the PRD have been met
  - [ ] Integration testing completed successfully
  - [ ] Code quality meets standards (SOLID, DRY, readability)
  - [ ] Test coverage meets minimum requirements (≥80%)
  - [ ] Development notes document has been generated
  - [ ] All outputs in [Output] have been generated and are consistent
  - [ ] All todo items are completed

