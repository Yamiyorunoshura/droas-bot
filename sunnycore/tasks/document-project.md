[Input]
  1. "{root}/docs/architecture/*.md" --Existing architecture documents (as context)
  2. "{root}/sunnycore/templates/concluded-architecture-tmpl.yaml" --Architecture document template
  3. "{root}/docs/knowledge/*.md" --Knowledge base
  4. "{root}/docs/progress.md" --Progress record
  5. Actual code

[Output]
  1. "{root}/docs/architecture/*.md" --Architecture documents (Markdown format)
  2. "{root}/CLAUDE.md" --Updated project guidance document with refreshed document index

[Constraints]
  1. Must produce valid JSON that fully complies with schema (additionalProperties=false)
  2. When producing final deliverables, no text descriptions are allowed outside of JSON
  3. Should use a 3-phase workflow, with checklist items placed in the final phase
  4. Every produced document must correspond to at least 1 source reference (source_refs)
  5. Can include architecture diagrams in Markdown documents as fenced code blocks
  6. All file paths must be under "docs/architecture/" with .md extension
  7. Must update the Document Index section in "{root}/CLAUDE.md" after creating architecture documents

[Tools]
  1. **todo_write**
    - [Step 1: Create task list; Steps 2-3: Track task progress]
  2. **sequentialthinking (MCP)**
    - [Step 2: Reasoning tasks for content integration and document writing]

[Steps]
  1. Analysis and Planning Phase
    - Read all input documents
    - Inventory architecture source documents to record gaps and assumptions
    - Plan target document section structure according to template
    - Create todo list based on actual tasks
    - Deliverable: Task list and source document list

  2. Write Architecture Documents Phase
    - Integrate and standardize content to write documents according to template
    - Verify document completeness and internal consistency (check item by item: whether all necessary sections are covered, whether documents trace back to source references, whether document paths are under "docs/architecture/" and in .md format)
    - save the temporary document as "{root}/docs/architecture.md"

  3. Sharding and Finalization Phase
    - Execute "{root}/sunnycore/scripts/shard-architecture.py" and record shards_created count
    - Update Document Index section in "{root}/CLAUDE.md" with newly created architecture documents (including path and purpose of each document)
    - Verify generated JSON complies with schema; if violations exist, should fix immediately
    - Only produce final JSON, without any explanatory text
    - Check all DoD items one by one to ensure they are met
    - Confirm all todo items are completed
    - Deliverable: Final JSON deliverable including shards_created

[DoD]
  - [ ] All input documents have been read
  - [ ] Task list has been created and all items are marked as completed
  - [ ] All architecture documents have been written according to template
  - [ ] Each produced document includes at least 1 source reference (source_refs)
  - [ ] All file paths are under "docs/architecture/" with .md extension
  - [ ] "shard-architecture.py" has been executed and shards_created recorded
  - [ ] "{root}/CLAUDE.md" Document Index section has been updated with newly created architecture documents
  - [ ] Generated JSON complies with schema (no additionalProperties violations)
  - [ ] Final production includes only JSON, no additional explanatory text
