## [Input]
  1. "{PRD}" --Product Requirements Document
  2. "{TMPL}/dev-notes-tmpl.yaml" --Development notes template

## [Output]
  1. "{root}/docs/prd-dev-notes.md" --Complete development notes document
  2. High-quality, tested code implementation for all PRD tasks
  3. Complete test coverage and test cases

## [Constraints]
  1. Must complete all tasks defined in the PRD in a single execution
  2. **Must use TDD methodology for development**: Follow the TDD practice guidelines detailed in [Development-Guidelines] section
  3. Development notes must preserve the indentation and numbering style used in the template
  4. Must comply with acceptance criteria and architecture mapping defined in the PRD
  5. If modification of files outside the PRD scope is needed, must record in development notes and explain the reason

## [Tools]
  1. **todo_write**
    - [Step 1: Create task list from PRD; Step 2: Track TDD phase progress; Steps 3-4: Track implementation progress]
  2. **sequentialthinking (MCP)**
    - [Step 1: Analyze PRD complexity; Step 2-4: Reason about implementation strategies for each requirement]
  3. **claude-context (MCP)**
    - [Step 1: Search codebase to prepare for PRD implementation - understand existing code structure, related modules, dependencies, and implementation patterns]

## [Steps]
  1. Preparation Phase
    - Read "{PRD}" completely
    - Understand requirements and architecture
    - Plan the TDD implementation strategy to fulfil all the funcitonla and non-functional requirements:
      * Identify test cases needed (unit tests, integration tests)
      * Determine test structure and test data requirements
      * Plan implementation approach following architecture mapping
    - Create todo list based on all tasks, respecting dependency order
    - Plan execution sequence considering task dependencies

  2. TDD Development Phase (Batch RED → GREEN → REFACTOR for all requirements)
    - RED Phase: Write test cases for all requirements
      * Read acceptance criteria for all requirements
      * Write all test cases (unit tests, integration tests) following architecture mapping in the PRD
      * Verify all tests fail as expected
    - GREEN Phase: Implement all functionality
      * Implement code for all requirements in dependency order following architecture mapping in the PRD
      * Ensure all tests pass
    - REFACTOR Phase: Refactor all implementations
      * Apply code quality standards (SOLID, DRY) to all implementations
      * Re-run all tests to verify refactoring
      * Mark all implementations completed when tests pass

  3. Integration Testing Phase
    - Execute complete test suite for all implementations
    - Verify all acceptance criteria from PRD are met
    - Test integration points between implementations
    - Ensure all functional and non-functional requirements are satisfied
    - if any tests fail then identify failing implementation and return to Step 2 for that implementation

  4. Documentation Phase
    - Generate comprehensive development notes according to template structure
    - Include implementation summary for all implementations
    - Document technical decisions made during development
    - Record any deviations from PRD and rationale
    - Document risk considerations and mitigation strategies

  5. Finalization Phase
    - Cross-verify implementation against all PRD acceptance criteria
    - Ensure all requirements have been satisfied
    - Verify code quality meets standards (SOLID principles, DRY, etc.)
    - Confirm test coverage meets minimum requirements (≥80%)

## [Error-Handling]
  1. PRD file not found: Report error and halt execution
  2. Test failures in RED phase that don't fail correctly: Fix test logic and retry
  3. Test failures in GREEN phase: Analyze failure, fix implementation, retry
  4. Test failures in REFACTOR phase: Rollback refactoring, re-evaluate strategy
  5. Integration test failures: Identify root cause, fix specific implementation, re-run integration tests

## [DoD]
  - [ ] PRD has been read and all requirements extracted
  - [ ] Todo list has been created with all requirements in dependency order
  - [ ] All requirements have been implemented following TDD cycle (RED → GREEN → REFACTOR)
  - [ ] All tests pass for all implementations
  - [ ] All acceptance criteria in the PRD have been met
  - [ ] Integration testing completed successfully
  - [ ] Code quality meets standards (SOLID, DRY, readability)
  - [ ] Test coverage meets minimum requirements (≥80%)
  - [ ] Development notes document has been generated
  - [ ] All outputs in [Output] have been generated and are consistent

