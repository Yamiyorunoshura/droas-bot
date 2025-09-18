<input>
  <context>
  1. {root}/docs/requirements/*.md - Authoritative project requirements
  2. {root}/sunnycore/scripts/shard-architecture.py - Architecture sharding script
  </context>
  <templates>
  1. {root}/sunnycore/templates/architecture-tmpl.yaml - Canonical architecture template
  </templates>
</input>

<output>
1. {root}/docs/architecture/*.md - Comprehensive architecture corpus
</output>

<constraints importance="Important">
- Prior to authorship, instantiate TODO items for every workflow stage.
- Exhaustively review all requirements under {root}/docs/requirements/*.md and determine the macro-architecture and critical interaction patterns before drafting.
- Author the draft with the architecture template into {root}/docs/architecture.md; comprehensively cover components, boundaries, data flows, risks, and decisions; strictly preserve section ordering.
- Post-draft, execute the sharding script: bash '{root}/warp code/scripts/shard-architecture.py'; verify materialized files under {root}/docs/architecture/.
- Use clear, concise English and 2-space indentation; refrain from introducing non-existent paths or files.
</constraints>

<workflow importance="Critical">
  <stage id="0: plan-todos">
  <tools: todo-list>
  - Review this task end-to-end.
  - Create a todo item for each working stage
  </tools: todo-list, sequential-thinking>
  </stage>

  <stage id="1: research">
  <tools: sequential-thinking, context7>
  - Digest all requirements under {root}/docs/requirements/*.md.
  - Delineate components, boundaries, and canonical data flows.
  - Converge on the macro-architecture and principal interaction contracts.
  </tools: sequential-thinking, context7>

  <questions>
  - Are the requirements complete, current, and authoritative?
  - Which cross-cutting concerns (security, observability, performance) are mandatory, and how will they be enforced?
  - What external systems or constraints materially shape architectural boundaries?
  </questions>
  </stage>

  <stage id="2: author">
  - Leverage the architecture template to draft markdown formatted {root}/docs/architecture.md.
  - Ensure sections comprehensively cover components, interactions, data flows, risks, and decisions.
  - Execute the sharding script to partition the document: uv run '{root}/sunnycore/scripts/shard-architecture.py'
  - Verify artifacts appear under {root}/docs/architecture/.
  </stage>

  <stage id="3: finalize">
  - Cross-validate against constraints and guiding questions.
  - Remediate typographical errors and normalize terminology.
  
  <checks>
  - [ ] {root}/docs/architecture.md exists and adheres to the template
  - [ ] Sharded files are generated under {root}/docs/architecture/
  - [ ] Section order and headings exactly match the template
  - [ ] Paths and commands are valid within this repository
  - [ ] Language is clear, concise, and free of typographical errors
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
- stage 0: Create a todo list
- stage 1: Research
- stage 2: Author
- stage 3: Finalize
</example>