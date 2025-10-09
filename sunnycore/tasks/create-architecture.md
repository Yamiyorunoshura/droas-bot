**GOAL**: Create comprehensive technical architecture documentation for Greenfield projects.

## [Input]
  1. "{REQ}/*.md" --Authoritative project requirements
  2. "{SCRIPTS}/shard-architecture.py" --Architecture sharding script
  3. "{TMPL}/architecture-tmpl.yaml" --Standardized architecture template

## [Output]
  1. Architecture document collection under "{ARCH}/" directory (*.md format)
  2. "{root}/docs/architecture.md" (temporary architecture file. Will be sharded after running "shard-architecture.py")

## [Constraints]
  1. Do not make assumptions when requirements are incomplete or conflicting (must record issues and confirm)
  2. Do not produce architecture that lacks 100% requirement coverage
  3. Do not deviate from template structure or introduce non-existent paths
  4. Do not skip executing shard-architecture.py

## [Tools]
  1. **todo_write**
    - [Step 1: Track and update execution tasks; Steps 2-4: Track writing progress and results]
  2. **sequential-thinking (MCP)**
    - [Step 1: Decompose requirements and identify applicable architecture patterns]
    - [Step 2: Design system components and boundaries, verify design decisions]
    - When to use: When need to evaluate multiple architecture options or perform pros/cons analysis
    - [Step 3: Structured drafting of architecture document, ensure logical consistency across sections]
    - [Step 4: Verify requirement-to-architecture mapping coverage]
  3. **context7 (MCP)**
    - [Step 2: Query official documentation for external packages and architecture pattern best practices]
    - When to use: When need to verify tech stack version compatibility or find official architecture guidance
  4. **playwright (MCP)**
    - [Step 1-2: Research similar system architecture designs and implementation cases]
    - When to use: When need to study architecture patterns from open-source projects or industry references

## [Steps]
  1. Requirement Analysis
  - Task: Analyze and understand all requirements and their implications
  - Expected outcome: Complete requirement-to-architecture mapping matrix with 100% coverage

  2. Architecture Design
  - Task: Design system architecture with all components and boundaries defined
  - Expected outcome: Every requirement maps to specific architecture elements, all cross-cutting concerns properly addressed

  3. Documentation and Approval
  - Task: Write architecture document and obtain user approval
  - Expected outcome: Compliant architecture draft following template structure, successfully execute shard-architecture.py script

  4. Final Verification
  - Task: Verify architecture satisfies all requirements
  - Expected outcome: Consistent and justified architecture decisions

## [Error-Handling]
  1. "shard-architecture.py" execution failure: Check if "architecture.md" format complies with template specifications, fix and re-execute
  2. Unresolvable requirement conflicts: Record conflicts and confirm with requirement authors, do not make assumptions
  3. Architecture design infeasibility: Record technical limitations and propose alternative solutions

## [Architecture-Design-Guidelines]
  1. **Template Compliance & Complete Mapping**
    - Follow template structure strictly (overview, components, data flows, ADRs)
    - Establish 100% bidirectional mapping: requirements ↔ architecture elements
    - Verify all functional and non-functional requirements have corresponding architecture design
  
  2. **Decision Records (ADR)**
    - Document key decisions with context, rationale, alternatives, and trade-offs
    - Explain why this approach was chosen over others
    - Include expected impact and potential technical debt
  
  3. **Cross-Cutting Concerns**
    - Address security, observability, performance, reliability systematically
    - Define consistency mechanisms (authentication, logging, error handling) across components
    - Transform non-functional requirements into concrete architecture constraints
  
  4. **Traceability & Justification**
    - Annotate source references for all architecture statements
    - Ensure every decision is justified by requirements or business drivers
    - Use clear terminology consistently throughout documentation

## [DoD]
  - [ ] Complete requirement-to-architecture mapping matrix with 100% coverage (all functional and non-functional requirements mapped)
  - [ ] Architecture document approved by user and exists at "{root}/docs/architecture.md" following template structure
  - [ ] Architecture successfully sharded with verified output files in "{ARCH}/"

## [Example]

### Example 1: Real-time Chat Application
[Input]
- Requirements: REQ-001 (WebSocket messaging), REQ-002 (message persistence), NFR-001 (< 100ms latency)
- Template: architecture-tmpl.yaml

[Decision]
- Components: WebSocket Gateway, Message Broker (Redis), Message Store (MongoDB)
- ADR-001: Choose WebSocket over HTTP polling for real-time performance
- Mapping: REQ-001 → WebSocket Gateway, REQ-002 → Message Store

[Expected Outcome]
- docs/architecture.md with components, data flows, ADRs
- docs/architecture/overview.md, components.md, traceability_matrix.md (after sharding)
- 100% requirement-to-architecture mapping verified

### Example 2: IoT Device Management Platform
[Input]
- Requirements: REQ-001 (device registration), REQ-002 (telemetry ingestion), NFR-001 (handle 10K devices)
- Template: architecture-tmpl.yaml

[Decision]
- Components: Device Registry (PostgreSQL), Telemetry Ingester (Kafka), Time-Series DB (InfluxDB)
- ADR-001: Event-driven architecture for scalability
- Cross-cutting: Authentication (JWT), Monitoring (Prometheus)

[Expected Outcome]
- docs/architecture.md with event-driven design and scalability considerations
- Sharded files showing clear component boundaries and data flows
- Traceability matrix linking all requirements to architecture elements

### Example 3: Financial Transaction Processing
[Input]
- Requirements: REQ-001 (payment processing), REQ-002 (fraud detection), NFR-001 (ACID compliance)
- Template: architecture-tmpl.yaml

[Decision]
- Components: Payment Gateway, Fraud Analyzer (ML service), Transaction DB (PostgreSQL)
- ADR-001: Synchronous processing for ACID guarantees
- ADR-002: Two-phase commit for distributed transactions

[Expected Outcome]
- docs/architecture.md emphasizing reliability and data consistency
- Architecture documents with detailed ADRs explaining transaction guarantees
- Complete mapping of functional and non-functional requirements

Work directory structure example:

```
{project_root}/
├── {config_dir}/                # {config_description}
│   ├── {settings_dir}/         # {settings_description}
│   └── {constants_dir}/        # {constants_description}
├── {src_dir}/                  # {src_description}
│   ├── {core_dir}/            # {core_description}
│   ├── {api_dir}/             # {api_description}
│   ├── {models_dir}/          # {models_description}
│   └── {utils_dir}/           # {utils_description}
├── {tests_dir}/               # {tests_description}
│   ├── {unit_dir}/           # {unit_description}
│   └── {integration_dir}/    # {integration_description}
├── {docs_dir}/               # {docs_description}
│   ├── {requirements_dir}/   # {requirements_description}
│   ├── {architecture_dir}/   # {architecture_description}
│   └── {guides_dir}/         # {guides_description}
└── {readme_file}             # {readme_description}
```
