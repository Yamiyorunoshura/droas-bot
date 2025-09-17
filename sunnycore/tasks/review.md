<input>
  <context>
  1. {root}/docs/dev-notes/{task_id}-dev-notes.md
  2. {root}/docs/implementation-plans/{task_id}-plan.md
  3. {root}/sunnycore/templates/review-tmpl.yaml
  </context>
  <templates>
  1. review-tmpl.yaml
  2. dev-notes-tmpl.yaml
  </templates>
</input>

<output>
1. {root}/docs/review-results/{task_id}-review.md
2. {root}/docs/tasks.md
</output>

<constraints, importance = "Important">
- Must produce machine-checkable Markdown with sections: Overview, Findings, Risks, Action Items.
- Must cross-reference plan/code/notes with file paths, line ranges, or anchors when available.
- Must prioritize requirement mismatches and critical defects over style issues.
- Should keep each finding concise (<= 120 words) and one issue per bullet.
- Must record an acceptance decision with rationale: Accept / Accept with changes / Reject.
</constraints>

<workflow, importance = "Important">
  <stage id="0: plan-todos">
  <tools: todo-list>
  - Read all working steps
  - Create one todo item per stage
  </tools: todo-list>
  </stage>

  <stage id="1: review-plan">
  <tools: sequential-thinking>
  - Read and understand the implementation plan
  - Identify verification approach and success criteria
  </tools: sequential-thinking>

  <questions>
  - Are acceptance criteria complete, testable, and measurable?
  - Are assumptions, risks, and rollback strategies explicitly stated?
  </questions>
  </stage>

  <stage id="2: review-code">
  <tools: sequential-thinking>
  - Read and understand the code
  - Check alignment with the plan and the requirements
  </tools: sequential-thinking>
  
  <questions>
  - Do tests cover critical paths, edge cases, and regressions?
  - Are security, performance, and observability concerns addressed?
  </questions>
  </stage>

  <stage id="3: review-dev-notes">
  <tools: sequential-thinking>
  - Read and understand the development notes
  - Check alignment between notes and implementation
  </tools: sequential-thinking>
  </stage>

  <stage id="4: produce-results">
  - Use the template to create the markdown formatted review results
  - Save to {root}/docs/review-results/{task_id}-review.md
  - If there is already an existing review results, update the review results with the new information
  <checks>
  - [ ] All required sections present and consistent
  - [ ] Findings reference plan/code/notes with links or anchors
  - [ ] Acceptance decision recorded with rationale
  - [ ] Action items prioritized and assignable
  </checks>
  </stage>
</workflow>

<example>
Minimal review result outline:

# Overview
- Scope: ...
- Decision: Accept with changes — rationale: ...

# Findings
- Issue: ...
- Evidence: ...
- Impact: ...
- Recommendation: ...

# Risks
- ...

# Action Items
- [P1] ...
</example>

<example>
markdown文件輸出方式：
	•	YAML 第一層 key 轉換為 Markdown 一級標題 (#)
	•	YAML 第二層 key 轉換為 Markdown 二級標題 (##)
	•	YAML 第三層 key 轉換為 Markdown 三級標題 (###)
	•	YAML value（字串或數字） 轉換為 Markdown 正文文字
</example>

