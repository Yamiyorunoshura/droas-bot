## [Input]
  1. User-provided requirement description and context
  2. "{TMPL}/prd-tmpl.yaml" --PRD template
  3. (Conditional) "{ARCH}/*.md" --Existing architecture (if Brownfield)
  4. "{KNOWLEDGE}/*.md" --Project knowledge (if exist)

## [Output]
  1. "{PRD}" --Complete Product Requirements Document

## [Constraints]
  1. Must determine project type (Greenfield/Brownfield) by checking "{ARCH}/" directory existence
  2. Each requirement must be verifiable and measurable; vague or subjective wording is not allowed
  3. Tasks must be feature-level, verifiable (clear functional scope and outcomes)
  4. Must exclude operational actions (e.g., git commit, npm install, deployment script execution) unless explicitly requested by the user
  5. Must ensure all file names/paths do not use spaces; prefer kebab-case
  6. If Brownfield, must preserve existing contracts (i.e., public API interfaces, data models, event formats, and other externally visible contracts) and provide impact analysis for changes

## [Tools]
  1. **todo_write**
    - [Step 1: Create todo list; Steps 2-6: Track working progress]
  2. **sequentialthinking (MCP)**
    - [Step 1: Analyze project type and complexity; Step 2: Design requirements; Step 3: Design architecture]
  3. **context7 (MCP)**
    - [Step 3: Obtain external package and architecture pattern references when needed]
  4. **claude-context (MCP)**
    - [Step 1: Search codebase for existing architecture implementations if Brownfield]

## [Steps]
  1. Initialization and Project Type Detection Phase
    - Read all workflow steps to understand expected work
    - Check if "{ARCH}/" directory exists
    - if directory exists then proceed to 1.1, else proceed to 1.2
      
      1.1. Brownfield Project
        - Read all existing architecture documents from "{ARCH}/*.md"
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

  3. Architecture Design Phase
    - if Brownfield then proceed to 3.1, else proceed to 3.2
      
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

  4. PRD Integration Phase
    - Integrate requirements, architecture, and tasks into PRD template structure
    - Set project-info.type to "greenfield" or "brownfield" based on Step 1 determination
    - Include requirement-to-architecture-to-task traceability
    - Identify requirement dependencies and execution order
    - Add constraints, assumptions, and risks sections

  5. Finalization Phase
    - Cross-verify PRD completeness and consistency
    - Ensure all requirements have corresponding architecture elements
    - Write PRD to "{PRD}" in Markdown format
    - Present the complete PRD to the user and request confirmation or modification suggestions`

## [Error-Handling]
  1. Directory check failure: Log error and proceed as Greenfield
  2. Existing architecture read failure: Record issue and request user clarification
  3. Requirement conflicts: Record conflicts and request user clarification
  4. Requirement conflicts cannot be automatically resolved: Document conflicting requirements and request user prioritization
  5. Architecture design infeasibility: Record technical limitations and propose alternatives
  6. Architecture design incompatible with non-functional requirements: Document incompatibility and request user guidance on trade-offs
  7. User rejects final PRD: Record rejection reasons and iterate on requirements/architecture based on feedback

## [DoD]
  - [ ] Project type (Greenfield/Brownfield) has been determined
  - [ ] If Brownfield, existing architecture has been reviewed
  - [ ] Functional requirements have been extracted, deduplicated, and atomized
  - [ ] Non-functional requirements have been identified and quantified
  - [ ] Each requirement has corresponding acceptance criteria (using Given-When-Then structure)
  - [ ] Architecture design is complete with components, data flows, and technical stack
  - [ ] If Brownfield, impact analysis has been documented
  - [ ] Requirement-to-architecture mapping has been established (100% coverage)
  - [ ] Requirement dependencies have been identified
  - [ ] All requirements are verifiable and outcome-oriented
  - [ ] PRD follows template structure and is saved to "{PRD}"
  - [ ] User confirmation of the final PRD has been obtained

