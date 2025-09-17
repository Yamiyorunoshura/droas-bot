<input>
  <context>
  1. {root}/docs/requirements/*.md
  2. {root}/docs/architecture/*.md
  3. {root}/docs/tasks.md
  </context>
  <templates>
  4. {root}/sunnycore/templates/implementation-plan-tmpl.yaml
  </templates>
</input>

<output>
1. {root}/docs/implementation-plan/{task_id}-plan.md
</output>

<constraints, importance = "Important">
- Derive tasks strictly from provided documents; do not invent requirements.
- Map every plan item to requirement IDs and architecture sections.
- Use Markdown with ATX headings; prefer numbered lists and concise bullets.
- Produce exactly one file at the specified output path.
</constraints>

<workflow, importance = "Important">
  <stage id="0: setup">
  <tools: todo-list>
  - Read all working steps
  - Create a todo item for each working step
  </tools: todo-list>
  </stage>

  <stage id="1: functional">
  <tools: sequential-thinking>
  - Identify functional requirements
  - Map each to the corresponding architecture component(s)
  - Draft an implementation plan for each functional requirement
  </tools: sequential-thinking>

  <questions>
  - Are requirement IDs and architecture references complete and unambiguous?
  - Are cross-cutting concerns (auth, logging, i18n) captured where applicable?
  </questions>
  </stage>

  <stage id="2: non-functional">
  <tools: sequential-thinking>
  - Identify non-functional requirements
  - Map each to the corresponding architecture capability
  - Create tasks and acceptance criteria for each non-functional requirement
  </tools: sequential-thinking>
  
  <questions>
  - Do tasks include measurable targets (latency, availability, security)?
  - Are dependencies and sequencing clear to avoid blocking?
  </questions>
  </stage>

  <stage id="3: finalize">
  - Use the template to finalize the plan
  - Output the Markdown plan to {root}/docs/implementation-plan/{task_id}-plan.md
  <checks>
  - [ ] All functional and non-functional items mapped to architecture
  - [ ] Each plan item includes tasks, owners (if known), and acceptance criteria
  - [ ] Markdown uses ATX headings and numbered lists
  - [ ] Output path and file naming follow the specified pattern
  </checks>
  </stage>
</workflow>