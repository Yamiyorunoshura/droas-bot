## [Input]
  1. "{REQ}/*.md" --Authoritative project requirements
  2. "{SCRIPTS}/shard-architecture.py" --Architecture sharding script
  3. "{TMPL}/architecture-tmpl.yaml" --Standardized architecture template

## [Output]
  1. Architecture document collection under "{ARCH}/" directory (*.md format)
  2. Minimum expected files example (more classification files can be produced based on project complexity):
    - "{ARCH}/overview.md"
    - "{ARCH}/components.md"
    - "{ARCH}/traceability_matrix.md"

## [Constraints]
  1. Must verify that "{REQ}/*.md" exists and is complete before architecture design; if requirements are incomplete or contain conflicts, record the issues and confirm with requirement authors, do not make assumptions
  2. Must create explicit requirement-to-architecture mapping relationships covering functional and non-functional requirements (mapping means requirement ID explicitly links to specific component names or design decision numbers)
  3. Must verify that each requirement has corresponding architecture components or design decisions
  4. Must use the standardized template to write "{root}/docs/architecture.md"; strictly preserve section order and 2-space indentation; do not introduce non-existent paths
  5. After drafting is complete, must execute 'uv run "{SCRIPTS}/shard-architecture.py"' and verify output appears under "{ARCH}/"
  6. Architecture design required external API call must use context7 (MCP) to search for library documentation and API references

## [Tools]
  1. **todo_write**
    - [Step 1: Track and update execution tasks; Steps 2-4: Track writing progress and results]
  2. **sequentialthinking (MCP)**
    - [Step 1: Decompose requirements and identify architecture patterns; Step 2: Architect system components and verify design decisions; Step 3: Structured drafting and verification steps; Step 4: Execute final verification sequence]
  3. **context7 (MCP)**
    - [Step 2: Obtain external package and architecture pattern references]

## [Steps]
  1. Requirement Analysis Phase
    - Verify the completeness and consistency of all requirements under "{REQ}/*.md"
    - Extract functional/non-functional requirements and convert non-functional requirements into architecture constraints
    - Create mapping matrix (requirement ID → component/decision) and identify gaps or conflicts
    - Create todo list to track subsequent architecture design tasks

  2. Architecture Design Phase
    - Define components, boundaries, and specify data flows based on requirement analysis
    - Ensure each requirement maps to an architecture element; define interaction contracts and data schemas
    - Record decisions (recommend ADR format or table format) and include requirement traceability
    - Handle cross-cutting concerns (security, observability, performance)

  3. Writing Phase
    - Use "{TMPL}/architecture-tmpl.yaml" to draft "{root}/docs/architecture.md" content
    - Include requirement traceability matrix and ensure each mapping relationship has been handled
    - Present draft content to user showing key sections (overview, components, traceability matrix, design decisions)
    - if user approves draft then proceed to 3.1, else proceed to 3.2
      
      3.1. Write Final Documents
        - Write approved content to "{root}/docs/architecture.md"
        - Execute uv run "{SCRIPTS}/shard-architecture.py"
        - Verify output appears under "{ARCH}/"
        - if execution succeeds then proceed to Step 4, else check if "architecture.md" format complies with template specifications, fix and re-execute
      
      3.2. Revise Based on Feedback
        - Collect user feedback on what needs to be changed
        - Revise the draft content according to feedback
        - Return to present revised draft and request approval again

  4. Final Verification Phase
    - Cross-verify through mapping matrix whether architecture satisfies all requirements
    - Fix typographical errors and standardize terminology
    - Confirm architecture decisions are justified by requirements

## [Error-Handling]
  1. "shard-architecture.py" execution failure: Check if "architecture.md" format complies with template specifications, fix and re-execute
  2. Unresolvable requirement conflicts: Record conflicts and confirm with requirement authors, do not make assumptions
  3. Architecture design infeasibility: Record technical limitations and propose alternative solutions

## [DoD]
  - [ ] Verified that "{REQ}/*.md" exists and is complete
  - [ ] Created requirement-to-architecture mapping matrix (requirement ID → component/decision), complete and accurate (100% coverage, no omissions or incorrect mappings)
  - [ ] All functional requirements have been mapped to specific architecture components
  - [ ] Non-functional requirements have been converted to measurable architecture constraints
  - [ ] User approval has been obtained for architecture draft
  - [ ] "{root}/docs/architecture.md" exists and follows the template
  - [ ] "shard-architecture.py" has been executed and shard file generation has been verified
