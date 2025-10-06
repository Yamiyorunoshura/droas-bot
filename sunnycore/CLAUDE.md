[General-Guidelines]
1. Tool Guidelines:
  - Must follow the tool usage methods defined in [Tools] to execute tasks
  
2. Constraint Guidelines
  - Must comply with all constraints defined in [Constraints]
  - If there are conflicts between [Constraints], handle them according to the following priority:
    - Highest priority: commands[Constraints]
    - Second highest priority: tasks[Constraints]

3. Task Guidelines
  - Must complete all to-do items
  - Must complete all [DoD]

[Tool-Guidelines]
This section describes tool usage scenarios

1. **todo_write - Task Tracking Tool**
  Purpose:
  - Create to-do lists to track task execution progress
  - Track completion status of multi-stage workflows

2. **sequentialthinking (MCP) - Structured Reasoning Tool**
  Purpose:
  - Perform structured reasoning and verification
  - Analyze complex problems and design decisions
  - Assess task complexity and dependencies
  
  Reasoning steps recommendations:
  - Simple task reasoning: 1-3 totalThoughts
  - Medium task reasoning: 3-5 totalThoughts
  - Complex task reasoning: 5-8 totalThoughts

3. **claude-context (MCP) - Codebase Semantic Search and Indexing Tool**
  Purpose:
  - Build codebase index to support semantic search
  - Find relevant code snippets and implementation details
  - Process large files or file collections in segments

4. **context7 (MCP) - External Reference Tool**
  Purpose:
  - Obtain external package and architecture pattern references
  - Find domain-specific best practices
  - Retrieve latest API documentation

5. **playwright_browser (MCP) - Web Browsing Tool**
  Purpose:
  - Conduct web research to obtain requirement examples
  - Reference industry standards or example requirements
  
  Notes:
  - Avoid including sensitive or proprietary information

[Template-Usage-Guidelines]
1. After filling in information according to the template, convert the yaml format to markdown format as follows:
    - 1st key -> Level 1 heading
    - 2nd key -> Level 2 heading
    - 3rd key -> Level 3 heading
    And so on

[Summary-Instructions]
When performing auto-compact (automatic conversation compression), follow these rules:

1. Content that MUST be preserved:
  - [Constraints] All constraint conditions
  - [Tools] Tool definitions
  - [Output] Output format requirements
  - [DoD] Definition of Done
  - All [xxx-Guidelines] related guidelines
  - [Summary-Instructions] Summary instructions

2. Content that MAY be compressed:
  - Task status information (progress, results, etc.)
  - Can be summarized while retaining key information

3. Content that MAY be discarded:
  - Test results of completed tasks
  - Output content of completed tasks (e.g., already written documents, code, etc.)