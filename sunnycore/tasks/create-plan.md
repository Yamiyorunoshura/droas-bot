**GOAL**: Create detailed TDD implementation plans for all tasks in epic, including RED/GREEN/REFACTOR three phases for each task.

## [Input]
  1. "{REQ}/*.md" --Project requirements
  2. "{ARCH}/*.md" --Architecture design
  3. "{EPIC}" --Feature-level task list
  4. "{TMPL}/implementation-plan-tmpl.yaml" --Implementation plan template (including: project information, requirement mapping, architecture reference, RED/GREEN/REFACTOR three phases, etc.)
  5. "{KNOWLEDGE}/*.md" --Project knowledge base (best practices, lessons learned, bug fixes)

## [Output]
  1. "{PLAN}/{task_id}-plan.md" --Implementation plans for all tasks (Markdown format)
    - Format: Use ATX headings; numbered lists; explicit requirement/architecture mapping sections
    - Content: Breaks down feature-level tasks into atomic implementation steps across TDD RED/GREEN/REFACTOR phases
    - Example: "{PLAN}/1-plan.md", "{PLAN}/2-plan.md", "{PLAN}/3-plan.md", etc.

## [Constraints]
  1. Do not fabricate requirements or tasks not in source documents
  2. Do not create plans without complete requirement-architecture mapping
  3. Do not proceed if conflicts or missing files are found (must record and request clarification)
  4. Do not ignore knowledge base best practices when planning

## [Tools]
  1. **todo_write**
    - [Step 1 (Setup Phase): Create todo list; Steps 2-4: Track task progress]
  2. **sequential-thinking (MCP)**
    - [Step 1: Analyze requirements and task complexity]
    - [Step 2: Plan RED phase - test and acceptance criteria design]
    - [Step 3: Plan GREEN phase - minimal implementation step design]
    - [Step 4: Plan REFACTOR phase - refactoring and optimization work identification]
    - When to use: When need to decompose complex tasks or design TDD workflow
  3. **claude-context (MCP)**
    - [Step 1: Search for relevant implementation references (if needed)]
  4. **context7 (MCP)**
    - [Step 3: Plan GREEN phase - When need to query official API usage and examples for external packages]
    - When to use: When implementation involves external SDK integration or need to verify package compatibility

## [Steps]
  1. Setup
  - Task: Understand all requirements, architecture, and task scope
  - Expected outcome: Project-specific best practices and lessons learned identified

  2. Plan RED Phase (For Each Task)
  - Task: Plan test cases with acceptance criteria and test conditions
  - Expected outcome: Complete test planning with measurable success metrics and edge cases defined

  3. Plan GREEN Phase (For Each Task)
  - Task: Design atomic implementation steps for minimal code
  - Expected outcome: Executable steps mapped to architecture components and test conditions

  4. Plan REFACTOR Phase (For Each Task)
  - Task: Plan refactoring and optimization work
  - Expected outcome: Comprehensive improvement plan with cross-cutting concerns and quality improvements identified

  5. Finalization
  - Task: Generate all implementation plan files
  - Expected outcome: Complete plans for all tasks at "{PLAN}/{task_id}-plan.md" following template structure

## [TDD-Planning-Guidelines]
  1. **TDD Three-Phase Structure**
    - **RED Phase**: Define test cases from acceptance criteria; plan tests that will fail initially (normal, edge cases, error handling)
    - **GREEN Phase**: Plan minimal implementation to make tests pass; follow architecture mapping
    - **REFACTOR Phase**: Plan quality improvements (patterns, DRY, performance); plan integration with real APIs/services if needed
  
  2. **Atomic Step Breakdown**
    - Break feature-level tasks from epic into atomic, executable steps within each TDD phase
    - Each step must be minimal and directly traceable to acceptance criteria
    - Map steps to specific architecture components and requirement IDs
  
  3. **Leverage Knowledge Base**
    - Apply documented best practices from knowledge base
    - Avoid repeated mistakes by referencing error patterns
    - Incorporate lessons learned into planning
  
  4. **Traceability & Dependencies**
    - Ensure three-way mapping: Task ↔ Requirement ↔ Architecture
    - Identify execution dependencies and parallel opportunities
    - Exclude operational actions (git, npm) unless explicitly requested

## [DoD]
  - [ ] Implementation plan exists for each task with complete TDD three-phase structure (RED/GREEN/REFACTOR sections)
  - [ ] All plans have proper requirement-architecture-test mapping with specific file references
  - [ ] All plan files created at "{PLAN}/{task_id}-plan.md" following template structure

## [Example]

### Example 1: Content Management System - Article Publishing
[Input]
- Task-1 from epic.md: "Implement article publishing"
- Requirements: REQ-001 (create/edit articles), acceptance criteria in Given-When-Then format
- Architecture: Article Service (Express.js), PostgreSQL
- Template: implementation-plan-tmpl.yaml

[Decision]
- RED phase: Write tests for article CRUD operations (create, read, update)
- GREEN phase: Implement minimal Article model, API endpoints (POST /articles, GET /articles/:id, PUT /articles/:id)
- REFACTOR phase: Add validation, error handling, apply repository pattern

[Expected Outcome]
- docs/plans/1-plan.md with TDD three-phase structure
- RED: Test cases for happy path, edge cases (empty title, invalid ID)
- GREEN: Step-by-step implementation mapped to src/services/ArticleService.js
- REFACTOR: Planned improvements (DRY, SOLID principles)

### Example 2: Financial Dashboard - Transaction Report
[Input]
- Task-2 from epic.md: "Build transaction report"
- Requirements: REQ-002 (aggregate transactions by date range), NFR-001 (query < 2s)
- Architecture: Reporting Service, TimescaleDB
- Knowledge base: best-practices-database-queries.md

[Decision]
- RED phase: Write tests for date range filtering, aggregation accuracy, performance (< 2s)
- GREEN phase: Implement query with indexed date column, aggregation logic
- REFACTOR phase: Add caching (Redis), query optimization using knowledge base insights

[Expected Outcome]
- docs/plans/2-plan.md following template structure
- RED: Performance test asserting query time < 2s
- GREEN: Minimal SQL queries and API implementation
- REFACTOR: Caching strategy, index optimization from knowledge base

### Example 3: IoT Platform - Device Registration
[Input]
- Task-3 from epic.md: "Implement device registration"
- Requirements: REQ-003 (register device with unique ID), REQ-004 (device metadata storage)
- Architecture: Device Registry (gRPC), PostgreSQL
- Template: implementation-plan-tmpl.yaml

[Decision]
- RED phase: Tests for device registration, duplicate ID rejection, metadata validation
- GREEN phase: gRPC service stub, database insert with unique constraint
- REFACTOR phase: Add device status tracking, integrate with authentication service

[Expected Outcome]
- docs/plans/3-plan.md with complete TDD cycle
- RED: Unit tests and integration tests for gRPC endpoint
- GREEN: Atomic steps mapped to proto files and database schema
- REFACTOR: Cross-cutting concerns (auth, logging) integration
