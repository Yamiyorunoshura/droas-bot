# General rules

## Output style

### Code-Style Oriented
- **Template-Driven**: Use standardized templates (`templates/` directory) to ensure document format consistency
- **Code Standards**:
  - Markdown: ATX headings (`#`), sentence-case titles, concise sections
  - YAML: 2-space indentation, kebab-case keys (e.g. `dev-subagent-list`)
  - Shell: POSIX-compatible bash, prefer functions + `set -e` for robustness
  - File/folder naming: kebab-case, no spaces

### Documentation Consistency
- Primary language: Traditional Chinese documentation and communication
- Technical terms: Chinese-English correspondence for accuracy
- Professional communication standards with structured XML output

## Behaviour

### Task Execution
- You must complete the tasks by following the unordered list items under each working stage

### Tool Usage Restrictions
**Core Principle**: Agents must not use any mcp or todo-list tools unless explicitly specified by XML tags in the prompt

#### Conditional Tool Activation
- Tools may only be used when the corresponding XML tags are present:

### Execution Mode
1. **Context Validation**: Read and verify all input files
2. **Task Categorization**: Assign to appropriate agents based on domain expertise
3. **Collaborative Execution**: Parallel task execution with specialized agents
4. **Structured Output**: Generate standardized documentation using templates

### File Structure Specifications
```
{root}/
├── .claude/
|    ├── commands/        # Custom command definitions  
├── sunnycore/ 
    ├── tasks/           # Task execution workflows
    ├── templates/       # Standardized templates
    ├── scripts/         # Utility scripts
    └── CLAUDE.md        # Agent system configuration
```

### Naming Conventions
- File names: kebab-case, no spaces (avoid introducing new spaced naming)
- Task IDs: Correspond to file names, use hyphen separation
- Agent IDs: Short identifiers (dev, pm, po, qa)
- Template IDs: Descriptive names ending with `-tmpl`

# QA rules

## 7-Dimensional Review Standards

### Review Dimension Framework
Each review task must be systematically evaluated based on the following 7 dimensions:

#### 1. Functional Requirements Compliance
**Assessment Focus**:
- Requirements traceability validation
- Acceptance criteria completeness check  
- Business logic correctness review

**Quality Indicators**:
- Bronze: Basic functionality implemented, some requirements not covered
- Silver: Main functionality complete, edge cases handled properly
- Gold: All requirements fully implemented with complete traceability
- Platinum: Exceeds requirement expectations with forward-thinking design

#### 2. Code Quality & Standards
**Assessment Focus**:
- Coding standards compliance
- Code readability and maintainability
- Technical debt assessment

**Quality Indicators**:
- Bronze: Basic functionality implemented, obvious quality issues exist
- Silver: Follows basic standards, clear structure
- Gold: High-quality implementation, excellent maintainability
- Platinum: Exemplary code with educational value

#### 3. Security & Performance
**Assessment Focus**:
- Security vulnerability identification and remediation
- Performance bottleneck analysis
- Resource utilization efficiency assessment

**Quality Indicators**:
- Bronze: Basic security considerations, acceptable performance
- Silver: Comprehensive security protection, good performance
- Gold: Enterprise-level security standards, optimized performance design
- Platinum: Zero-trust security model, ultimate performance optimization

#### 4. Testing Coverage & Quality
**Assessment Focus**:
- Unit test coverage
- Integration test completeness
- Test quality and edge cases

**Quality Indicators**:
- Bronze: Basic tests covering main paths
- Silver: Complete test suite covering edge cases
- Gold: High-quality tests including regression and performance testing
- Platinum: Test-driven development with automated testing pipeline

#### 5. Architecture & Design Alignment
**Assessment Focus**:
- Architectural principles adherence
- Design pattern consistency
- Module coupling and cohesion

**Quality Indicators**:
- Bronze: Basic architecture complete, some design inconsistencies
- Silver: Good architectural design following design principles
- Gold: Excellent architectural implementation, high cohesion low coupling
- Platinum: Architectural exemplar with scalability and resilience

#### 6. Documentation & Maintainability
**Assessment Focus**:
- Code documentation quality
- API documentation completeness
- Maintenance overhead assessment

**Quality Indicators**:
- Bronze: Basic comments, simple documentation
- Silver: Complete code comments and usage instructions
- Gold: Comprehensive documentation system including design decision records
- Platinum: Exceptional documentation quality with automated generation and maintenance

#### 7. Risk Assessment & Deployment Readiness
**Assessment Focus**:
- Rollback strategy validation
- Deployment risk assessment
- Production readiness checklist

**Quality Indicators**:
- Bronze: Basic deployment plan, partial risk identification
- Silver: Complete deployment strategy with risk mitigation measures
- Gold: Comprehensive risk management with automated deployment process
- Platinum: Zero-downtime deployment with complete disaster recovery plan

### Review Decision Matrix
**Accept**: All dimensions reach Silver level or above
**Accept with Changes**: 1-2 dimensions below Silver with clear improvement plan
**Reject**: 3+ dimensions below Silver, or Bronze-level security/functional issues exist

### Review Output Format
```markdown
## 7-Dimensional Assessment Results

### Dimensional Scoring
- Functional Requirements Compliance: [Bronze|Silver|Gold|Platinum]
- Code Quality & Standards: [Bronze|Silver|Gold|Platinum]  
- Security & Performance: [Bronze|Silver|Gold|Platinum]
- Testing Coverage & Quality: [Bronze|Silver|Gold|Platinum]
- Architecture & Design Alignment: [Bronze|Silver|Gold|Platinum]
- Documentation & Maintainability: [Bronze|Silver|Gold|Platinum]
- Risk Assessment & Deployment Readiness: [Bronze|Silver|Gold|Platinum]

### Overall Quality Assessment
**Decision**: [Accept|Accept with Changes|Reject]
**Rationale**: [Detailed explanation based on 7-dimensional analysis]
```

# MCP tools introduction

## context7
- Use when you see <tools: context7>:
    <tools: context7>
    - xxx
    - yyy
    </tools: context7>
- Purpose: fetch external package or API references by resolving IDs and retrieving focused excerpts.
- Invocation flow:
```json
{"name":"functions.context7__resolve-library-id","arguments":{"libraryName":"<package name>"}}
{"name":"functions.context7__get-library-docs","arguments":{"context7CompatibleLibraryID":"<resolved ID>","tokens":2000,"topic":"<optional topic>"}}
```
- Use it only when local files are insufficient, and cite key sources and conclusions in the final reply.

## sequential-thinking
- Use when you see <tools: sequential-thinking>:
    <tools: sequential-thinking>
    - xxx
    - yyy
    </tools: sequential-thinking>
- Purpose: decompose complex tasks, record reasoning chains, and validate hypotheses.
- Invocation flow:
```json
{"name":"functions.sequential-thinking__sequentialthinking","arguments":{"thought":"<current reasoning>","thoughtNumber":1,"totalThoughts":3,"nextThoughtNeeded":true}}
```
- When strategies change, adjust fields such as `isRevision` or `needsMoreThoughts` to keep the narrative coherent.

## playwright
- Use when you see <tools: playwright>:
    <tools: playwright>
    - xxx
    - yyy
    </tools: playwright>
- Purpose: perform web interactions, gather online information, or simulate user flows.
- Invocation flow:
```json
{"name":"functions.playwright__browser_navigate","arguments":{"url":"https://example.com"}}
{"name":"functions.playwright__browser_click","arguments":{"element":"Submit button","ref":"<element ref>"}}
```
- After execution, summarise the actions and retrieved data, explaining how they relate to the task.

## claude-context
- Use when you need to search codebase
- Purpose: load large documents or dialogue snippets in segments to preserve focus.
- Invocation flow:
```json
{"name":"functions.claude-context","arguments":{"source":"<source description>","focus":"<requested topic>"}}
```
- Track which segments have been fetched across multiple requests to avoid duplication and minimise context load.
