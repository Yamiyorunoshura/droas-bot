## [Input]
  1. "{REQ}/*.md" --Project functional and non-functional requirements
  2. "{ARCH}/*.md" --Architecture design and technical specifications
  3. "{TMPL}/tasks-tmpl.yaml" --Task template format

## [Output]
  1. "{EPIC}" --Task list (Markdown format)

## [Constraints]
  1. Must create feature-level tasks representing major features within modules (e.g., "Implement login functionality", "Implement registration functionality")
  2. Tasks should be verifiable at the feature level with clear functional scope and outcomes
  3. Must exclude operational actions unless explicitly requested by the user
    - Operational actions refer to execution-level commands such as git commit, npm install, etc., but design, implementation, testing and other development tasks do not fall into this category
  4. Must ensure all file names/paths do not use spaces; prefer kebab-case
  5. Note: Atomic breakdown of tasks will be handled later in the plan-tasks phase using TDD RED/GREEN/REFACTOR cycles

## [Tools]
  1. **todo_write**
    - [Step 1: Create task list; Steps 2-4: Track task progress]
  2. **sequentialthinking (MCP)**
    - [Step 1: Analyze requirement complexity and task dependencies; Step 2: Design atomic tasks and logical grouping]

## [Steps]
  1. Research Phase
    - Verify all necessary input files (including templates) exist and are readable. If reading fails, stop responding and explicitly notify the user of missing file paths
    - Read requirement and architecture source documents
    - Identify scope, success criteria, and constraints to drive task design
    - Map non-functional requirements to cross-cutting tasks
    - Create todo list to track subsequent task design work

  2. Drafting Phase
    - Use template to generate feature-level tasks
    - Each task should represent a major feature within a module
    - Include brief acceptance hints to ensure verifiability at feature level
    - Logically group tasks by module or feature area, avoid overlap
    - Deliverable: Task draft in Markdown format

  3. Review Phase
    - Deduplicate and remove non-actionable items
    - Ensure each task represents a feature-level scope, not overly atomized
    - Ensure each task is traceable to requirements
    - Verify format complies with template requirements

  4. Finalization Phase
    - Prepare draft task list in Markdown format with brief introduction explaining grouping and scope
    - Present draft content to user showing all task groups and individual tasks
    - if user approves draft then proceed to 4.1, else proceed to 4.2
      
      4.1. Write Final Document
        - Write approved task list to "{EPIC}" (Markdown format)
      
      4.2. Revise Based on Feedback
        - Collect user feedback on what needs to be changed (e.g., task granularity, grouping, acceptance criteria)
        - Revise the task list according to feedback
        - Return to present revised draft and request approval again

## [DoD]
  - [ ] All requirement and architecture documents have been read
  - [ ] Research notes are complete, including mapping of functional requirements/non-functional requirements to tasks
  - [ ] User approval has been obtained for task list draft
  - [ ] File "{EPIC}" exists and is valid Markdown
  - [ ] All tasks comply with template fields
  - [ ] Each task is feature-level, outcome-oriented, and verifiable
  - [ ] No file names or key names use spaces; kebab-case is enforced
