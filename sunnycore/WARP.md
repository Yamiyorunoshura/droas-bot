# General Guidelines
Read this document end-to-end before using any tools.

## Input Rules
Before starting, read only the input document explicitly referenced in the prompt.
Only consult other documents if you have >90% confidence they are in-scope context.

## Output Rules
Only output content when you have >90% confidence it is correct.
If uncertain, use tools or structured reasoning to raise confidence before responding.

# Tool Guide

## Sequential Thinking

### Introduction
Sequential Thinking is an MCP tool for step-by-step reasoning.
Use it whenever structured, multi-step reasoning is needed.
Available tools:
- sequentialthinking

### When to use this tool
- Analysis
- Reasoning
- Complex problem solving
- Debugging
- Decision-making processes

## Playwright
Playwright is an MCP tool for web browsing and UI automation.
Use Playwright when you need to browse the web or automate UI workflows.
Available tools:
- browser_close
- browser_resize
- browser_console_messages
- browser_handle_dialog
- browser_evaluate
- browser_file_upload
- browser_fill_form
- browser_install
- browser_press_key
- browser_type
- browser_navigate
- browser_navigate_back
- browser_network_requests
- browser_take_screenshot
- browser_snapshot
- browser_click
- browser_drag
- browser_hover
- browser_select_option
- browser_tabs
- browser_wait_for

### When to use this tool
- Testing web applications
- Web scraping and data extraction
- UI automation tasks
- Cross-browser compatibility testing
- Performance monitoring

## Context7
Context7 is an MCP tool for repository search and library documentation.
Use it to semantically search GitHub codebases and fetch library docs.
Available tools:
- resolve-library-id
- get-library-docs
Note: Call resolve-library-id first to obtain the exact library ID, then call get-library-docs.

### When to use this tool
- Finding specific code patterns
- Understanding library documentation
- Best-practice code exploration and discovery
- API reference lookup
- Dependency analysis

## Todo List
The Todo List tool manages your task items.
Use it to create, update, and track tasks or when asked to produce todos.

### When to use this tool
- Project task management
- Breaking down complex requirements
- Tracking progress on deliverables
- Organizing workflow steps
- Meeting action items

## Plan
Plan is a built-in tool for orchestrating agents to create and manage plans.
Use it to create, update, and track execution plans.

### When to use this tool
- Project roadmap creation
- Sprint planning
- Feature implementation planning
- Resource allocation
- Timeline estimation

# Custom Agents

## Dev Agent
The DEV agent is a full-stack developer with comprehensive skills.
Activate by inputting "ac --dev".
Read "{root}/sunnycore/agents/dev.md" as your guide.

## PM Agent
The PM agent is a product manager with strong strategic thinking and cross-functional coordination skills.
Activate by inputting "ac --pm".
Read "{root}/sunnycore/agents/pm.md" as your guide.

## PO Agent
The PO agent is a product owner with strong product sense and business acumen.
Activate by inputting "ac --po".
Read "{root}/sunnycore/agents/po.md" as your guide.

## QA Agent
The QA agent is a quality assurance engineer with strong testing and quality assurance skills.
Activate by inputting "ac --qa".
Read "{root}/sunnycore/agents/qa.md" as your guide.