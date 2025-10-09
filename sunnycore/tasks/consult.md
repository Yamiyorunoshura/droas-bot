**GOAL**: Analyze requirements and recommend the most suitable development workflow (full development workflow or PRD workflow).

## [Input]
  1. User requirement description
  2. (Conditional) "{ARCH}/*.md" --Existing architecture documents (if exist)

## [Output]
  1. Simple workflow recommendation (which command to use)

## [Constraints]
  1. Do not misidentify project type (Greenfield/Brownfield)
  2. Do not skip existing architecture analysis for Brownfield projects
  3. Do not provide verbose analysis unless explicitly requested
  4. Do not recommend workflows that violate Decision-Criteria

## [Tools]
  1. **todo_write**
    - [Step 1: Create task list; Steps 2-4: Track progress]
  2. **sequential-thinking (MCP)**
    - [Step 2: Systematically decompose requirement scope and impact surface]
    - When to use: When need to assess requirement impact on existing system and estimate task complexity
    - [Step 3: Branch reasoning for PRD workflow vs Full workflow applicability]
    - When to use: When requirement scope is unclear or difficult to determine if architecture changes are needed
  3. **claude-context (MCP)**
    - [Step 2: When project is Brownfield, search existing architecture boundaries and public contracts]
    - Query examples: "What are the public API interfaces?" "Where are the integration points?" "How are module boundaries defined?"
  4. **context7 (MCP)**
    - [Step 2: Requirement Analysis Phase - Query API documentation when assessing technical feasibility]
    - When to use: When need to verify if external APIs support required functionality
  5. **playwright (MCP)**
    - [Step 2: Requirement Analysis Phase - Research competitor product implementations]
    - When to use: When need to study how similar features are implemented in existing products

## [Steps]
  1. Initialization
  - Task: Determine project type (Greenfield/Brownfield) and gather context
  - Expected outcome: Project type correctly identified with appropriate context

  2. Requirement Analysis
  - Task: Understand requirement scope and architectural impact
  - Expected outcome: Requirement complexity and workflow depth assessed

  3. Recommendation
  - Task: Provide clear workflow recommendation based on analysis
  - Expected outcome: Workflow recommendation with rationale based on decision criteria

  4. Finalization
  - Task: Provide concise, actionable recommendation with next step command
  - Expected outcome: User understands which workflow to use and why

## [Decision-Criteria]
  **Recommend Full Workflow (*create-requirements) when:**
  - Introduces new system components or modules
  - Changes core architecture patterns (e.g., monolith to microservices)
  - Requires new technology stack or frameworks
  - Adds new external integrations or third-party services
  - Modifies component boundaries or responsibilities
  - Significant cross-cutting concerns (security, observability architecture)
  - Large scope (estimated 5+ tasks)

  **Recommend PRD Workflow (*create-prd) when:**
  - Modifications within existing component boundaries
  - Feature enhancements using existing architecture
  - Bug fixes or technical improvements
  - UI/UX changes without backend architecture changes
  - Small to medium scope (estimated 1-5 tasks)
  - Does not introduce new modules or architectural patterns

## [Workflow-Analysis-Guidelines]
  1. **Scope Assessment**
    - Analyze if requirement introduces new components/modules or changes architecture patterns
    - Estimate task count and complexity (1-5 tasks = PRD; 5+ tasks = Full workflow)
  
  2. **Architecture Impact**
    - For Brownfield: check if existing contracts/boundaries are affected
    - Identify if new technology stack, integrations, or cross-cutting concerns are needed
  
  3. **Decision Criteria** (use Decision-Criteria section above for specifics)
    - Full workflow: architectural changes, new modules, 5+ tasks
    - PRD workflow: within existing boundaries, 1-5 tasks, no new patterns

## [DoD]
  - [ ] Project type correctly determined with requirement scope and impact analyzed
  - [ ] Clear workflow recommendation provided with specific command (*create-requirements or *create-prd)
  - [ ] User understands workflow recommendation and rationale

## [Example]

### Example 1: Add Search Feature to Existing App
[Input]
- User requirement: "Add product search functionality with filters"
- Project type: Brownfield (docs/architecture/ exists with API Gateway, Product Service)

[Decision]
- Scope: Small (within existing Product Service boundaries, no new modules)
- Impact: No architecture changes (uses existing API, adds new endpoint)
- Estimated tasks: 2-3 (search endpoint, filter logic, UI integration)
- Recommendation: PRD workflow (small scope, existing architecture sufficient)

[Expected Outcome]
- Recommendation: "Use `/sunnycore_pm *create-prd` - This is a small feature addition within existing architecture (2-3 tasks estimated)"
- Rationale: No new components needed, fits within current Product Service

### Example 2: Migrate to Microservices Architecture
[Input]
- User requirement: "Break monolith into microservices for better scalability"
- Project type: Brownfield (docs/architecture/ shows monolithic architecture)

[Decision]
- Scope: Large (fundamental architecture change, multiple new services)
- Impact: Major (changes system boundaries, adds service mesh, new deployment model)
- Estimated tasks: 10+ (service extraction, API gateway, data migration, etc.)
- Recommendation: Full workflow (requires comprehensive architecture redesign)

[Expected Outcome]
- Recommendation: "Use `/sunnycore_pm *create-requirements` - This requires architecture redesign (microservices pattern, 10+ tasks estimated)"
- Rationale: Introduces new architectural patterns and multiple new components

