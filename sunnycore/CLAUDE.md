# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## [Constraints]
1. Only execute commands explicitly defined in [Custom-Commands]; no unlisted operations are allowed.
2. When executing commands, fully follow the steps and checkpoints in the corresponding task files, without skipping or simplifying processes.
3. When user commands are unclear or do not match the defined formats, request clarification rather than making assumptions.
4. Read all files explicitly defined in the task [Input].
5. Follow the tool usage methods defined in [Tools] to assist task execution.
6. Comply with all constraints defined in [Constraints].
  6.1 If there are conflicts between [Constraints], handle them according to the following priority:
    - Highest priority: CLAUDE.md[Constraints]
    - Second highest priority: commands[Constraints]
    - Third highest priority: tasks[Constraints]
7. Complete all to-do items in the todo list.
8. Verify the completion of all [DoD].
9. Read and complete tasks based on the relevant guidelines stated in [xxx-Guidelines].
10. Wrap-Up Gatekeeper — do not wrap up before completion
  10.1 No “completion” claims with incomplete ToDos: as long as the todo list contains any non-completed status (including pending, in_progress, blocked), it is forbidden to output closure phrases such as “completed,” “wrap up,” or “write summary,” and forbidden to enter the “generate development notes/journal” flow.
  10.2 Only three legal end states:
    - completed: all ToDos are completed and all [DoD] have been verified.
    - blocked: there are external dependencies or permission/resource issues. Each item must be marked blocked with a specific blocking reason and an unblocking condition.
    - external-interrupt: interruption caused by platform/tool limits (such as time/quota/timeout). Do not claim completion; you must execute the “state persistence and resume plan” workflow (see 10.6).

## [Tool-Guidelines]
This section describes tool usage scenarios.

1. **todo_write — Task Tracking Tool**
  Purpose:
  - Create to-do lists to track task execution progress
  - Track completion status of multi-stage workflows

2. **sequentialthinking (MCP) — Structured Reasoning Tool**
  Purpose:
  - Conduct structured reasoning and verification
  - Analyze complex problems and design decisions
  - Assess task complexity and dependencies
  
  Reasoning step recommendations:
  - Simple tasks: 1–3 totalThoughts
  - Medium tasks: 3–5 totalThoughts
  - Complex tasks: 5–8 totalThoughts

3. **claude-context (MCP) — Codebase Semantic Search and Indexing Tool**
  Purpose:
  - Build a codebase index to support semantic search
  - Find relevant code snippets and implementation details
  - Process large files or file collections in segments
  
  Notes:
  - Always use search_code directly for codebase search
  - Only use index_codebase when the codebase has not yet been indexed
  - Check indexing status first before attempting to index

4. **context7 (MCP) — External Reference Tool**
  Purpose:
  - Obtain external package and architecture pattern references
  - Find domain-specific best practices
  - Retrieve the latest API documentation

5. **playwright_browser (MCP) — Web Browsing Tool**
  Purpose:
  - Conduct web research to obtain requirement examples
  - Reference industry standards or example requirements
  
  Notes:
  - Avoid including sensitive or proprietary information

## [Template-Usage-Guidelines]

### Template Philosophy

All templates are minimal structural frameworks using placeholders (`""`, `[]`, `{}`). When using templates:
1. Read the template to understand the expected structure
2. Fill in only the fields relevant to your project
3. Add additional fields as needed for your specific context
4. After filling, convert YAML to Markdown using the rules below

### YAML to Markdown Conversion Rules

1. **Heading Level Conversion**
   - 1st level key → h1 (`#`)
   - 2nd level key → h2 (`##`)
   - 3rd level key → h3 (`###`)
   - 4th level and deeper → h4 (`####`)
   - Avoid using h5 and h6 to maintain clear hierarchy

2. **Value Type Conversion**
   - **Simple string values**: Render as paragraph text below the heading
   - **Nested object values**: Each key-value pair as `**Key**: Value` format
   - **Multi-line text** (using `|` or `>`): Convert to paragraph text, preserving line breaks
   - **Numbers and booleans**: Display as plain text

3. **List Conversion**
   - **Simple lists**: Convert to Markdown unordered list using `-`
   - **Empty lists**: Omit the section (assume no content to display)

4. **Object Array Conversion**
   - Each object in the array becomes a subsection
   - First property (typically `id`, `name`, or `title`) becomes h4 heading
   - Remaining properties formatted as `**Property**: Value` below the heading
   - If objects have no natural heading property, use numeric subsections

5. **Empty Value Handling** (Critical for minimal templates)
   - **Empty strings** (`""`): Omit the section entirely
   - **Empty lists** (`[]`): Omit the section entirely
   - **Empty objects** (`{}`): Omit the section entirely
   - **Only display sections with actual content**

6. **Special Markers**
   - **YAML comments** (`# comment`): Remove during conversion (used only as template guidance)
   - **Inline code/references**: Preserve in backticks when appropriate

7. **Formatting Best Practices**
   - Use bold (`**text**`) for property labels and important terms
   - Use code formatting (`` `text` ``) for technical terms, file paths, IDs, and identifiers
   - Add blank lines between sections for readability
   - Preserve the hierarchical structure from YAML in the Markdown output

## [Summary-Instructions]
When performing auto-compact (automatic conversation compression), follow these rules:

1. Content that MUST be preserved:
  - [Constraints] All constraint conditions
  - [Tools] Tool definitions
  - [Output] Output format requirements
  - [DoD] Definition of Done
  - All [xxx-Guidelines] related guidelines
  - [Summary-Instructions] Summary instructions
  - [Path-Variables] Path variables

2. Content that MAY be compressed:
  - Task status information (progress, results, etc.)
  - May be summarized while retaining key information

3. Content that MAY be discarded:
  - Test results of completed tasks
  - Output content of completed tasks (for example, already written documents, code, etc.)