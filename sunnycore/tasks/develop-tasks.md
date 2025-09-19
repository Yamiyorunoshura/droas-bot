<input>
  <context>
  1. {root}/docs/implementation-plan/{task_id}-plan.md - Implementation Plan (實作計劃)
  2. {root}/docs/architecture/*.md
  </context>
  <templates>
  1. {root}/sunnycore/templates/dev-notes-tmpl.yaml - Development Notes Template
  </templates>
</input>

<output>
1. {root}/docs/dev-notes/{task_id}-dev-notes.md - 完整的Development Notes文檔
2. high quality, tested code implementation
3. complete Test Coverage and Test Cases
4. refactored code that meets best practices
</output>

<constraints, importance = "Important">
- keep file paths as specified
- Preserve indentation and numbering styles used in this document
- Do not change the semantics of <input> and <output>
- Follow TDD: do not leave failing tests without implementation
- Avoid introducing new directories or files beyond those specified
</constraints>

<example>
markdown文件輸出方式：
	•	YAML 第一層 key 轉換為 Markdown 一級標題 (#)
	•	YAML 第二層 key 轉換為 Markdown 二級標題 (##)
	•	YAML 第三層 key 轉換為 Markdown 三級標題 (###)
	•	YAML value（字串或數字） 轉換為 Markdown 正文文字
</example>

<example>
initial todo list example:
- [ ] stage 0: Create a todo list
- [ ] stage 1: Understand the plan
- [ ] stage 2: Functional requirements
- [ ] stage 3: Non-functional requirements
- [ ] stage 4: Refactor
- [ ] stage 5: Dev notes
</example>

<example>
the updated todo list during functional requirements stage:
- [x] stage 0: Create a todo list
- [x] stage 1: Understand the plan
- [x] stage 2: Functional requirements
- [ ] xxx(e.g F-1: xxx)
- [ ] yyy
- [ ] zzz
- [ ] stage 3: Non-functional requirements
- [ ] stage 4: Refactor
- [ ] stage 5: Dev notes
</example>

<example>
the updated todo list during non-functional requirements stage:
- [x] stage 0: Create a todo list
- [x] stage 1: Understand the plan
- [x] stage 2: Functional requirements
- [x] stage 3: Non-functional requirements
- [ ] xxx(e.g N-1: xxx)
- [ ] yyy
- [ ] zzz
- [ ] stage 4: Refactor
- [ ] stage 5: Dev notes

<workflow, importance = "Important">
  <stage id="0: todo">
  <tools: todo-list>
  - Read all working steps
  - Take reference from the example and create a todo item 
  </tools: todo-list>
  </stage>
  
  <stage id="1: understand-plan">
  <tools: sequential-thinking>
  - Read the implementation plan
  - Think about the steps to implement the implementation plan based on TDD approach
  </tools: sequential-thinking>
  </stage>
  
  <stage id="2: functional-requirements">
  <tools: sequential-thinking>
  - Identify the functional requirements
  - Think about the test cases for each functional requirement and update the todo list
  - Write the test cases for each functional requirement
  - Write the code for each functional requirement
  </tools: sequential-thinking>

  <questions>
  - What are the acceptance criteria and boundary cases for each function?
  - Which external dependencies must be mocked or faked in tests?
  - How will error handling and retries be validated?
  </questions>
  </stage>
  
  <stage id="3: non-functional-requirements">
  <tools: sequential-thinking>
  - Identify the non-functional requirements
  - Think about the test cases for each non-functional requirement and update the todo list base on the test cases
  - Write the test cases for each non-functional requirement
  - Write the code for each non-functional requirement
  </tools: sequential-thinking>

  <questions>
  - What measurable budgets apply (latency, memory, throughput)?
  - How will observability (logs, metrics, traces) be validated?
  - Are security/compliance constraints testable and enforced?
  </questions>
  </stage>
  
  <stage id="4: refactor">
  <tools: sequential-thinking>
  - Refactor the code to meet best practices and actual requirements
  - Think about the test cases for the refactoring and update the todo list base on the test cases
  - Refactor the code of functional requirements
  - Refactor the code of non-functional requirements
  </tools: sequential-thinking>
  
  <questions>
  - Does the refactor preserve behavior and keep tests green?
  - Are interfaces clearer and modules decoupled without added complexity?
  </questions>
  </stage>
  
  <stage id="5: dev-notes">
  - Use the development notes template to write the development notes for the code
  - Output the markdown format development notes to {root}/docs/dev-notes/{task_id}-dev-notes.md
  - If there is already an existing development notes, update the development notes with the new information
  
  <checks>
  - [ ] All outputs in <output> are produced and consistent
  - [ ] All tests pass in CI with acceptable coverage thresholds
  - [ ] Code style and repository conventions are followed
  - [ ] Dev notes path and filename follow the specified pattern
  </checks>
  </stage>
</workflow>