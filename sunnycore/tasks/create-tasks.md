<input>
  <context>
    1. {root}/docs/requirements/*.md - Project functional and non-functional requirements
    2. {root}/docs/architecture/*.md - Architecture design and technical specifications
    3. {root}/sunnycore/templates/tasks-tmpl.yaml - Task template format
  </context>
  <templates>
    1. sunnycore/templates/tasks-tmpl.yaml - Standard task structure template
  </templates>
</input>

<output>
  1. {root}/docs/tasks.md - Comprehensive task breakdown with atomized sub-tasks
</output>

<constraints, importance = "Important">
- Use 2-space indentation and kebab-case keys in YAML.
- Create atomic, verifiable tasks (≤14 words, clear outcome).
- Exclude operational actions unless explicitly requested by the user.
- Ensure no filenames/paths use spaces; prefer kebab-case.
</constraints>

<workflow importance="Important">
  <stage id="research">
  <tools: sequential-thinking>
  - Read requirements and architecture sources.
  - Identify scope, success criteria, and constraints to drive task design.
  - Map non-functional requirements to cross-cutting tasks.
  </tools: sequential-thinking>
  
  <questions>
  - Are all FRs and NFRs discoverable and current?
  - What dependencies or sequencing constraints exist between tasks?
  - What are acceptance signals for each task?
  </questions>
  </stage>

  <stage id="draft">
  - Use the template to generate atomic tasks.
  - Include brief acceptance hints for verifiability.
  - Group tasks logically while avoiding overlaps.
  </stage>

  <stage id="review">
  - De-duplicate and prune non-actionable items.
  - Ensure traceability from each task to requirement(s).
  - Validate formatting against the template.
  </stage>

  <stage id="finalize">
  - Write markdown format tasks to {root}/docs/tasks.md.
  - Include a short introduction explaining grouping and scope.

  <checks>
  - [ ] File {root}/docs/tasks.md exists and is valid Markdown.
  - [ ] Tasks conform to template fields and 2-space indentation.
  - [ ] Each task is atomic, outcome-oriented, and verifiable.
  - [ ] No filenames or keys use spaces; kebab-case enforced.
  </checks>
  </stage>
</workflow>