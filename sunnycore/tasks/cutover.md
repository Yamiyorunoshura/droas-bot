## [Input]
  1. (Conditional) "{PRD}" --Product Requirements Document (if exists, used as primary requirement source)
  2. (Conditional) "{REQ}/*.md" --Requirement documents (required if "PRD.md" does not exist)
  3. (Conditional) "{ARCH}/*.md" --Architecture documents (required if "PRD.md" does not exist)
  4. "{TMPL}/cutover-report-tmpl.yaml" --Cutover report template (required)

## [Output]
  1. "{CUTOVER}" --Cutover report (Markdown format)

## [Constraints]
  1. Must follow cutover report template structure
  2. Must test project functionality from end-user perspective, not technical testing
  3. If required configuration is missing or unclear, must document specific needs
  4. If project fails to run, must record detailed error information and reproduction steps
  5. Must verify all critical business requirements stated in requirement documents
  6. If any required input files are missing, must generate a missing file list and halt execution

## [Tools]
  1. **todo_write**: Create and manage task list
    - [Step 1: Create todo list; Steps 2-6: Track task progress]
  2. **sequentialthinking (MCP)**: Perform structured reasoning and verification
    - [Step 2: Reason about configuration needs; Step 4: Analyze acceptance test results]

## [Steps]
  1. Preparation and Validation Phase
    - Verify existence of all required input files
    - Check if "{PRD}" exists
    - Understand business objectives and project structure
    - Create todo list based on actual tasks
    - if "PRD.md" exists then proceed to 1.1, else proceed to 1.2
      
      1.1. PRD-based Project
        - Read "{PRD}"
        - Extract requirements from PRD requirements section
        - Extract architecture information from PRD architecture section
        - Use PRD as the primary requirement source for acceptance testing
      
      1.2. Traditional Project Structure
        - Read all requirement documents from "{REQ}/*.md"
        - Read all architecture documents from "{ARCH}/*.md"
        - If any required files are missing, generate missing list and halt execution

  2. Understanding and Configuration Phase
    - Identify project type (web app, API, CLI tool, library, etc.)
    - Identify required dependencies and environment setup
    - Identify configuration needs (API keys, tokens, database connections, etc.)
    - Read documentation for setup instructions
    - Document all configuration requirements clearly for end users

  3. Environment Setup Phase
    - Follow documentation to set up project environment
    - Install all required dependencies
    - Apply necessary configurations
    - Document all setup steps performed
    - If setup fails, record error details and potential causes

  4. Project Execution Phase
    - Attempt to run the project according to documentation
    - Verify project functionality based on project type (web app starts and is accessible, API endpoints are reachable, CLI commands execute)
    - Document success or failure with detailed logs
    - If execution fails, record exact error messages and reproduction steps

  5. Acceptance Testing Phase
    - For each critical business requirement in requirement documents, test functionality from end-user perspective
    - Verify acceptance criteria are met and document test result (Pass/Fail) with evidence
    - Evaluate user experience and usability
    - Test error handling and edge cases
    - Document all issues found with severity and business impact

  6. Report Generation Phase
    - Create cutover report using template structure
    - Document cutover status: Success / Partial Success / Failed
    - List all configuration requirements identified
    - Document all test results with evidence
    - List all issues found with details and provide recommendations for fixes or improvements
    - Save report to "{CUTOVER}"

## [DoD]
  - [ ] All required input files have been read
  - [ ] Project type and configuration needs have been identified
  - [ ] Environment setup has been attempted and documented
  - [ ] Project execution has been attempted and results documented
  - [ ] All critical business requirements have been tested
  - [ ] Cutover report has been generated and complies with template structure
  - [ ] Cutover status (Success/Partial Success/Failed) has been determined
  - [ ] All configuration requirements have been documented
  - [ ] All test results have been recorded with evidence
  - [ ] All issues have been documented with severity and reproduction steps