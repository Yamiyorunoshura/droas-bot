**GOAL**: Execute project acceptance, validating deliverables from business and user perspectives.

## [Input]
  1. (Conditional) "{PRD}" --Product Requirements Document (if exists, used as primary requirement source)
  2. (Conditional) "{REQ}/*.md" --Requirement documents (required if "PRD.md" does not exist)
  3. (Conditional) "{ARCH}/*.md" --Architecture documents (required if "PRD.md" does not exist)
  4. "{TMPL}/cutover-report-tmpl.yaml" --Cutover report template (required)

## [Output]
  1. "{CUTOVER}" --Cutover report (Markdown format)

## [Constraints]
  1. Do not perform technical testing (must test from end-user perspective)
  2. Do not proceed if required input files are missing (must list and halt)
  3. Do not skip testing any critical business requirements
  4. Do not deviate from template structure

## [Tools]
  1. **todo_write**: Create and manage task list
    - [Step 1: Create todo list; Steps 2-6: Track task progress]
  2. **sequential-thinking (MCP)**: Perform structured reasoning and verification
    - [Step 2: Reason about configuration requirements and environment dependencies]
    - [Step 4: Analyze acceptance test results and assess business impact]
  3. **context7 (MCP)**
    - [Step 2: Query official documentation related to deployment configuration]

## [Steps]
  1. Preparation and Validation
  - Task: Understand business objectives and project structure
  - Expected outcome: Progress tracking established for acceptance tasks

  2. Understanding and Configuration
  - Task: Identify project type and configuration needs
  - Expected outcome: All configuration requirements clearly documented

  3. Environment Setup
  - Task: Configure project environment following documentation
  - Expected outcome: Fully configured environment with all setup steps documented

  4. Project Execution
  - Task: Run project and verify functionality
  - Expected outcome: Running project with execution results and detailed logs documented

  5. Acceptance Testing
  - Task: Test all critical business requirements from end-user perspective
  - Expected outcome: Comprehensive test results with evidence and all issues documented with severity

  6. Report Generation
  - Task: Generate cutover report and determine status
  - Expected outcome: Complete cutover report at "{CUTOVER}" with status (Success/Partial Success/Failed)

## [Acceptance-Testing-Guidelines]
  1. **End-User Perspective Testing**
    - Test all critical business requirements from user perspective (not technical testing)
    - Verify user workflows are smooth and intuitive
    - Assess error messages clarity and user guidance
  
  2. **Configuration & Setup Validation**
    - Identify and document all configuration requirements
    - Verify setup documentation completeness
    - Test deployment and installation procedures
  
  3. **Issue Documentation**
    - Record all issues with severity classification (critical/high/medium/low)
    - Provide clear reproduction steps for each issue
    - Assess business impact of each issue
  
  4. **Business Value & Readiness**
    - Confirm deliverables align with business objectives
    - Determine cutover status: Success / Partial Success / Failed
    - Assess production deployment readiness and risk

## [DoD]
  - [ ] All critical business requirements tested from end-user perspective with results recorded
  - [ ] Complete cutover report at "{CUTOVER}" with status (Success/Partial Success/Failed) and clear rationale
  - [ ] All issues documented with severity classification and reproduction steps

## [Example]

### Example 1: Mobile App Release
[Input]
- PRD: docs/PRD.md (REQ-001: user login, REQ-002: push notifications, REQ-003: offline mode)
- Template: cutover-report-tmpl.yaml

[Decision]
- Setup: Install Expo, configure Firebase (successful)
- Run: expo start (app launches successfully on iOS/Android simulators)
- Test REQ-001: Login with test credentials (✓ success)
- Test REQ-002: Send test notification (✗ failed - Firebase config missing)
- Test REQ-003: Offline data sync (✓ success)
- Issues: 1 critical (push notifications not working)

[Expected Outcome]
- docs/cutover-report.md with status: Partial Success
- Test results: REQ-001 (pass), REQ-002 (fail - missing FCM server key), REQ-003 (pass)
- Issues documented: Critical severity, reproduction steps (send notification → error "Invalid FCM token")
- Configuration needs: Firebase Cloud Messaging server key required

### Example 2: API Deployment
[Input]
- Requirements: docs/requirements/functional.md (REQ-001: user CRUD, REQ-002: authentication)
- Architecture: docs/architecture/components.md (Node.js API, PostgreSQL, Redis)
- Template: cutover-report-tmpl.yaml

[Decision]
- Setup: docker-compose up (PostgreSQL starts, Redis fails - port conflict)
- Config fix: Change Redis port from 6379 to 6380
- Run: API server starts successfully on port 3000
- Test REQ-001: Create user, get user, update user, delete user (✓ all pass)
- Test REQ-002: Login, token refresh (✓ all pass)
- Status: Success (all requirements met after config fix)

[Expected Outcome]
- docs/cutover-report.md with status: Success
- Configuration documented: Redis port changed to 6380 (add to .env file)
- All test results recorded with API response examples
- Recommendation: Update README.md with Redis port configuration