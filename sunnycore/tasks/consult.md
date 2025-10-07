## [Input]
  1. User requirement description
  2. (Conditional) "{ARCH}/*.md" --Existing architecture documents (if exist)

## [Output]
  1. Simple workflow recommendation (which command to use)

## [Constraints]
  1. Must check "{ARCH}/" directory existence to determine project type
  2. If Brownfield, must read and analyze existing architecture documents
  3. Output must be concise and actionable (recommend specific command only)
  4. Must not provide detailed analysis unless user explicitly requests it

## [Tools]
  1. **todo_write**
    - [Step 1: Create task list; Steps 2-4: Track progress]
  2. **sequentialthinking (MCP)**
    - [Step 2: Analyze requirement scope and impact; Step 3: Determine workflow recommendation]
  3. **claude-context (MCP)**
    - [Step 2: Search codebase for existing architecture when Brownfield]

## [Steps]
  1. Initialization Phase
    - Read all workflow steps to understand expected work
    - Create todo list to track subsequent analysis tasks
    - Check if "{ARCH}/" directory exists
    - if directory exists then proceed to 1.1, else proceed to 1.2
      
      1.1. Brownfield Project Detection
        - Read all existing architecture documents from "{ARCH}/*.md"
        - Index existing components, modules, and technical stack
        - Note current system boundaries and extension points
      
      1.2. Greenfield Project Detection
        - Note that this is a new project with no existing architecture

  2. Requirement Analysis Phase
    - Extract key aspects from user requirement description:
      - Does it introduce new system components or modules?
      - Does it change core architecture patterns or technology stack?
      - Does it require new external integrations or services?
      - Does it modify existing component boundaries or responsibilities?
    - if Brownfield then proceed to 2.1, else proceed to 2.2
      
      2.1. Brownfield Requirement Analysis
        - Compare requirement against existing architecture
        - Assess if requirement fits within existing component boundaries
        - Determine if requirement needs new modules or architectural changes
      
      2.2. Greenfield Requirement Analysis
        - Assess requirement complexity and scope
        - Determine if requirement needs full architecture design

  3. Recommendation Phase
    - Based on analysis, determine workflow type
    - if requirement needs architectural changes or new modules then proceed to 3.1, else proceed to 3.2
      
      3.1. Recommend Full Development Workflow
        - Output recommendation: "Based on your requirement, I recommend using the **full development workflow** starting with `*create-requirements`"
        - Briefly explain: "Your requirement involves [architectural changes/new modules/new integrations], which requires comprehensive architecture design and planning."
        - if Brownfield then also mention: "Since this is a Brownfield project, please ensure architecture documents are up-to-date first using `/sunnycore_architect *document-project`"
      
      3.2. Recommend PRD Workflow
        - Output recommendation: "Based on your requirement, I recommend using the **PRD workflow** with `*create-prd`"
        - Briefly explain: "Your requirement appears to be a modification within existing architecture boundaries, suitable for the streamlined PRD approach."

  4. Finalization Phase
    - Present workflow recommendation to user
    - Provide next step command to execute
    - if user requests more details then provide detailed analysis, else complete

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

## [DoD]
  - [ ] Project type (Greenfield/Brownfield) has been determined
  - [ ] If Brownfield, existing architecture documents have been reviewed
  - [ ] Requirement scope and impact have been analyzed
  - [ ] Clear workflow recommendation has been provided
  - [ ] Next step command has been specified
  - [ ] User understands which workflow to use and why

