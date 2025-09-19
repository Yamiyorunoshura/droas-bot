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
  2. Comprehensive development notes: {root}/docs/dev-notes/{task_id}-dev-notes.md
</output>

<constraints, importance = "Important">
  - Changes must be minimal, reversible, and scoped to identified issues
  - Preserve existing architecture, public APIs, and naming conventions; justify any new dependency in plan/dev-notes
  - Add or update tests to cover each fix/action and run tests after every change
  - Follow project style and lint rules; avoid unrelated reformatting or file churn
  - Keep todos updated per stages and record decisions/risks in the development notes
</constraints>


<example>
markdown文件輸出方式：
	•	YAML 第一層 key 轉換為 Markdown 一級標題 (#)
	•	YAML 第二層 key 轉換為 Markdown 二級標題 (##)
	•	YAML 第三層 key 轉換為 Markdown 三級標題 (###)
	•	YAML value（字串或數字） 轉換為 Markdown 正文文字
</example>

<example>
todo list example:
- [ ] stage 0: Create a todo list
- [ ] stage 1: Analyse the current issues
- [ ] stage 2: Fix the issues
- [ ] stage 3: Enact recommended actions
- [ ] stage 4: Create a development notes
</example>

<workflow, importance = "Normal">
  <stage id="0: Create a todo list">
  <tools: todo-list>
  - Read this file end-to-end
  - Enumerate all stages and sub-steps
  - Take reference from the example and create a todo item 
  </tools: todo-list>

  <questions>
  - Have all steps been enumerated without omission?
  - Are todos atomic and actionable?
  - Is naming consistent with project conventions?
  </questions>
  </stage>

  <stage id="1: Analyse the current issues">
  <tools: sequential-thinking>
  - Read the review results document
  - Analyse current issues and root causes
  - Define the minimal effective fix for each issue
  </tools: sequential-thinking>

  <questions>
  - What are the root causes vs symptoms?
  - Which fixes have highest risk or impact?
  - Do we need design changes or refactors?
  </questions>
  </stage>

  <stage id="2: Fix the issues">
  <tools: todo-list>
  - Update the todo list with concrete steps per solution
  - Implement fixes iteratively, one at a time
  - Run and pass tests after each fix
  </tools: todo-list>

  <questions>
  - Is each change minimal and reversible?
  - Do tests cover the fixed behavior?
  - Any regression risks identified?
  </questions>
  </stage>

  <stage id="3: Enact recommended actions">
  <tools: todo-list>
  - Update the todo list with sequenced steps for the recommended actions
  - Enact the recommended actions iteratively
  - Run and pass tests after each action
  </tools: todo-list>
  
  <questions>
  - Are recommendations prioritized by value/effort?
  - Are acceptance criteria met per action?
  - Do changes align with architecture guidelines?
  </questions>
  </stage>

  <stage id="4: Create a development notes">
  - Conclude the development process and create development notes based on the template
  - If there is already an existing development notes, update the development notes with the new information
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
  - [ ] Dev-notes created at {root}/docs/dev-notes/{task_id}-dev-notes.md
  - [ ] No scope creep; todos remain atomic
  </checks>
</workflow>