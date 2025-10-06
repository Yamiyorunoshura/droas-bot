[Input]
  1. User-provided requirement description and context
  2. "{root}/sunnycore/templates/prd-tmpl.yaml" --PRD template
  3. (Conditional) "{root}/docs/architecture/*.md" --Existing architecture (if Brownfield)
  4. "{root}/docs/knowledge/*.md" --Project knowledge (if exist)

[Output]
  1. "{root}/docs/PRD.md" --Complete Product Requirements Document

[Constraints]
  1. Must determine project type (Greenfield/Brownfield) by checking "{root}/docs/architecture/" directory existence
  2. Each requirement must be verifiable and measurable; vague or subjective wording is not allowed
  3. Tasks must be atomic, verifiable (≤50 characters for task names, clear outcomes)
  4. Must exclude operational actions (e.g., git commit, npm install, deployment script execution) unless explicitly requested by the user
  5. Must ensure all file names/paths do not use spaces; prefer kebab-case
  6. If Brownfield, must preserve existing contracts (i.e., public API interfaces, data models, event formats, and other externally visible contracts) and provide impact analysis for changes

[Tools]
  1. **todo_write**
    - [Step 1: Create task list; Steps 2-6: Track task progress]
  2. **sequentialthinking (MCP)**
    - [Step 1: Analyze project type and complexity; Step 2: Design requirements; Step 3: Design architecture; Step 4: Design tasks]
  3. **context7 (MCP)**
    - [Step 3: Obtain external package and architecture pattern references when needed]
  4. **claude-context (MCP)**
    - [Step 1: Process large existing architecture documents if Brownfield]

[Steps]
  1. Initialization and Project Type Detection Phase
    - Read all workflow steps to understand expected work
    - Check if "{root}/docs/architecture/" directory exists
    - If the directory exists, execute the Brownfield flow (1.1); otherwise, execute the Greenfield flow (1.2)
      1.1. Brownfield Project
        - Read all existing architecture documents from "{root}/docs/architecture/*.md"
        - Identify extension points, constraints, and shared services
        - Create todo list for Brownfield PRD creation
      1.2. Greenfield Project
        - No existing architecture to consider
        - Create todo list for Greenfield PRD creation

  2. Requirements Phase
    - Extract functional requirements from user input and context
    - Deduplicate and atomize statements (single testable condition)
    - Identify non-functional requirements across performance, reliability, security, compliance
    - Quantify targets (e.g., P95 latency, uptime SLO) and constraints
    - Define acceptance criteria for each requirement using Given-When-Then structure
    - Cross-check and request user confirmation

  3. Architecture Design Phase
    - if Brownfield then 3.1, else 3.2
      3.1. Brownfield Architecture Design
        - Define new module responsibilities, boundaries, and interfaces
        - Specify data flows and interaction methods with existing components
        - Assess non-functional requirements compatibility
        - Write "Impact Analysis" for all proposed changes
        - Ensure compatibility with existing contracts
      3.2. Greenfield Architecture Design
        - Define components, boundaries, and specify data flows based on requirements
        - Ensure each requirement maps to an architecture element
        - Define interaction contracts and data schemas
        - Record architecture decisions and rationale
        - Handle cross-cutting concerns (security, observability, performance)

  4. Task Generation Phase
    - Generate atomic, verifiable tasks based on requirements and architecture
    - Ensure each task conforms to the constraints defined in [Constraints] (atomic, verifiable, ≤50 characters for task names, clear outcomes)
    - Include brief acceptance hints to ensure verifiability
    - Map each task to specific requirement IDs and architecture components
    - Identify task dependencies and execution order
    - Logically group tasks, avoid overlap
    - Exclude operational actions (git commit, npm install, etc.) unless explicitly requested

  5. PRD Integration Phase
    - Integrate requirements, architecture, and tasks into PRD template structure
    - Set project-info.type to "greenfield" or "brownfield" based on Step 1 determination
    - Include requirement-to-architecture-to-task traceability
    - Add constraints, assumptions, and risks sections
    - Ensure all sections are complete and consistent

  6. Finalization Phase
    - Cross-verify PRD completeness and consistency
    - Ensure all requirements have corresponding architecture elements and tasks
    - Verify task set completely covers all requirements
    - Write PRD to "{root}/docs/PRD.md" in Markdown format
    - Present the complete PRD to the user and request confirmation or modification suggestions
    - Check all DoD items one by one to ensure they are met
    - Confirm all todo items are completed

[Error-Handling]
  1. Directory check failure: Log error and proceed as Greenfield
  2. Existing architecture read failure: Record issue and request user clarification
  3. Requirement conflicts: Record conflicts and request user clarification
  4. Requirement conflicts cannot be automatically resolved: Document conflicting requirements and request user prioritization
  5. Architecture design infeasibility: Record technical limitations and propose alternatives
  6. Architecture design incompatible with non-functional requirements: Document incompatibility and request user guidance on trade-offs
  7. Task generation coverage insufficient (<80% requirements): Identify missing requirements and regenerate tasks
  8. User rejects final PRD: Record rejection reasons and iterate on requirements/architecture/tasks based on feedback

[DoD]
  - [ ] Project type (Greenfield/Brownfield) has been determined
  - [ ] If Brownfield, existing architecture has been reviewed
  - [ ] Functional requirements have been extracted, deduplicated, and atomized
  - [ ] Non-functional requirements have been identified and quantified
  - [ ] Each requirement has corresponding acceptance criteria (using Given-When-Then structure)
  - [ ] Architecture design is complete with components, data flows, and technical stack
  - [ ] If Brownfield, impact analysis has been documented
  - [ ] Requirement-to-architecture mapping has been established (100% coverage)
  - [ ] Atomic tasks have been generated with requirement and architecture traceability
  - [ ] Task dependencies have been identified
  - [ ] All tasks are verifiable and outcome-oriented
  - [ ] PRD follows template structure and is saved to "{root}/docs/PRD.md"
  - [ ] User confirmation of the final PRD has been obtained
  - [ ] All todo items are completed

