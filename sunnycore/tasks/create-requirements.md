<input>
  <context>
  1. Transform initial ideas into complete, testable requirements through a structured, interactive workflow.
  2. Use the canonical requirement template to ensure consistency and verifiability.
  3. Maintain English-only prompt content; user communications may be multilingual.
  </context>
  <templates>
  1. {root}/sunnycore/templates/requirement-tmpl.yaml - Standard requirement template
  </templates>
</input>

<output>
1. {root}/docs/requirements/*.md - Complete requirement specifications including functional, non-functional, and acceptance criteria.
</output>

<constraints importance="Important">
- Each requirement must be verifiable and measurable; avoid ambiguous or subjective phrasing.
- Use numbered lists and concise sentences (average < 20 words) for scanability.
- Align sections and field names exactly with the requirement template.
- Do not include sensitive or personal data in examples.
- Clarify the requirements and the acceptance criteria with the user.
</constraints>

<workflow importance="Important">
  <stage id="1, init">
  <tools: todo-list>
  - Read all working steps to understand the expected deliverables.
  - Create a todo item for each working stage
  </tools: todo-list, sequential-thinking>
  </stage>

  <stage id="2, functional">
  <tools: sequential-thinking>
  - Derive functional requirements from the user's input and context.
  - Consolidate duplicates and remove ambiguity; keep each statement atomic.
  - Organize by user stories or system capabilities as appropriate.
  </tools: sequential-thinking>

  <questions>
  - What are the core user goals and success criteria for each scenario?
  - Which preconditions, data contracts, and error states must be explicit?
  - Are there external integrations or upstream/downstream dependencies?
  </questions>
  </stage>

  <stage id="3, nonfunctional">
  <tools: sequential-thinking>
  - Identify non-functional requirements across performance, reliability, security, compliance, and operability.
  - Quantify targets (e.g., P95 latency, uptime SLO, RTO/RPO) and constraints.
  - Map NFRs to monitoring/observability signals when relevant.
  </tools: sequential-thinking>
  
  <questions>
  - What performance baselines and environment assumptions are in scope?
  - What threat model, data classification, or compliance scope applies?
  - How will we validate and observe these NFRs post-deployment?
  </questions>
  </stage>

  <stage id="4, acceptance">
  <tools: sequential-thinking>
  - Define acceptance criteria per requirement; ensure they are deterministic and testable.
  - Reference inputs, preconditions, and explicit pass/fail outcomes.
  - Use clear structure; Gherkin-style is acceptable if helpful.
  </tools: sequential-thinking>
  </stage>

  <stage id="5, finalize">
  - Cross-check consistency across FRs, NFRs, and acceptance criteria and ask for user's confirmation.
  - Populate the requirement template and place markdown formatted outputs under {root}/docs/requirements/.
  - Run the {root}/sunnycore/scripts/shard-requirements.py script by using uv run to shard the requirements

  <checks>
  - [ ] Outputs include FRs, NFRs, and acceptance criteria per template
  - [ ] All content is written in English and uses numbered lists where applicable
  - [ ] Each requirement is measurable and verifiable (no ambiguity)
  - [ ] File paths and names match {root}/docs/requirements/*.md
  - [ ] Sensitive data has not been introduced in examples
  - [ ] The requirements are sharded into {root}/docs/requirements/*.md
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
- [ ] stage 0: Create a todo list
- [ ] stage 1: Functional
- [ ] stage 2: Nonfunctional
- [ ] stage 3: Acceptance
- [ ] stage 4: Finalize
</example>