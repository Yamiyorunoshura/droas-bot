<input>
  <context>
    1. {root}/docs/requirements
    2. {root}/docs/architecture/*.md
    3. {root}/sunnycore/scripts/shard-architecture.sh - Architecture Script
  </context>
  <templates>
    {root}/sunnycore/templates/architecture-tmpl.yaml
  </templates>
</input>

<output>
1. {root}/docs/architecture/*.md
</output>

<constraints importance="Critical">
- 在完成每一個步驟後，必須更新Todo-list狀態
- 必須使用Sequential-thinking Tool進行深度思考分析
- 必須獲得User確認才能進行關鍵決策
- 必須嚴格按照Architecture Template格式進行設計
</constraints>

<workflow importance="Critical">
  <stage id="0: Create Todo List" level_of_think="think" cache_read_budget="low">
    - 閱讀整份Workflow Specifications
    - 進一步閱讀所有Stage Steps
    - 閱讀所有步驟下的無序列表項
    - 使用todo list tool為每一個無序列表項創建todo items

    <questions>
    - 是否完整理解了整個Brownfield Architecture創建流程？
    - 每個階段的核心目標是什麼？
    - 需要哪些輸入資源和工具支援？
    </questions>

    <checks>
    - [ ] 是否成功閱讀了完整的Workflow？
    - [ ] 是否識別了所有Stage Steps？
    - [ ] 是否為每個無序列表項創建了todo item？
    - [ ] Sequential-thinking Tool是否正常運作？
    - [ ] todo list tool是否正常運作？
    </checks>
  </stage>

  <stage id="1: Read Existing Architecture Files" level_of_think="think hard" cache_read_budget="medium">
    - 閱讀{root}/docs/architecture/technical-stack.md
    - 閱讀{root}/docs/architecture/system-architecture.md  
    - 使用Sequential-thinking Tool深度思考分析現有Architecture

    <questions>
    - 現有的Technical Stack是否滿足新的需求？
    - System Architecture中是否存在瓶頸或限制？
    - 現有Architecture的優勢和劣勢是什麼？
    - 哪些Architecture決策需要重新評估？
    - 現有架構的可擴展性如何？
    </questions>

    <checks>
    - [ ] 是否成功讀取了technical-stack.md？
    - [ ] 是否成功讀取了system-architecture.md？
    - [ ] 是否使用Sequential-thinking Tool完成了深度分析？
    - [ ] 是否理解了現有Architecture的核心設計原則？
    - [ ] 是否識別了Architecture改進的機會點？
    </checks>
  </stage>

  <stage id="2: Create Functional Requirements Architecture" level_of_think="think harder" cache_read_budget="high">
    - 閱讀{root}/docs/requirements/functional-requirements/*.md
    - 使用Sequential-thinking Tool深度思考分析現有Architecture
    - 使用Context7搜索是否有可滿足相關Functional Requirements的Architecture Design
    - 為每個Functional Requirement創建一個Architecture Design
    - 在完成每一個Functional Requirement的Architecture Design後，必須詢問User是否Satisfied
    - 若User Satisfied、則將Architecture Design填入Architecture Template
    - 若User不Satisfied，則重複上述步驟，直到User Satisfied為止

    <questions>
    - 每個Functional Requirement的核心挑戰是什麼？
    - 現有Architecture如何支持這些需求？
    - 需要哪些新的Architecture組件？
    - 如何確保Architecture Design的可擴展性和可維護性？
    - 這個Architecture Design是否符合industry best practices？
    </questions>

    <checks>
    - [ ] 是否完整閱讀了requirements/functional-requirements/*.md？
    - [ ] 是否使用Context7搜索了相關的Architecture Design？
    - [ ] 是否為每個Functional Requirement創建了對應的Architecture Design？
    - [ ] 是否獲得了User的Satisfied確認？
    - [ ] 是否正確填入了Architecture Template？
    - [ ] Architecture Design是否考慮了performance和scalability？
    </checks>
  </stage>

  <stage id="3: Create Non-functional Requirements Architecture" level_of_think="think harder" cache_read_budget="high">
    - 閱讀{root}/docs/requirements/non-functional-requirements/*.md 
    - 使用Sequential-thinking Tool深度思考分析現有Architecture
    - 使用Context7搜索是否有可滿足相關Non-functional Requirements的Architecture Design
    - 為每個Non-functional Requirement創建一個Architecture Design
    - 在完成每一個Non-functional Requirement的Architecture Design後，必須詢問User是否Satisfied
    - 若User Satisfied、則將Architecture Design填入Architecture Template
    - 若User不Satisfied，則重複上述步驟，直到User Satisfied為止

    <questions>
    - 每個Non-functional Requirement對系統性能的影響是什麼？
    - 如何平衡不同Non-functional Requirements之間的衝突？
    - Security和Performance requirements如何協調？
    - Scalability設計是否能應對未來增長？
    - 監控和維護策略是否充分？
    </questions>

    <checks>
    - [ ] 是否完整閱讀了requirements/non-functional-requirements/*.md？
    - [ ] 是否深入分析了所有Non-functional Requirements？
    - [ ] 是否考慮了Security Architecture Design？
    - [ ] 是否設計了Performance Architecture？
    - [ ] 是否規劃了Availability和Disaster Recovery？
    - [ ] 是否獲得了User對每個設計的Satisfied確認？
    - [ ] Architecture Design是否包含了監控和維護策略？
    </checks>
  </stage>

  <stage id="4: Update Architecture Files" level_of_think="think" cache_read_budget="medium">
    - 使用Sequential thinking深度思考是否需要更新Technical Stack
    - 使用Sequential thinking深度思考是否需要更新System Architecture
    - 若需要更新，則更新Architecture Files
    - 若不需要更新，則跳過此步驟

    <questions>
    - Technical Stack更新會帶來什麼影響？
    - System Architecture變更是否會影響現有功能？
    - 更新策略是否考慮了風險最小化？
    - 是否需要階段性的遷移計劃？
    </questions>

    <checks>
    - [ ] 是否評估了Technical Stack更新的必要性？
    - [ ] 是否分析了System Architecture變更的影響？
    - [ ] 是否制定了詳細的更新計劃？
    - [ ] 是否考慮了風險mitigation策略？
    - [ ] 更新後的Architecture是否保持一致性？
    </checks>
  </stage>

  <stage id="5: Review Final Architecture Files" level_of_think="think" cache_read_budget="low">
    - 使用sequential thinking思考如何將完整的Architecture Files轉換為markdown格式
    - 詢問User是否Satisfied完整的Architecture Files
    - 若User Satisfied、則輸出最終Architecture Files
    - 若User不Satisfied，則重複上述步驟，直到User Satisfied為止
    - 運行{root}/sunnycore/scripts/shard-architecture.sh將architecture.md分割為多個文件

    <questions>
    - Architecture Files是否完整地涵蓋了所有Requirements？
    - 文檔的結構是否清晰易懂？
    - 是否包含了必要的Implementation Roadmap？
    - Risk Assessment是否充分？
    - 文檔是否符合團隊的標準格式？
    </questions>

    <checks>
    - [ ] 是否獲得了User對完整Architecture Files的Satisfied確認？
    - [ ] 是否正確轉換為markdown格式？
    - [ ] template中所有需要填入的項目都已填入？
    - [ ] 是否包含了完整的Technical Stack定義？
    - [ ] 是否包含了詳細的System Architecture圖表？
    - [ ] 是否包含了Implementation Roadmap？
    - [ ] 是否成功輸出到指定路徑？
    - [ ] 是否成功運行shard-architecture.sh？
    </checks>
  </stage>
</workflow>

<example>
```yaml
xxx:
  111:
    aaa:
    bb:
    cc:
```
- example output
```markdown

# xxx

## 111

### aaa

### bb

### cc
```
<example>