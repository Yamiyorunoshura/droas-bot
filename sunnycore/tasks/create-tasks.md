<input>
  <context>
    1. {root}/docs/requirements/*.md - Project Functional Requirements and Non-functional Requirements
    2. {root}/docs/architecture/*.md - Architecture Design and Technical Specifications  
    3. {root}/sunnycore/templates/tasks-tmpl.yaml - Task Template Format
  </context>
  <templates>
    - tasks-tmpl.yaml - Standard task structure template
  </templates>
</input>

<output>
  1. {root}/docs/tasks.md - Comprehensive task breakdown with atomized sub-tasks
</output>

<constraints importance="Critical">
- 必須在完成每一個階段後更新todo list狀態
- Task必須包含實現功能所需的原子化任務 (Atomized Tasks)
- 嚴格遵循tasks-tmpl.yaml模板格式
- 所有技術術語必須使用英文標準化表達
- 確保Functional Requirements和Non-functional Requirements完整覆蓋
</constraints>

<workflow importance="Critical">
  <stage id="1: Context Analysis and Planning" level_of_think="think hard">
    - 閱讀整份Workflow
    - 進一步閱讀所有Steps
    - 閱讀所有Steps下的無序列表項
    - 使用todo list tool為每一個無序列表項創建一個todo item
    - 建立Dependencies Analysis

    <questions>
    - 我是否完全理解了所有Functional Requirements和Non-functional Requirements？
    - 是否存在未識別的Dependencies？
    - Task原子化程度是否適當？
    - 資源分配是否平衡合理？
    </questions>

    <checks>
    - [ ] 確認所有Input Files已被讀取和理解
    - [ ] 驗證計劃包含所有必要的Work Items
    - [ ] 檢查Dependencies Identification的完整性
    </checks>
  </stage>

  <stage id="2: Functional Requirements Task Creation" level_of_think="think hard">
    - 閱讀Requirements.md
    - 使用Sequential-thinking Tool深度分析Requirements.md
    - 識別第一個Functional Requirement
    - 閱讀Design.md
    - 識別第一個Functional Requirement對應的Architecture Design
    - 為第一個Functional Requirement創建一個Task
    - Task應該包含實現功能所需的Atomized Tasks
    - 根據Template Files填入Tasks並輸出
    - 重複上述步驟，直到所有Functional Requirements都被識別且Tasks被創建完成

    <questions>
    - 每個Functional Requirement是否都有對應的具體Implementation Plan？
    - Task分解是否足夠原子化且可獨立執行？
    - Architecture Design是否與Functional Requirements完全對應？
    - 是否遺漏了任何重要的功能特性？
    </questions>

    <checks>
    - [ ] Template Compliance檢查：輸出是否嚴格遵循tasks-tmpl.yaml格式
    - [ ] Requirements Coverage檢查：所有Functional Requirements是否都有對應的Tasks
    - [ ] Task Completeness檢查：每個Task是否包含必要的Sub-tasks
    - [ ] Technical Terminology Consistency檢查：是否使用了統一的英文技術術語
    </checks>
  </stage>

  <stage id="3: Non-Functional Requirements Task Creation" level_of_think="think hard">
    - 閱讀Requirements.md
    - 使用Sequential-thinking Tool深度分析Requirements.md
    - 識別第一個Non-functional Requirement
    - 閱讀Design.md  
    - 識別第一個Non-functional Requirement對應的Architecture Design
    - 為第一個Non-functional Requirement創建一個Task
    - Task應該包含實現功能所需的Atomized Tasks
    - 根據Template Files填入Tasks並輸出
    - 重複上述步驟，直到所有Non-functional Requirements都被識別且Tasks被創建完成

    <questions>
    - Performance Optimization需求是否已充分考慮？
    - Security Requirements是否得到適當的Implementation Plan？
    - Scalability和Maintainability考量是否完整？
    - Infrastructure Requirements是否與系統設計匹配？
    </questions>

    <checks>
    - [ ] Dependencies Validity檢查：Tasks間的Dependencies是否邏輯正確
    - [ ] Task Executability檢查：每個Task是否具體、可操作、可測量
    - [ ] Non-functional Requirements Coverage檢查
    - [ ] 與Functional Requirements的Integration檢查
    </checks>
  </stage>

  <stage id="4: Quality Assurance and Validation" level_of_think="think">
    - 閱讀Tasks.md
    - 使用Sequential-thinking Tool深度分析Tasks.md
    - 校對template中所有需要填入的項目都已填入
    - 若不符合，則重複上述步驟，直到Tasks.md符合Template Files格式
    - 確認所有Functional Requirements和Non-functional Requirements都被識別且Tasks被創建完成

    <questions>
    - 所有Tasks是否符合Template Compliance要求？
    - Task執行順序是否符合Development Logic？
    - 是否存在遺漏或重複的Tasks？
    - 最終輸出是否達到Quality Standards？
    </questions>

    <checks>
    - [ ] Priority Rationality檢查：Task順序是否符合Development Logic
    - [ ] Resource Allocation檢查：Task複雜度分配是否平衡合理
    - [ ] template中所有需要填入的項目都已填入
    - [ ] Completeness和Consistency最終確認
    </checks>
  </stage>
</workflow>

<example>
```yaml
xxx:
  xxx:
    111:
    222:
    333:
  yyy:
  zzz:
```
```markdown

# xxx:
  - [ ] xxx
    - [ ] 111
    - [ ] 222
    - [ ] 333
  - [ ] yyy
  - [ ] zzz
```
</example>
