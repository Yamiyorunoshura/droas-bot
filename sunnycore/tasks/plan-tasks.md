## [Input]
  1. "{REQ}/*.md" --Project requirements
  2. "{ARCH}/*.md" --Architecture design
  3. "{EPIC}" --Feature-level task list
  4. "{TMPL}/implementation-plan-tmpl.yaml" --Implementation plan template (including: project information, requirement mapping, architecture reference, RED/GREEN/REFACTOR three phases, etc.)
  5. "{KNOWLEDGE}/*.md" --Project knowledge (if exist)

## [Output]
  1. "{PLAN}/{task_id}-plan.md" --Implementation plan (Markdown format)
    - Format: Use ATX headings; numbered lists; explicit requirement/architecture mapping sections
    - Content: Breaks down feature-level tasks into atomic implementation steps across TDD RED/GREEN/REFACTOR phases
    - Example: "{PLAN}/1-plan.md"

## [Constraints]
  1. Must strictly extract tasks from provided documents; do not fabricate requirements
  2. Must map each plan item to requirement ID and architecture section
  3. Must use Markdown format (ATX headings and numbered lists)
  4. Must produce exactly one file at the specified output path
  5. The produced implementation plan must ensure logical coherence of TDD three phases (refer to [Development-Guidelines] section for TDD methodology)
  6. If requirement-architecture conflicts, unclear requirements, or missing necessary files are found, must record issues and request user clarification, do not make assumptions or skip
  7. Must break down feature-level tasks (from epic.md) into atomic, executable steps within each TDD phase
  8. Each atomic step in GREEN phase should be minimal and directly traceable to a specific acceptance criterion in RED phase

## [Tools]
  1. **todo_write**
    - [Step 1 (Setup Phase): Create todo list; Steps 2-4: Track task progress]
  2. **sequentialthinking (MCP)**
    - [Step 1: Analyze requirement and task complexity; Step 2: Plan RED phase content (test and acceptance criteria design); Step 3: Plan GREEN phase content (minimal implementation step design); Step 4: Plan REFACTOR phase content (refactoring and optimization work identification)]
  3. **claude-context (MCP)**
    - [Step 1: Search codebase for relevant implementations if needed]

## [Steps]
  1. Setup Phase
    - Read all workflow steps and requirement documents
    - Understand the scope and complexity of the target feature-level task (identify number of requirements, architecture components, and cross-system dependencies)
    - Assess complexity level (simple/medium/complex) based on requirement count and architecture component count
    - Determine the depth of subsequent planning based on complexity (note: input task is at feature-level and will be broken down into atomic implementation steps)
    - Create todo list to track execution progress of Steps 2-5

  2. Planning RED Phase Content: Define Tests and Acceptance
    - Write the "RED Phase" section: break down the feature-level task into specific acceptance criteria and test conditions
    - Map requirements to testable outcomes and verification methods
    - Define measurable success metrics, failure conditions, and edge cases

  3. Planning GREEN Phase Content: Minimal Implementation Steps
    - Write the "GREEN Phase" section: break down the feature into atomic, minimal implementation steps
    - Map each acceptance criterion to specific architecture components, files, and concrete development tasks
    - Each implementation step should be atomic, executable, and traceable to test conditions
    - Steps should be granular enough for straightforward implementation (e.g., "Create User model", "Add login endpoint", "Implement password validation")

  4. Planning REFACTOR Phase Content: Refactoring and Optimization Steps
    - Write the "REFACTOR Phase" section: list refactoring and optimization work after GREEN phase completion
    - Identify and integrate duplicate code, cross-cutting concerns (authentication, logging, error handling, etc.)
    - Plan performance optimizations, code quality improvements, and architecture enhancement steps

  5. Finalization Phase
    - Integrate content from Steps 2-4 into a complete implementation plan document
    - Format the plan using the template: reference template structure to organize content ensuring all necessary sections are included (requirement mapping, architecture reference, RED/GREEN/REFACTOR three phases, etc.)
    - Verify completeness and executability of the plan
    - Generate Markdown plan to "{PLAN}/{task_id}-plan.md"

## [DoD]
  - [ ] All requirement, architecture, and task documents have been read
  - [ ] Plan document includes TDD three-phase structure (RED/GREEN/REFACTOR sections)
  - [ ] RED section: Each requirement has corresponding acceptance criteria and test conditions
  - [ ] GREEN section: All implementation steps correspond to specific acceptance criteria and include architecture/file references
  - [ ] REFACTOR section: Refactoring and optimization work has been planned, including cross-cutting concerns integration
  - [ ] Plan follows TDD cycle structure: test-first (RED), minimal implementation (GREEN), refactoring optimization (REFACTOR)
  - [ ] Output path and file naming follow specified pattern
  - [ ] "{PLAN}/{task_id}-plan.md" has been created
