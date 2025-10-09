**GOAL**: Summarize project development outcomes and generate comprehensive delivery report.

## [Input]
  1. "{root}/docs/*.md" --All files in docs/ directory recursively (required)
  2. "{LOCK}" --Version information (required)
  3. "{TMPL}/completion-report-tmpl.yaml" --Completion report template (required)

## [Output]
  1. Completion report: "{root}/docs/completion-report.md" (Markdown format)

## [Constraints]
  1. Do not omit any of the 5 core content items (decisions, tech choices, issues/solutions, evidence, recommendations)
  2. Do not incorrectly parse or miss version number from lock file
  3. Do not move architecture/, knowledge/, or completion-report.md during archiving
  4. Do not proceed if required input files are missing (must list missing files and halt)

## [Tools]
  1. **todo_write**: Create and manage task list
    - [Step 1: Create todo list; Steps 2-7: Track task progress]
  2. **sequential-thinking (MCP)**: Perform structured reasoning and verification
    - [Step 2: Reason about report conception and content organization]
    - [Step 3: Verify if DoD is satisfied]
    - When to use: When need to structure complex completion report or validate completeness
  3. **claude-context (MCP)**: Search codebase to locate implementation details
    - [Step 2: Find relevant code for evidence]
    - [Step 3: Precise keyword search for implementation verification]
    - Note: If not indexed or search fails, switch to grep tool for keyword search, or annotate "Unable to locate code evidence" and continue

## [Steps]
  1. Input Validation
  - Task: Parse version number and validate required input files
  - Expected outcome: Version number extracted from lock file, all required files verified

  2. Information Extraction and Mapping
  - Task: Extract all 5 core content items and map to template fields
  - Expected outcome: Complete extraction with code evidence supplemented where needed

  3. Write Report
  - Task: Generate completion report at "{root}/docs/completion-report.md"
  - Expected outcome: Complete report following template structure with all 5 core items

  4. Quality Check
  - Task: Verify report completeness and quality
  - Expected outcome: Report meets template requirements with all core items clearly presented

  5. Create Archive
  - Task: Create archive directory at "{ARCHIVE}/{version_name}/"
  - Expected outcome: Archive directory created and verified

  6. Move Files to Archive
  - Task: Archive files while preserving architecture/, knowledge/, and completion-report.md
  - Expected outcome: Files properly archived with verification complete

  7. Update Document References
  - Task: Update document references to point to archive paths
  - Expected outcome: All references updated and verified for correctness

## [Completion-Report-Guidelines]
  1. **Core Content (5 Required Items)**
    - **Decisions**: Key decisions with rationale, context, and trade-offs
    - **Technical Choices**: Technology selection with pros/cons and alternatives
    - **Issues & Solutions**: Problems encountered, root cause, solutions, preventive measures
    - **Evidence**: Verifiable evidence for all statements (format: file_path:line_number)
    - **Recommendations**: Future improvements and optimization directions
  
  2. **Traceability & Verification**
    - Link all statements to actual code, test results, or document sections
    - Use DoD verification evidence format: "file_path:Lstart-Lend"
    - If any core item missing from sources, annotate "To be supplemented: missing XX item"
  
  3. **Version & Archiving**
    - Parse version from *.lock file (format: "version = x.x.x")
    - Archive all docs to "{ARCHIVE}/{version_name}/" except architecture/, knowledge/, completion-report.md
    - Update references in preserved docs to point to archive paths

## [DoD]
  - [ ] Completion report generated at "{root}/docs/completion-report.md" with all 5 core items (decisions, tech choices, issues/solutions, evidence, recommendations)
  - [ ] All docs/ files EXCEPT architecture/, knowledge/, and completion-report.md moved to "{ARCHIVE}/{version_name}/"
  - [ ] Document references updated to point to correct archive paths

## [Example]

### Example 1: E-commerce Feature Completion
[Input]
- Lock file: sunnycore.lock (version = 1.2.0)
- Documents: docs/PRD.md, docs/prd-dev-notes.md, docs/cutover-report.md
- Template: completion-report-tmpl.yaml

[Decision]
- Version: 1.2.0 (parsed from lock file)
- Workflow: PRD (PRD.md exists)
- Extract: Key decisions (async payment processing - ADR-003), tech choices (Stripe over PayPal), issues (rate limit bug fixed), evidence (src/services/PaymentService.js:L42-L56)
- Archive: Move PRD.md, prd-dev-notes.md, cutover-report.md to docs/archive/1.2.0/

[Expected Outcome]
- docs/completion-report.md with all 5 core items (decisions, tech choices, issues/solutions, evidence, recommendations)
- docs/archive/1.2.0/ contains PRD.md, dev-notes, cutover-report.md
- docs/ only has architecture/, knowledge/, completion-report.md
- References updated: knowledge/best-practices.md now references docs/archive/1.2.0/prd-dev-notes.md

### Example 2: Microservices Migration Project
[Input]
- Lock file: sunnycore.lock (version = 2.0.0)
- Documents: docs/requirements/*.md, docs/architecture/*.md, docs/epic.md, docs/plans/*.md, docs/dev-notes/*.md (8 tasks), docs/review/*.md
- Template: completion-report-tmpl.yaml

[Decision]
- Version: 2.0.0 (major version, architecture change)
- Workflow: Traditional/Full (epic.md, plans/ exist)
- Extract: Decisions (strangler fig pattern - ADR-001), tech choices (gRPC for inter-service communication), issues (data migration challenges), evidence (src/services/OrderService/server.js:L10-L80)
- Archive: Move requirements/, plans/, dev-notes/, review/, epic.md to docs/archive/2.0.0/

[Expected Outcome]
- docs/completion-report.md documenting microservices migration journey
- All 5 core items covered with detailed evidence from codebase
- docs/archive/2.0.0/ contains all project artifacts except architecture/ and knowledge/
- architecture/components.md references updated to point to archive for historical decisions
