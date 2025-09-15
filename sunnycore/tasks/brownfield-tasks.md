<input>
  <context>
    1. Implementation plan document: {root}/docs/implementation-plan/{task_id}-plan.md
    2. Review results document: {root}/docs/review-results/{task_id}-review.md
  </context>
  <templates>
    1. Development notes template: {root}/sunnycore/templates/dev-notes-tmpl.yaml
  </templates>
</input>

<output>
  1. Code fixes implemented based on review findings
  2. Comprehensive development notes: {root}/docs/dev-notes/{task_id}-notes.md
</output>

<constraints, importance = "Important">
  - Changes must be minimal, reversible, and scoped to identified issues
  - Preserve existing architecture, public APIs, and naming conventions; justify any new dependency in plan/dev-notes
  - Add or update tests to cover each fix/action and run tests after every change
  - Follow project style and lint rules; avoid unrelated reformatting or file churn
  - Keep todos updated per stages and record decisions/risks in the development notes
</constraints>

<workflow, importance = "Normal">
  <stage id="0: Create a todo list">
  - Read this file end-to-end
  - Enumerate all stages and sub-steps
  - Create one todo per step (atomic, outcome-focused)

  <questions>
  - Have all steps been enumerated without omission?
  - Are todos atomic and actionable?
  - Is naming consistent with project conventions?
  </questions>
  </stage>

  <stage id="1: Analyse the current issues">
  - Read the review results document
  - Analyse current issues and root causes
  - Define the minimal effective fix for each issue

  <questions>
  - What are the root causes vs symptoms?
  - Which fixes have highest risk or impact?
  - Do we need design changes or refactors?
  </questions>
  </stage>

  <stage id="2: Fix the issues">
  - Update the todo list with concrete steps per solution
  - Implement fixes iteratively, one at a time
  - Run and pass tests after each fix

  <questions>
  - Is each change minimal and reversible?
  - Do tests cover the fixed behavior?
  - Any regression risks identified?
  </questions>
  </stage>

  <stage id="3: Enact recommended actions">
  - Update the todo list with sequenced steps for the recommended actions
  - Enact the recommended actions iteratively
  - Run and pass tests after each action

  <questions>
  - Are recommendations prioritized by value/effort?
  - Are acceptance criteria met per action?
  - Do changes align with architecture guidelines?
  </questions>
  </stage>

  <stage id="4: Create a development notes">
  - Conclude the development process
  - Create development notes based on the template
  - Save the markdown dev-notes to the stated directory

  <questions>
  - Are decisions and trade-offs documented?
  - Are next steps and risks recorded?
  - Are links to plans and reviews included?
  </questions>
  </stage>

  <checks>
  - [ ] Questions provided (2-3) for each stage
  - [ ] Tests pass after each fix/action
  - [ ] Dev-notes created at {root}/docs/dev-notes/{task_id}-notes.md
  - [ ] No scope creep; todos remain atomic
  </checks>
</workflow>