**GOAL**: Create feature-level task list (epic), breaking down requirements into executable tasks.

## [Input]
  1. "{REQ}/*.md" --Project functional and non-functional requirements
  2. "{ARCH}/*.md" --Architecture design and technical specifications
  3. "{TMPL}/epic-tmpl.yaml" --Epic template format

## [Output]
  1. "{EPIC}" --Task list (Markdown format)

## [Constraints]
  1. Do not create tasks below feature-level granularity
  2. Do not include operational actions unless explicitly requested
  3. Do not use spaces in file names/paths (use kebab-case)

## [Tools]
  1. **todo_write**
    - [Step 1: Create task list; Steps 2-4: Track task progress]
  2. **sequential-thinking (MCP)**
    - [Step 1: Analyze requirement complexity and task dependencies]
    - [Step 2: Design feature-level tasks with logical grouping]
    - When to use: When need to decompose complex requirements or identify task dependencies

## [Steps]
  1. Research
  - Task: Understand requirements, architecture, and project scope
  - Expected outcome: Success criteria and constraints for task design identified

  2. Drafting
  - Task: Break down requirements into feature-level tasks
  - Expected outcome: Feature-level task breakdown with proper grouping and clear scope

  3. Review
  - Task: Deduplicate and verify tasks
  - Expected outcome: Tasks are actionable with proper granularity and requirement traceability

  4. Finalization
  - Task: Obtain user approval and save epic document
  - Expected outcome: Final epic document saved to "{EPIC}" and approved by user

## [Task-Design-Guidelines]
  1. **Feature-Level Tasks**
    - Each task represents a major feature (e.g., "Implement login", "Add export function")
    - Tasks should be completable within reasonable scope; description ≤ 14 characters
    - Clear and verifiable outcomes with specific DoD
  
  2. **Complete Traceability**
    - Map each task to specific requirement IDs and architecture components
    - Ensure 100% requirement coverage across all tasks
    - Identify and document dependencies between tasks
  
  3. **Results-Oriented**
    - Focus on deliverables and acceptance criteria, not implementation steps
    - Exclude operational actions (git commit, npm install) unless explicitly requested
    - Atomic breakdown will happen later in create-plan phase using TDD cycles

## [DoD]
  - [ ] Epic file exists at "{EPIC}" with all tasks feature-level, outcome-oriented, and properly mapped to requirements
  - [ ] Task list approved by user with 100% requirement coverage
  - [ ] All file names use kebab-case (no spaces)

## [Example]

### Example 1: Data Analytics Pipeline
[Input]
- Requirements: REQ-001 (data ingestion), REQ-002 (transformation), REQ-003 (visualization)
- Architecture: Kafka, Spark, Dashboard Service
- Template: epic-tmpl.yaml

[Decision]
- Task-1: Implement data ingestion (maps to REQ-001, Kafka Consumer)
- Task-2: Implement ETL pipeline (maps to REQ-002, Spark Jobs)
- Task-3: Build analytics dashboard (maps to REQ-003, Dashboard Service)

[Expected Outcome]
- docs/epic.md with 3 feature-level tasks
- Each task has clear DoD and requirement traceability
- Tasks are logically ordered: ingestion → transformation → visualization

### Example 2: Booking System
[Input]
- Requirements: REQ-001 (search availability), REQ-002 (create reservation), REQ-003 (payment processing)
- Architecture: Search API, Booking Service, Payment Gateway
- Template: epic-tmpl.yaml

[Decision]
- Task-1: Implement availability search (REQ-001 → Search API)
- Task-2: Implement booking creation (REQ-002 → Booking Service)
- Task-3: Integrate payment (REQ-003 → Payment Gateway)
- Dependencies: Task-3 depends on Task-2

[Expected Outcome]
- docs/epic.md with tasks ordered by dependencies
- Each task has requirement mapping and architecture component references
- No operational actions (git commit, npm install) included

### Example 3: Inventory Management
[Input]
- Requirements: REQ-001 (product CRUD), REQ-002 (stock tracking), REQ-003 (low stock alerts), REQ-004 (reporting)
- Architecture: Product Service, Inventory Service, Notification Service
- Template: epic-tmpl.yaml

[Decision]
- Task-1: Implement product management (REQ-001 → Product Service)
- Task-2: Implement stock tracking (REQ-002 → Inventory Service)
- Task-3: Build alert system (REQ-003 → Notification Service)
- Task-4: Create reports (REQ-004 → Inventory Service)

[Expected Outcome]
- docs/epic.md with 4 feature-level tasks
- 100% requirement coverage across tasks
- Each task description ≤14 characters, clear and actionable

Format of tasks in the epic with checkboxes:

- [ ] Task-{task_id}
  - {description of the task}
- [ ] Task-{task_id}
  - {description of the task}
- [ ] Task-{task_id}
  - {description of the task}
