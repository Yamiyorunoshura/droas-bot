<input>
  <context>
    1. {root}/docs/requirements - Canonical requirements source
    2. {root}/docs/architecture/*.md - Existing architecture corpus
    3. {root}/sunnycore/scripts/shard-architecture.sh - Architecture sharding script
  </context>
  <templates>
    {root}/sunnycore/templates/architecture-tmpl.yaml
  </templates>
</input>

<output>
1. {root}/docs/architecture/*.md - Updated architecture documentation set
</output>

<constraints importance="Important">
- Thoroughly review requirements and the extant architecture before proposing designs.
- New modules must integrate without violating existing contracts; include an explicit impact analysis for any proposed changes.
- Rigorously follow the template structure and section order.
- Draft into {root}/docs/architecture.md, then execute {root}/warp code/scripts/shard-architecture.sh to shard.
- Use clear, concise English and 2-space indentation.
</constraints>

<workflow importance="Critical">
  <stage id="0: plan-todos">
  <tools: todo-list>
  - Review all working steps comprehensively.
  - Instantiate TODO items for each step.
  </tools: todo-list>
  </stage>

  <stage id="1: assess-existing">
  <tools: sequential-thinking>
  - Examine the current architecture under {root}/docs/architecture/*.md.
  - Identify extension points, constraints, and shared services.
  - Map affected domains, bounded contexts, and dependencies.
  </tools: sequential-thinking>
  </stage>

  <stage id="2: design-new-modules">
  <tools: sequential-thinking, context7>
  - Define responsibilities, boundaries, and interfaces for new modules.
  - Specify data flows and interactions with existing components.
  - Evaluate non-functional requirements (security, observability, performance) and compatibility.
  </tools: sequential-thinking, context7>
  
  <questions>
  - Which extension points or APIs will the new modules depend on?
  - Which contracts must be preserved to preclude regressions?
  - What data ownership or consistency implications emerge?
  </questions>
  </stage>

  <stage id="3: author-and-shard">
  - Use the architecture template to draft markdown formatted {root}/docs/architecture.md.
  - Ensure sections emphasize new modules and integration impacts.
  - Run the sharding script to split the document:
    - bash '{root}/sunnycore/scripts/shard-architecture.sh'
  - Verify files appear under {root}/docs/architecture/.
  </stage>

  <stage id="4: finalize">
  - Cross-check against constraints and guiding questions; remedy gaps and inconsistencies.
  
  <checks>
  - [ ] New modules documented with boundaries, interfaces, and data flows
  - [ ] Compatibility with existing contracts articulated (no breaking changes)
  - [ ] {root}/docs/architecture.md exists and adheres to the template
  - [ ] Sharded files generated under {root}/docs/architecture/
  - [ ] Paths and commands reference {root}/warp code/scripts/shard-architecture.sh
  </checks>
  </stage>
</workflow>