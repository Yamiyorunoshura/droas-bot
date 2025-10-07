## [Input]
  1. "{root}/docs/*.md" --All files in docs/ directory recursively (required)
  2. "{LOCK}" --Version information (required)
  3. "{TMPL}/completion-report-tmpl.yaml" --Completion report template (required)

## [Output]
  1. Completion report: "{root}/docs/completion-report.md" (Markdown format)

## [Constraints]
  1. Must ensure completion report complies with template requirements
  2. Completion report must include the following core content:
    (1) All key decisions and their rationale
    (2) Technology choices and alternative solution comparisons
    (3) Problems encountered, root cause analysis, and solutions
    (4) Future recommendations
    (5) DoD verification evidence (format as "file path:line number", e.g., "src/main.py:L42-L56")
  3. If required input files (sunnycore.lock, template) are missing or incorrectly formatted, must generate a missing file list in the terminal and halt execution, waiting for user to provide supplementary files
  4. If any of the 5 core content items are completely missing from source documents, should annotate "To be supplemented: missing XX item" in the corresponding chapter of the completion report and continue execution
  5. Version name must be parsed from sunnycore.lock file (format: "version = x.x.x")
  6. When archiving files, must preserve docs/architecture/, docs/knowledge/, and docs/completion-report.md in their original locations

## [Tools]
  1. **todo_write**: Create and manage task list
    - [Step 1: Create todo list; Steps 2-7: Track task progress]
  2. **sequentialthinking (MCP)**: Perform structured reasoning and verification
    - [Step 2: Reasoning tasks for conceiving the report; Step 3: Verify if DoD is satisfied]
  3. **claude-context (MCP)**: Search codebase to locate implementation details
    - [Step 2: Find relevant code; Step 3: Precise keyword search]
    - Prerequisite: Codebase has been indexed through index_codebase
    - Failure handling: If not indexed or search fails, switch to grep tool for keyword search, or annotate "Unable to locate code evidence" and continue execution

## [Steps]
  1. Input Validation Phase
    - Verify existence of "{LOCK}" and read version number (format: "version = x.x.x")
    - Parse version name from lock file (e.g., "1.12.8")
    - Verify existence of completion report template
    - Recursively scan "{root}/docs/" directory to get all file list (including all subdirectories)
    - If required input files are missing (sunnycore.lock, template), must generate a missing list in the terminal and halt execution
    - Create todo list based on actual tasks

  2. Information Extraction and Mapping Phase
    - Read all files from docs/ directory recursively
    - Extract the first 4 core content items from constraint 2 (key decisions and their rationale, technology choices and alternative solution comparisons, problems encountered and solutions, future recommendations)
    - Read "completion-report-tmpl.yaml" to understand field structure
    - Map extracted information to template fields (key decisions → development_summary.key_decisions, technology choices → development_summary.technologies_used, problems → development_summary.challenges_encountered, future recommendations → recommendations.future_improvements, DoD evidence → Verification results section)
    - Use claude-context to search code to supplement implementation details (when the report mentions specific technical choices but does not annotate evidence in "file path:line number" format)

  3. Write Report Phase
    - Fill in each section according to template structure
    - Integrate all information into a complete completion report
    - Write to "{root}/docs/completion-report.md"

  4. Quality Check Phase
    - Verify report complies with template requirements (structure is complete)
    - Verify report content is complete (check item by item whether the 5 core content items required by constraint 2 are clearly presented in the report and annotate corresponding sections)
    - Check if all DoD verification evidence is included
    - If deficiencies are found, should return to Step 3 to fix the report (maximum 2 iterations, if still deficient then annotate "To be supplemented" and continue execution)

  5. Create Archive Directory Phase
    - Create "{ARCHIVE}/{version_name}/" directory if it does not exist
    - Verify directory creation was successful

  6. Move Files to Archive Phase
    - For both Traditional and PRD workflows:
      * Move all files and directories from "{root}/docs/" EXCEPT architecture/, knowledge/, and completion-report.md to "{ARCHIVE}/{version_name}/"
    - Suggested command: `find "{root}/docs" -mindepth 1 -maxdepth 1 ! -name "architecture" ! -name "knowledge" ! -name "completion-report.md" -exec mv -n {} "{ARCHIVE}/{version_name}/" \;`
    - If mv -n fails because files already exist, record a warning and annotate "Files already exist in archive folder"
    - Execute verification command: `ls -la "{root}/docs"` to confirm only architecture/, knowledge/, and completion-report.md remain
    - Execute verification command: `ls -la "{ARCHIVE}/{version_name}"` to confirm archived files exist in the target folder

  7. Update Document References Phase
    - Scan all files in "{ARCH}/" and "{KNOWLEDGE}/" directories
    - Identify all document (including completion-report.md) references that point to archived files (must not edit the document reference pointing the previous archived files(e.g. "archive/1.14.4/xxx/yyy.md"))
    - For each reference to an archived file, update the path to point to "archive/{version_name}/xxx/yyy.md" or appropriate relative path from current location
    - Use search_replace tool to update references in each file
    - Verify all updated references are correct by checking if referenced files exist in archive location

## [DoD]
  - [ ] *.lock file has been read and version number has been parsed successfully
  - [ ] Workflow type (Traditional/PRD) has been determined
  - [ ] All files in docs/ directory have been scanned recursively
  - [ ] Completion report has been generated and complies with template structure (including all necessary sections in the template)
  - [ ] Completion report content fully covers the 5 core content items listed in constraint 2
  - [ ] All files and directories in docs/ EXCEPT architecture/, knowledge/, and completion-report.md have been moved to {ARCHIVE}/{version_name}/
  - [ ] Only architecture/, knowledge/, and completion-report.md remain in docs/ directory
  - [ ] Document references in architecture/ and knowledge/ have been updated to point to correct archive paths
