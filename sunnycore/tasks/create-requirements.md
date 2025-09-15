<input>
  <context>
  1. Transform initial ideas into complete, testable requirements through a structured, interactive workflow.
  2. Use the canonical requirement template to ensure consistency and verifiability.
  3. Maintain English-only prompt content; user communications may be multilingual.
  </context>
  <templates>
  1. {root}/warp code/templates/requirement-tmpl.yaml - Standard requirement template
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
</constraints>

<workflow importance="Important">
  <stage id="1, init">
  - Read all working steps to understand the expected deliverables.
  - Create an internal TODO item for each working step.
  </stage>

  <stage id="2, functional">
  - Derive functional requirements from the user's input and context.
  - Consolidate duplicates and remove ambiguity; keep each statement atomic.
  - Organize by user stories or system capabilities as appropriate.

  <questions>
  - What are the core user goals and success criteria for each scenario?
  - Which preconditions, data contracts, and error states must be explicit?
  - Are there external integrations or upstream/downstream dependencies?
  </questions>
  </stage>

  <stage id="3, nonfunctional">
  - Identify non-functional requirements across performance, reliability, security, compliance, and operability.
  - Quantify targets (e.g., P95 latency, uptime SLO, RTO/RPO) and constraints.
  - Map NFRs to monitoring/observability signals when relevant.

  <questions>
  - What performance baselines and environment assumptions are in scope?
  - What threat model, data classification, or compliance scope applies?
  - How will we validate and observe these NFRs post-deployment?
  </questions>
  </stage>

  <stage id="4, acceptance">
  - Define acceptance criteria per requirement; ensure they are deterministic and testable.
  - Reference inputs, preconditions, and explicit pass/fail outcomes.
  - Use clear structure; Gherkin-style is acceptable if helpful.
  </stage>

  <stage id="5, finalize">
  - Populate the requirement template and place outputs under {root}/docs/requirements/.
  - Cross-check consistency across FRs, NFRs, and acceptance criteria.
  - Run the {root}/sunnycore/scripts/shard-requirements.sh script to shard the requirements

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