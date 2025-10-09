**GOAL**: Initialize development environment and project documentation structure.

## [Input]
  1. "{ARCH}/*.md" --Architecture design documents (extract technology stack, development environment requirements)
  2. "{REQ}/*.md" --Project requirement documents
  3. "{EPIC}" --Task list

## [Output]
  1. "{root}/CLAUDE.md" --Project guidance document (including technology stack, development standards, requirement overview, project goals, document index)
  2. Initialized development environment (such as virtual environment, database, toolchain, etc.)

## [Constraints]
  1. Do not add tools or configurations not explicitly required in architecture documents
  2. Do not skip creating document index in CLAUDE.md
  3. Do not skip codebase indexing for Brownfield projects

## [Tools]
  1. **todo_write**
    - [Step 1: Create todo list; Steps 2-4: Track execution progress]
  2. **claude-context (MCP)**
    - [Step 2: When project is Brownfield, execute codebase indexing]
    - [Step 2: Semantic search existing architecture implementations to understand tech stack, module structure, and dependencies]
    - Query examples: "Where is the tech stack configured?" "How are main modules organized?" "How is the development environment set up?"
  3. **context7 (MCP)**
    - [Step 2: Query official documentation for tech stack mentioned in architecture documents and environment setup guides]
    - When to use: After extracting tech stack from {ARCH}/*.md, query official installation and configuration docs for corresponding versions

## [Steps]
  1. Preparation
  - Task: Understand project's technology stack, environment requirements, and goals
  - Expected outcome: Progress tracking mechanism established for initialization tasks

  2. Environment Setup
  - Task: Configure development environment and dependencies
  - Expected outcome: Fully operational environment with indexed codebase for efficient searching

  3. Document Generation
  - Task: Generate project guidance document at "{root}/CLAUDE.md"
  - Expected outcome: Complete CLAUDE.md with all required sections and clear document index

  4. Verification
  - Task: Verify development environment and documentation
  - Expected outcome: Environment successfully initialized and CLAUDE.md contains complete information

## [Initialization-Guidelines]
  1. **Extract from Architecture**
    - Identify all technology stack, tools, and environment requirements explicitly stated in architecture docs
    - Do not add undefined tools or configurations
    - Set up only what's documented in requirements and architecture
  
  2. **CLAUDE.md Structure**
    - Include: technology stack, development standards, requirement overview, project goals, document index
    - Create comprehensive document index for architecture and requirement documents
    - Enable easy navigation for subsequent development tasks
  
  3. **Codebase Indexing**
    - Build searchable index of existing codebase (if Brownfield)
    - Enable efficient searching during development phases

## [DoD]
  - [ ] Development environment fully initialized and operational with all tools installed and verified
  - [ ] Codebase indexed and ready for semantic search (if Brownfield)
  - [ ] Complete CLAUDE.md exists with all required sections (tech stack, standards, requirements overview, document index)

## [Example]

### Example 1: Microservices Platform with Docker
[Input]
- Architecture: docs/architecture/tech-stack.md specifies Node.js, PostgreSQL, Redis, Docker
- Requirements: docs/requirements/*.md (3 services to implement)
- Epic: docs/epic.md (5 tasks)

[Decision]
- Environment setup: Install Docker, docker-compose, Node.js v18, PostgreSQL client
- Initialize: Create docker-compose.yml with services (postgres, redis, api)
- Index codebase: Run claude-context index for existing shared libraries
- CLAUDE.md: Include tech stack, document index, development commands

[Expected Outcome]
- Working environment: docker-compose up runs all services
- Codebase indexed for semantic search
- CLAUDE.md with: tech stack (Node.js, Docker), document index (architecture/*, requirements/*), dev standards (TDD, SOLID)

### Example 2: Python Data Pipeline
[Input]
- Architecture: docs/architecture/*.md specifies Python 3.11, Airflow, Spark, S3
- Requirements: docs/requirements/*.md (ETL pipeline requirements)
- Epic: docs/epic.md (4 data processing tasks)

[Decision]
- Environment: Setup Python venv, install Airflow, PySpark, AWS CLI
- Initialize: Configure Airflow DAGs folder, S3 bucket connections
- Index: Build codebase index for existing data transformers
- CLAUDE.md: Document Airflow setup, data flow diagrams, S3 bucket structure

[Expected Outcome]
- Airflow webserver accessible at localhost:8080
- S3 connections configured and tested
- CLAUDE.md with: Python stack, Airflow DAG index, data pipeline overview, testing approach

### Example 3: React Native Mobile App
[Input]
- Architecture: docs/architecture/*.md specifies React Native, Expo, Firebase
- Requirements: docs/requirements/*.md (mobile app features)
- Epic: docs/epic.md (6 screen implementation tasks)

[Decision]
- Environment: Install Node.js, Expo CLI, Android Studio, Xcode (macOS)
- Initialize: expo init, Firebase project setup, configure app.json
- Index: Index existing component library and navigation structure
- CLAUDE.md: Document app structure, navigation flow, Firebase config, testing setup

[Expected Outcome]
- App runs on iOS/Android simulators via expo start
- Firebase connected (auth, firestore configured)
- CLAUDE.md with: React Native stack, screen navigation index, component library docs, Firebase setup guide
