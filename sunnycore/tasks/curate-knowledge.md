[Input]
  1. "{root}/docs/review-results/*.md" --Review reports
  2. "{root}/docs/dev-notes/*.md" --Development notes
  3. "{root}/docs/knowledge/*.md" --Existing knowledge base(if exist)
  4. "{root}/docs/cutover.md" --Cutover report (required)
  5. "{root}/docs/cutover-dev-notes.md" --Cutover development notes (required)
  6. "{root}/docs/progress.md" --Progress record

[Output]
  1. "{root}/docs/knowledge/*.md" --Knowledge base (create directory first if it does not exist)
    - Document organization: May produce "best-practices.md", "errors.md" based on actual content, or subdivide by domain/type
    - Content allocation: One entry per platinum practice, one entry per error case

[Rules]
  1. Must identify all practices and errors during development, but only define platinum-level practices as best practices and produce them
  2. Platinum level will be marked in review reports, only need to read marks without making own judgment
  3. If no platinum-level practices are found, should record "No sufficiently validated best practices in this phase" in the knowledge base and explain the reason
  4. If contradictory practice recommendations are found, should annotate conflicts and preserve all evidence sources for subsequent decision-making

[Tools]
  1. **sequentialthinking (MCP)**
    - [Step 2: Reason about knowledge base organization structure]
  2. **claude-context (MCP)**
    - [Step 2: Find relevant code]
    - Usage scenario: When technical details mentioned in documentation need to be verified or supplemented from code
  3. **todo_write**
    - [Step 2: Create task list; Steps 3-4: Track task progress]
    - Usage scenario: Create todo list in preparation phase, track task progress

[Steps]
  1. Preparation Phase
    - Read all development notes and review reports
    - Identify best practices marked as platinum level and record in temporary list
    - Identify all errors encountered during development process (including error type, occurrence context, solution) and record in temporary list
    - Create todo list to track subsequent knowledge base conception and production tasks

  2. Knowledge Base Conception Phase
    - Conceive the structure and classification method of project knowledge base (reference: classify by technical domain/error type/development phase, choose based on actual content. Consideration factors include: content volume, technical domain distribution, error type diversity, etc. Technical domain classification examples: API design/error handling/testing strategy, etc.)
    - Decide organization structure for best practices and errors

  3. Produce Documents Phase
    - First create "{root}/docs/knowledge/" directory (if it does not exist)
    - Produce best practices to corresponding documents by classification (document naming: "best-practices-{domain}.md" or "best-practices.md")
    - Produce error cases to corresponding documents by classification (document naming: "errors-{type}.md" or "errors.md")
    - Document content format: Each knowledge point includes title, description, evidence source, applicable scenarios
    - Annotate evidence source for each knowledge point (annotation format: file path + relevant section, e.g., "docs/dev-notes/feature-x.md" [Error Handling] paragraph)
    - Archive the "review-results/" and "dev-notes/" files to "{root}/docs/archive/{version_name}/"

  4. DoD Verification Phase
    - Check all DoD items one by one to ensure they are met
    - Confirm all todo items are completed

[DoD]
  - [ ] All files in "{root}/docs/review-results/" and "{root}/docs/dev-notes/" have been read
  - [ ] Knowledge base structure and classification method have been conceived
  - [ ] Knowledge base directory "{root}/docs/knowledge/" has been created
  - [ ] All platinum-level best practices have been identified and produced by classification
  - [ ] All errors during development process have been identified and recorded (including type, context, solution)
  - [ ] Each knowledge point has clear evidence source annotation
  - [ ] Todo list has been created and all items are marked as completed
