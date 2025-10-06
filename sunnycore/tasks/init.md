[Input]
  1. "{root}/docs/architecture/*.md" --Architecture design documents (extract technology stack, development environment requirements)
  2. "{root}/docs/requirements/*.md" --Project requirement documents
  3. "{root}/docs/epic.md" --Task list

[Output]
  1. "{root}/CLAUDE.md" --Project guidance document (including technology stack, development standards, requirement overview, project goals, document index)
  2. Initialized development environment (such as virtual environment, database, toolchain, etc.)

[Constraints]
  1. Must extract all technology stack and environment requirements from architecture documents
  2. Must create document index for convenient lookup of architecture and requirement documents during subsequent development
  3. Environment initialization must be based on explicit requirements in architecture documents, do not add undefined tools or configurations
  4. "{root}/CLAUDE.md" must include complete technical information, requirement overview, project goals, and document index

[Tools]
  1. **todo_write**
    - [Step 1: Create todo list; Steps 2-4: Track execution progress]
  2. **claude-context (MCP)**
    - [Step 2: Build index for codebase]

[Steps]
  1. Preparation Phase
    - Read all architecture, requirement, and task documents
    - Identify required technology stack and development environment configuration
    - Create todo list to track execution progress of Steps 2-4

  2. Environment Initialization Phase
    - Set up development environment according to architecture documents
      * Create virtual environment (such as Python venv, Node.js environment, etc.)
      * Install necessary toolchain (such as compilers, package managers, etc.)
      * Configure databases (such as PostgreSQL, Redis, etc.)
      * Install project dependencies
    - Verify environment configuration correctness
    - Record any environment configuration issues or dependency conflicts
    - Build index for codebase

  3. Document Generation Phase
    - Create "{root}/CLAUDE.md" project guidance document
    - Include the following sections:
      * Technology Stack: Technology selections extracted from architecture documents
      * Development Standards: Coding style, testing requirements, etc.
      * Requirement Overview: Core requirement summary extracted from requirement documents
      * Project Goals: Main goals and success criteria of the project
      * Document Index: List locations and purposes of all architecture and requirement documents
    - Ensure document format is clear and readable

  4. Verification Phase
    - Confirm development environment has been successfully initialized and is usable
    - Verify "{root}/CLAUDE.md" has been created and contains complete information
    - Check all DoD items one by one to ensure they are met
    - Confirm all todo items are completed

[DoD]
  - [ ] All architecture, requirement, and task documents have been read
  - [ ] Development environment has been initialized and is usable (all necessary tools are installed and configured correctly)
  - [ ] Codebase index has been built
  - [ ] "{root}/CLAUDE.md" has been created and includes all required sections
  - [ ] All todo items are completed
