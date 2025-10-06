[Input]
  1. "{root}/docs/requirements" --Standardized requirement source
  2. "{root}/docs/architecture/*.md" --Existing architecture document collection
  3. "{root}/sunnycore/scripts/shard-architecture.py" --Architecture sharding script
  4. "{root}/sunnycore/templates/architecture-tmpl.yaml" --Architecture template
  5. "{root}/docs/knowledge/*.md" --Project knowledge

[Output]
  1. "{root}/docs/architecture/*.md" --Updated architecture document collection (*.md format)

[Constraints]
  1. Must thoroughly review "{root}/docs/requirements" and "{root}/docs/architecture/*.md" before proposing design (evidence: reference reviewed content in architecture draft using `Reference: {file path}#{section heading}` format)
  2. Must preserve existing contracts; any proposed changes must include explicit "Impact Analysis" subsection
  3. Must fully comply with "{root}/sunnycore/templates/architecture-tmpl.yaml" structure and section order
  4. Must draft to "{root}/docs/architecture.md", then execute: uv run "{root}/sunnycore/scripts/shard-architecture.py" to perform sharding
  5. Should use clear, concise English with 2-space indentation throughout
  6. Architecture design required external API call must use context7 (MCP) to search for library documentation and API references

[Tools]
  1. **todo_write**
    - [Step 1: Track tasks; Steps 2-4: Track task status]
  2. **sequentialthinking (MCP)**
    - [Step 1: Evaluate existing architecture]
    - [Step 2: Design module boundaries and integration patterns]
    - [Step 3: Structured drafting]
  3. **claude-context (MCP)**
    - [Step 1: Evaluate existing architecture - handling large document collections]

[Steps]
  1. Evaluate Existing Architecture Phase
    - Review current architecture under "{root}/docs/architecture/*.md"
    - Identify extension points, constraints, and shared services
    - Map affected domains, bounded contexts, and dependencies
    - Create todo list to track subsequent design and writing tasks

  2. Design New Module Phase
    - Define responsibilities, boundaries, and interfaces of new module
    - Specify data flows and interaction methods with existing components
    - Assess non-functional requirements (security, observability, performance) and compatibility
    - Write "Impact Analysis" for all proposed changes to explain potential impact on existing system

  3. Writing and Sharding Phase
    - Use architecture template to draft "{root}/docs/architecture.md" content in Markdown format
    - Ensure sections emphasize new module and integration impact
    - Present draft content to user showing key sections (new module design, integration points, impact analysis)
    - if user approves draft then proceed to 3.1, else proceed to 3.2
      
      3.1. Write Final Documents
        - Write approved content to "{root}/docs/architecture.md"
        - Execute sharding script: uv run "{root}/sunnycore/scripts/shard-architecture.py"
        - Verify documents appear under "{root}/docs/architecture/"
        - if execution succeeds then proceed to Step 4, else check format compliance and re-execute
      
      3.2. Revise Based on Feedback
        - Collect user feedback on what needs to be changed
        - Revise the draft content according to feedback
        - Return to present revised draft and request approval again

  4. Finalization Phase
    - Cross-check against constraints and guidance questions; fix gaps and inconsistencies
    - Confirm all impact analyses are included and complete
    - Verify compatibility between new and old modules
    - Check all DoD items one by one to ensure they are met
    - Confirm all todo items are completed

[DoD]
  - [ ] "{root}/docs/requirements" and "{root}/docs/architecture/*.md" have been thoroughly reviewed
  - [ ] Extension points, constraints, and affected domains have been identified
  - [ ] New module has documented boundaries, interfaces, and data flows
  - [ ] All proposed changes include explicit "Impact Analysis" subsections
  - [ ] Compatibility with existing contracts has been clarified (no breaking changes)
  - [ ] User approval has been obtained for architecture draft
  - [ ] "{root}/docs/architecture.md" exists and follows the template
  - [ ] "shard-architecture.py" has been executed and shard file generation has been verified
  - [ ] All todo items are completed
