**GOAL**: Fix design conflicts and inconsistencies identified in design validation report, then clean up the validation report file.

## [Input]
  1. "{root}/docs/design-validation.md" --Validation report with identified issues

## [Output]
  1. Modified design documents based on conflict resolution:
    - "{PRD}" (if PRD workflow was used)
    - "{REQ}/*.md" (if full workflow was used)
    - "{ARCH}/*.md" (if full workflow was used)
    - "{EPIC}" (if full workflow was used)
    - "{PLAN}/*.md" (if full workflow was used)
  2. Deleted: "{root}/docs/design-validation.md" (after successful fixes)

## [Constraints]
  1. Do not fix issues out of severity order (Critical → High → Medium → Low)
  2. Do not apply fixes without user confirmation of strategy
  3. Do not introduce new conflicts or inconsistencies during fixes
  4. Do not delete validation report before all fixes are complete

## [Tools]
  1. **todo_write**
    - [Step 1: Create todo list with all issues to fix; Steps 2-4: Track fix progress]
  2. **sequential-thinking (MCP)**
    - [Step 1: Analyze issues and determine fix strategies]
    - [Step 2-3: Reason about conflict resolution approaches and evaluate impact scope]
    - When to use: When need to evaluate multiple fix approaches or analyze cross-document consistency
  3. **claude-context (MCP)**
    - [Step 2: Verify impact of fixes on existing code]

## [Steps]
  1. Analysis
  - Task: Understand all issues from validation report
  - Expected outcome: Issues extracted, grouped, and prioritized

  2. Fix Planning
  - Task: Create comprehensive fix plans for all issues
  - Expected outcome: Fix plans approved by user for each issue

  3. Fix Execution
  - Task: Resolve all approved fixes in priority order
  - Expected outcome: All fixes applied with cross-document consistency maintained

  4. Validation
  - Task: Review and validate all changes
  - Expected outcome: All changes validated with no new conflicts introduced

  5. Cleanup
  - Task: Delete validation report and provide summary
  - Expected outcome: Validation report deleted, user guided to re-run validation for confirmation

## [Fix-Strategies]

### For Fabricated Content:
  - Remove fabricated references
  - Replace with actual existing entities
  - Add missing entities if they should exist

### For Broken References:
  - Update references to correct entity IDs
  - Create missing entities if needed
  - Remove orphaned references

### For Conflicts:
  - Identify authoritative source (usually requirements)
  - Update conflicting documents to match
  - Resolve contradictions by choosing one approach

### For Inconsistent Naming:
  - Establish canonical naming convention
  - Update all occurrences across all documents
  - Create glossary if needed

### For Coverage Gaps:
  - Add missing mappings (req → arch → task → plan)
  - Create missing entities if needed
  - Remove orphaned entities if they are not needed

### For Terminology Inconsistencies:
  - Standardize terms across all documents
  - Use find-and-replace for consistent updates
  - Document terminology decisions

## [Error-Handling]
  1. Validation report missing: Cannot proceed, ask user to run validate-design first
  2. Validation report shows no issues: Nothing to fix, inform user and exit gracefully
  3. User rejects all fix proposals: Document user's concerns and stop execution
  4. Fix introduces new conflicts: Revert changes and propose alternative approach
  5. Critical files cannot be modified: Report error and list problematic files

## [Conflict-Resolution-Guidelines]
  1. **Prioritize by Severity**
    - Fix in order: Critical → High → Medium → Low
    - Focus on issues that block development or cause cascading problems
    - Obtain user confirmation for each fix strategy before execution
  
  2. **Maintain Consistency**
    - When fixing references, update all related documents simultaneously
    - Establish canonical source of truth (usually requirements for functional specs)
    - Use consistent terminology and naming conventions across fixes
  
  3. **Preserve Traceability**
    - Ensure 100% bidirectional mapping after fixes: requirements ↔ architecture ↔ tasks ↔ plans
    - Verify no new orphaned entities or broken references introduced
    - Document rationale for each resolution decision
  
  4. **Validation After Fixes**
    - Recommend re-running validate-design after all fixes complete
    - Verify cross-document consistency maintained
    - Ensure no new conflicts introduced during fix process

## [DoD]
  - [ ] All approved fixes successfully applied with cross-document consistency verified
  - [ ] No new issues introduced and all changes confirmed by user
  - [ ] Validation report "{root}/docs/design-validation.md" deleted

## [Example]

### Example 1: Broken References in PRD
[Input]
- Validation report: docs/design-validation.md (3 critical issues: REQ-005 doesn't exist, Component "EmailService" fabricated, Task-4 references non-existent REQ-006)

[Decision]
- Issue 1: REQ-005 referenced but doesn't exist in PRD requirements section
  - Fix: Remove REQ-005 reference, map to existing REQ-003 instead
- Issue 2: Architecture mentions "EmailService" but no such component defined
  - Fix: Add EmailService to architecture components section with proper definition
- Issue 3: Task-4 maps to REQ-006 which doesn't exist
  - Fix: Update Task-4 to map to REQ-004 (notification requirements)

[Expected Outcome]
- Updated docs/PRD.md with fixes: REQ-005 → REQ-003, EmailService added to architecture, Task-4 → REQ-004
- Cross-document consistency verified (all references valid)
- docs/design-validation.md deleted
- Recommendation: Re-run `/sunnycore_po *validate-design prd` to confirm

### Example 2: Coverage Gaps in Full Workflow
[Input]
- Validation report: docs/design-validation.md (Coverage: REQ-001 has no architecture mapping, COMP-003 not mapped to any task, Task-2 has no implementation plan)

[Decision]
- Gap 1: REQ-001 (user authentication) missing architecture component
  - Fix: Add "AuthService" component to docs/architecture/components.md, map to REQ-001
- Gap 2: COMP-003 (Cache Layer) exists but no task implements it
  - Fix: Add Task-5 "Implement caching" to docs/epic.md, map to COMP-003
- Gap 3: Task-2 in epic but no plan file
  - Fix: Create docs/plans/2-plan.md with TDD structure for Task-2

[Expected Outcome]
- docs/architecture/components.md: Added AuthService with REQ-001 mapping
- docs/epic.md: Added Task-5 for Cache Layer implementation
- docs/plans/2-plan.md: Created implementation plan for Task-2
- 100% coverage achieved: requirements → architecture → tasks → plans
- docs/design-validation.md deleted after user confirmation

