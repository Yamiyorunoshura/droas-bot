## [Input]
  1. "{TMPL}/requirement-tmpl.yaml" --Requirement template
  2. User-provided initial ideas and requirement descriptions

## [Output]
  1. "{REQ}/*.md" --Complete requirement specifications (including functional requirements, non-functional requirements, and acceptance criteria)

## [Constraints]
  1. Each requirement must be verifiable and measurable; vague or subjective wording is not allowed
  2. Section and field names must align exactly with the requirement template
  3. Examples must not contain sensitive or personal data
  4. This workflow transforms initial ideas into complete, testable requirements through structured interaction
  5. Use standardized requirement templates to ensure consistency and verifiability

## [Tools]
  1. **todo_write**
    - [Step 1: Create task list; Steps 2-4: Track task progress]
  2. **sequentialthinking (MCP)**
    - [Step 1: Structured decomposition and reflective reasoning; Step 2: Systematically decompose complex functional requirements; Step 3: Cross-domain systematic non-functional requirement analysis; Step 4: Structured Given-When-Then and verification logic]
  3. **playwright_browser (MCP)**
    - [Step 2: Conduct web research when needed to obtain requirement examples]
  4. **claude-context (MCP)**
    - [Step 3: Search codebase for requirement-related implementations]

## [Steps]
  1. Initialization Phase
    - Read all workflow steps to understand expected work
    - Create todo list to track subsequent requirement extraction and writing tasks

  2. Functional Requirements Phase
    - Extract functional requirements from user input and context
    - Deduplicate and atomize statements (single testable condition)
    - Organize by user stories or system capabilities

  3. Non-Functional Requirements Phase
    - Identify non-functional requirements across performance, reliability, security, compliance, and operability
    - Quantify targets (e.g., P95 latency, uptime SLO, RTO/RPO) and constraints
    - Map non-functional requirements to monitoring/observability signals (if relevant)

  4. Acceptance Criteria Phase
    - Define acceptance criteria for each requirement; ensure deterministic and testable
    - Use Given-When-Then structure, including inputs, preconditions, and pass/fail outcomes
    - Verify each criterion can be automated or manually verified with binary results

  5. Finalization Phase
    - Cross-check functional requirements, non-functional requirements, and acceptance criteria for consistency
    - Populate requirement template and prepare draft content for "{root}/docs/requirements.md"
    - Present draft content to user showing all sections (functional requirements, non-functional requirements, acceptance criteria)
    - if user approves draft then proceed to 5.1, else proceed to 5.2
      
      5.1. Write Final Documents
        - Write approved content to "{root}/docs/requirements.md"
        - Execute uv run "{SCRIPTS}/shard-requirements.py"
        - if execution succeeds then proceed to 5.1.1, else proceed to 5.1.2
          
          5.1.1. Execution Success Path
            - Summarize the tasks completed
          
          5.1.2. Execution Failure Path
            - Check if the requirement document follow the templates structure and fulfil the requirements of the template and the scripts
            - Fix the requirement document if it does not follow the structure required by the scripts and templates
            - Re-execute the script
      
      5.2. Revise Based on Feedback
        - Collect user feedback on what needs to be changed
        - Revise functional requirements, non-functional requirements, or acceptance criteria according to feedback
        - Return to present revised draft and request approval again

## [DoD]
  - [ ] Functional requirements have been extracted, deduplicated, and atomized
  - [ ] Non-functional requirements have been identified and quantified (including performance, security, reliability, etc.)
  - [ ] Each requirement has corresponding acceptance criteria (using Given-When-Then structure)
  - [ ] Content is in English and uses numbered lists
  - [ ] Each requirement is measurable and verifiable
  - [ ] "shard-requirements.py" has been executed or errors have been logged to "{root}/logs/errors.log" and user has been notified
  - [ ] User confirmation of the final requirement set has been obtained
