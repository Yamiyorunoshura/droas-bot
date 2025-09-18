<input>
  <context>
  1. {root}/docs/review-results/*.md - Review Records from development process
  2. {root}/sunnycore/templates/completion-report-tmpl.yaml - Completion Report Template
  </context>
  <templates>
  1. {root}/sunnycore/completion-report-tmpl.yaml - Project completion report template
  </templates>
</input>

<output>
1. {root}/docs/completion-report.md - Comprehensive Project Completion Report
</output>

<constraints, importance = "Important">
- Read only the specified inputs; do not invent content
- Produce concise, action-oriented bullets
- Use the provided completion report template sections
- Output exactly one file at the stated path
</constraints>

<workflow, importance = "Important">
  <stage id="0: create todo list">
  <tools: todo-list>
  - Read this file end-to-end
  - Enumerate all stages and sub-steps
  - Create a todo item for each working stage
  </tools: todo-list, sequential-thinking>

  <questions>
  - Are all steps captured as actionable todos?
  - Is each todo unambiguous and outcome-focused?
  - Do todos map cleanly to the workflow stages?
  </questions>

  </stage>

  <stage id="1: conclude the current issues">
  <tools: sequential-thinking>
  - Extract all issues from the review records
  - Define the minimal effective fix for each issue
  - Finalise each issue with assigned fix and priority
  </tools: sequential-thinking>
  
  <questions>
  - What root causes recur across review records?
  - What is the minimal viable fix for each issue?
  - Do any fixes introduce cross-cutting changes?
  </questions>

  </stage>

  <stage id="2: conclude the recommended actions">
  <tools: sequential-thinking>
  - Extract all recommended actions from the review records
  - Specify best-practice approach and sequencing per action
  - Finalise the action plan with rationale and sequencing
  </tools: sequential-thinking>

  <questions>
  - Which actions yield the highest impact/effort ratio?
  - What dependencies or sequencing are required?
  - Which best practices or references support each action?
  </questions>

  </stage>

  <stage id="3: conclude the completion report">
  - Synthesise issues and actions into a coherent summary
  - Draft the completion report using the template
  - Save the Markdown report to the stated path

  <questions>
  - Does the report clearly synthesise issues and actions?
  - Does it follow the template sections and structure?
  - Are all referenced paths and links valid?
  </questions>

  <checks>
  - [ ] Report saved at {root}/docs/completion-report.md
  - [ ] All template sections are filled without placeholders
  - [ ] Issues, fixes, and actions are cross-referenced consistently
  </checks>
  </stage>
</workflow>

<example>
markdown文件輸出方式：
	•	YAML 第一層 key 轉換為 Markdown 一級標題 (#)
	•	YAML 第二層 key 轉換為 Markdown 二級標題 (##)
	•	YAML 第三層 key 轉換為 Markdown 三級標題 (###)
	•	YAML value（字串或數字） 轉換為 Markdown 正文文字
</example>

<example>
todo list example:
- [ ] stage 0: create todo list
- [ ] stage 1: conclude the current issues
- [ ] stage 2: conclude the recommended actions
- [ ] stage 3: conclude the completion report
</example>