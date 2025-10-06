[Input]
  1. "{root}/docs/*.md" --All files in docs/ directory recursively (required)
  2. "{root}/*.lock" --Version information (required)
  3. "{root}/sunnycore/templates/completion-report-tmpl.yaml" --Completion report template (required)

[Output]
  1. Completion report: "{root}/docs/completion-report.md" (Markdown format)

[Constraints]
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

[Tools]
  1. **todo_write**: Create and manage task list
    - [Step 1: Create todo list; Steps 2-7: Track task progress]
  2. **sequentialthinking (MCP)**: Perform structured reasoning and verification
    - [Step 2: Reasoning tasks for conceiving the report; Step 3: Verify if DoD is satisfied]
  3. **claude-context (MCP)**: Search codebase to locate implementation details
    - [Step 2: Find relevant code; Step 3: Precise keyword search]
    - Prerequisite: Codebase has been indexed through index_codebase
    - Failure handling: If not indexed or search fails, switch to grep tool for keyword search, or annotate "Unable to locate code evidence" and continue execution

[Steps]
  1. Input Validation Phase
    - Verify existence of "{root}/sunnycore.lock" and read version number (format: "version = x.x.x")
    - Parse version name from lock file (e.g., "1.12.8")
    - Verify existence of completion report template
    - Recursively scan "{root}/docs/" directory to get all file list (including all subdirectories)
    - If required input files are missing (sunnycore.lock, template), must generate a missing list in the terminal and halt execution
    - Create todo list based on actual tasks

  2. Information Extraction Phase
    - Read all files from docs/ directory recursively
    - Extract the first 4 core content items from constraint 2 (key decisions and their rationale, technology choices and alternative solution comparisons, problems encountered and solutions, future recommendations)
    - If key information is missing, handle according to constraint 4

  3. Structure Mapping Phase
    - Read "completion-report-tmpl.yaml" to understand field structure
    - Map extracted information to template fields:
      * Key decisions and their rationale → development_summary.key_decisions
      * Technology choices and alternative solution comparisons → development_summary.technologies_used
      * Problems encountered and solutions → development_summary.challenges_encountered
      * Future recommendations → recommendations.future_improvements
      * DoD verification evidence → Verification results section at the end of the report
    - Use claude-context to search code to supplement implementation details (when the report mentions specific technical choices but does not annotate evidence in "file path:line number" format)

  4. Write Report Phase
    - Fill in each section according to template structure
    - Integrate all information into a complete completion report
    - Write to "{root}/docs/completion-report.md"

  5. Quality Check Phase
    - Verify report complies with template requirements (structure is complete)
    - Verify report content is complete (check item by item whether the 5 core content items required by constraint 2 are clearly presented in the report and annotate corresponding sections)
    - Check if all DoD verification evidence is included
    - If deficiencies are found, should return to Step 4 to fix the report (maximum 2 iterations, if still deficient then annotate "To be supplemented" and continue execution)

  6. File Archiving Phase
    - Create "{root}/archive/{version_name}/" directory if it does not exist
    - Move all files and directories from "{root}/docs/" EXCEPT the following to "{root}/archive/{version_name}/":
      * architecture/ directory
      * knowledge/ directory
      * completion-report.md file
    - Suggested command: `mkdir -p "{root}/archive/{version_name}" && find "{root}/docs" -mindepth 1 -maxdepth 1 ! -name "architecture" ! -name "knowledge" ! -name "completion-report.md" -exec mv -n {} "{root}/archive/{version_name}/" \;`
    - If mv -n fails because files already exist, should record a warning and annotate "Files already exist in archive folder", DoD is considered complete
    - Execute verification command: `ls -la "{root}/docs"` to confirm only architecture/, knowledge/, and completion-report.md remain
    - Execute verification command: `ls -la "{root}/archive/{version_name}"` to confirm archived files exist in the target folder

  7. Update Document References Phase
    - Scan all files in "{root}/docs/architecture/" and "{root}/docs/knowledge/" directories
    - Identify all document references that point to archived files (formats to detect: "docs/xxx/yyy.md", "../xxx/yyy.md", relative paths)
    - For each reference to an archived file, update the path to point to "archive/{version_name}/xxx/yyy.md" or appropriate relative path from current location
    - Record the number of files updated and the number of references updated
    - Use search_replace tool to update references in each file
    - Verify all updated references are correct by checking if referenced files exist in archive location

  8. DoD Verification Phase
    - Check all DoD items one by one to ensure they are met
    - Attach item-by-item DoD verification results at the end of the report
    - Confirm all todo items are completed

[DoD]
  - [ ] sunnycore.lock file has been read and version number has been parsed successfully
  - [ ] All files in docs/ directory have been scanned recursively
  - [ ] Completion report has been generated and complies with template structure (including all necessary sections in the template)
  - [ ] Completion report content fully covers the 5 core content items listed in constraint 2
  - [ ] All files and directories in docs/ EXCEPT architecture/, knowledge/, and completion-report.md have been moved to archive/{version_name}/
  - [ ] Only architecture/, knowledge/, and completion-report.md remain in docs/ directory
  - [ ] Document references in architecture/ and knowledge/ have been updated to point to correct archive paths
  - [ ] All todo items are completed (including: input validation, information extraction, structure mapping, write report, quality check, DoD verification, file archiving, update references)
