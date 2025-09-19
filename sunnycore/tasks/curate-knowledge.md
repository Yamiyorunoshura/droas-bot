<input>
  <context>
  1. {root}/docs/review-results/*.md - Code Review results and practice level assessments.
  2. {root}/docs/dev-notes/*.md - Development Notes capturing error cases and fixes.
  </context>
  <templates>
  1. {root}/sunnycore/project-knowledge-tmpl.yaml - Project knowledge base structure template.
  </templates>
</input>

<output>
1. {root}/docs/project-knowledge.md - Consolidated Project Knowledge Base summarizing Error Cases and Best Practices with source references.
</output>

<constraints importance="Important">
- Derive content strictly from provided files; do not invent information.
- Cite source file paths for each best practice and error case.
- Align sections and fields to the template structure.
- Prefer concise bullet points; average sentence length < 20 words.
</constraints>

<example>
<snippet>
# Best Practices
- Retry external requests with exponential backoff (source: docs/review-results/service-a.md).

# Error Cases
- Null pointer due to async race in handler.
  - Root cause: early access before promise resolution.
  - Fix: guard with null checks; await dependencies.
  - Source: docs/dev-notes/2025-05-10-null-pointer.md
</snippet>
</example>

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
- [ ] stage 1: Conclude the review results
- [ ] stage 2: Conclude the development notes
- [ ] stage 3: Curate the knowledge
</example>

<workflow importance="Important">
  <stage id="0: plan">
  <tools: todo-list>
  - Read all working steps.
  - Take reference from the example and create a todo item 
  </tools: todo-list, sequential-thinking>
  </stage>

  <stage id="1: conclude_review_results">
  <tools: sequential-thinking>
  - Read the review results.
  - Extract best practices and error cases with file references.
  - Summarize patterns and practice levels if available.
  </tools: sequential-thinking>

  <questions>
  - Are there repeated failure patterns across files?
  - Which practices have clear acceptance criteria?
  </questions>
  </stage>

  <stage id="2: conclude_development_notes">
  <tools: sequential-thinking>
  - Read the development notes.
  - Extract best practices and error cases with file references.
  - Normalize naming for similar issues.
  </tools: sequential-thinking>
  </stage>

  <stage id="3: curate_knowledge">
  - Use the project knowledge template to structure content.
  - Output to {root}/docs/project-knowledge.md in Markdown.
  - Ensure cross-references and links resolve.

  <checks>
  - [ ] All required sections from the template are present.
  - [ ] Each item cites at least one source file path.
  - [ ] No unverifiable claims or missing references.
  - [ ] Links and paths resolve within the repo.
  </checks>
  </stage>
</workflow>