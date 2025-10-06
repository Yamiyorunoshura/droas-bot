[Input]
  1. "{root}/docs/implementation-plan/{task_id}-plan.md" --Implementation plan
  2. "{root}/docs/architecture/*.md" --Architecture design
  3. "{root}/sunnycore/templates/dev-notes-tmpl.yaml" --Development notes template

[Output]
  1. "{root}/docs/dev-notes/{task_id}-dev-notes.md" --Complete development notes document
  2. High-quality, tested code implementation
  3. Complete test coverage and test cases
  4. Refactored code following best practices

[Constraints]
  1. Must comply with acceptance criteria and architecture mapping defined in the implementation plan
  2. Must follow TDD cycle: implement tests first (RED), minimal code (GREEN), then refactor (REFACTOR)
  3. Development notes must preserve the indentation and numbering style used in the template
  4. Allowed to modify modules specified in the implementation plan and their test files, as well as necessary configuration files; if modification of files outside the plan is needed (referring to files not explicitly listed for modification in the implementation plan, including but not limited to: shared utility classes, configuration files, dependency injection settings, etc.), must record in development notes and explain the reason
  5. Development notes must include: implementation summary, technical decisions, risk considerations, test results, and other complete sections (refer to template)

[Tools]
  1. **sequentialthinking (MCP)**
    - [Step 1: Analyze implementation plan complexity; Steps 2-4: Reason about implementation strategies for each TDD phase; Step 5: Evaluate validation results]
  2. **todo_write**
    - [Step 1: Create TDD phase todo list; Steps 2-5: Track implementation progress for each phase]
  3. **claude-context (MCP)**
    - [Step 1: Process large implementation plans in segments]

[Steps]
  1. Preparation Phase
    - Read the TDD-based implementation plan produced in the plan-tasks phase
    - Extract the implementation steps for RED, GREEN, REFACTOR three phases
    - Create todo list based on implementation steps

  2. RED Phase: Implement Tests
    - Read the RED phase in the implementation plan and obtain the test cases to be implemented
    - Implement all test cases and check if all are red as expected
    - Execute test suite (such as pytest/npm test) to confirm all newly added test cases have status FAILED and error messages match expectations
    - if all new tests fail and error messages match expectations then 2.1, else 2.2
      2.1. Tests fail correctly
        - Enter GREEN phase
      2.2. Tests partially pass or no errors
        - Check if test implementation is correct
        - Fix test cases
        - Re-execute validation
        - Repeat this process until all tests fail correctly

  3. GREEN Phase: Minimal Implementation
    - Implement minimal code to make tests pass (GREEN phase)
    - Follow architecture mapping specified in the implementation plan
    - Execute all tests
    - if all tests pass and acceptance criteria are met then 3.1, else 3.2
      3.1. All tests pass
        - Enter REFACTOR phase
      3.2. Some tests fail
        - Analyze failure reasons
        - Fix code
        - Re-execute tests
        - Repeat this process until tests pass

  4. REFACTOR Phase: Refactoring and Optimization
    - Perform refactoring while keeping all tests green (REFACTOR phase)
    - Apply planned optimizations and cross-cutting concerns without reducing coverage
    - Improve code quality and maintainability to meet quality goals
    - Execute tests after each refactoring
    - if all tests still pass then 4.1, else 4.2
      4.1. Tests pass after refactoring
        - Continue with next refactoring
        - Or enter validation and documentation phase
      4.2. Tests fail after refactoring
        - Immediately rollback refactoring (git reset or manual undo)
        - Re-evaluate refactoring strategy
        - Fix refactoring approach
        - Repeat this process until refactoring is complete and tests pass

  5. Validation and Documentation Phase
    - Validate implementation against all acceptance criteria and planned test conditions
    - Generate development notes according to template structure, including implementation summary, technical decisions, risk considerations, test results, and other sections
    - Generate document to "{root}/docs/dev-notes/{task_id}-dev-notes.md"
    - Check all DoD items one by one to ensure they are met
    - Confirm all todo items are completed

[DoD]
  - [ ] All acceptance criteria in the implementation plan have been met
  - [ ] Tests cover all verification methods specified in the plan
  - [ ] Implementation follows TDD cycle: RED → GREEN → REFACTOR
  - [ ] Code corresponds to planned architecture components
  - [ ] All outputs in [Output] have been generated and are consistent
  - [ ] All todo items are completed
