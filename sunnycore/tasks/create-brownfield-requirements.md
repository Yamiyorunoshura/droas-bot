<input>
  <context>
  1. User Requirements (用戶需求)
  2. {root}/docs/architecture/*.md
  3. {root}/sunnycore/templates/requirements-tmpl.yaml
  4. {root}/sunnycore/scripts/shard-requirements.sh - Requirements Script
  </context>
  <templates>
  {root}/sunnycore/templates/requirements-tmpl.yaml
  </templates>
</input>

<output>
1. {root}/docs/requirements/*.md
</output>

<workflow importance="Critical">
  <stage id="0: Create Todo List", level_of_think="think">
  - 閱讀整份Workflow
  - 進一步閱讀所有Stage
  - 閱讀所有Stage下的無序列表項
  - 使用todo list tool為每個無序列表項創建一個todo item

  <questions>
  - 是否已完整閱讀所有Workflow步驟？
  - 每個無序列表項是否都已創建對應的todo item？
  </questions>

  <checks>
  - [ ] 確認Sequential-thinking Tool已正確使用
  - [ ] 驗證todo list已創建並包含所有必要項目
  </checks>
  </stage>

  <stage id="1: Analyze User Requirements", level_of_think="think hard">
  - 閱讀User Requirements (用戶需求)
  - 使用Sequential-thinking Tool深度分析User Requirements
  - 使用Context7搜索是否有可滿足相關需求的Requirements
  - 為每個需求創建一個Requirement
  - 在完成每一個Requirement後，必須詢問User是否滿意
  - 若User滿意、則將Requirement填入Requirement Template
  - 若User不滿意，則重複上述步驟，直到User滿意為止

  <questions>
  - User Requirements是否已完整理解？
  - 每個需求是否都有對應的Requirement？
  - User對每個Requirement是否滿意？
  </questions>

  <checks>
  - [ ] 確認Sequential-thinking Tool已正確使用
  - [ ] 驗證Context7搜索結果的相關性
  - [ ] 確保User滿意度確認已完成
  </checks>
  </stage>

  <stage id="2: Design Functional Requirements on Existing Architecture", level_of_think="think harder">
  - 閱讀{root}/docs/architecture/*.md
  - 使用Sequential-thinking Tool深度思考分析現有Architecture
  - 使用Context7搜索是否有可滿足相關需求的Requirements
  - 構思(Conceptualization)如何在盡可能不破壞現有Architecture之上滿足Functional Requirements (功能性需求)
  - 詢問用戶是否滿意
  - 若User滿意、則將Requirement填入Requirement Template
  - 若User不滿意，則重複上述步驟，直到User滿意為止

  <questions>
  - 現有Architecture是否已充分理解？
  - Functional Requirements與現有Architecture的兼容性如何？
  - 設計方案是否最小化對現有Architecture的影響？
  </questions>

  <checks>
  - [ ] 確認Architecture文檔已完整閱讀
  - [ ] 驗證Functional Requirements設計的可行性
  - [ ] 確保User對設計方案滿意
  </checks>
  </stage>

  <stage id="3: Design Non-functional Requirements on Existing Architecture", level_of_think="think harder">
  - 閱讀{root}/docs/architecture/*.md
  - 使用Sequential-thinking Tool深度思考分析現有Architecture
  - 使用Context7搜索是否有可滿足相關需求的Requirements
  - 構思(Conceptualization)如何在盡可能不破壞現有Architecture之上滿足Non-functional Requirements (非功能性需求)
  - 詢問用戶是否滿意

  <questions>
  - Non-functional Requirements與現有Architecture的兼容性如何？
  - Performance、Security、Scalability等要求是否已充分考慮？
  - 設計是否平衡了需求滿足與Architecture穩定性？
  </questions>

  <checks>
  - [ ] 確認Non-functional Requirements設計的完整性
  - [ ] 驗證與Functional Requirements的協調性
  - [ ] 確保User對設計方案滿意
  </checks>
  </stage>

  <stage id="4: Output Final Requirements Document", level_of_think="think">
  - 使用sequential thinking思考如何將完整的Requirements轉換為markdown格式
  - 將轉換後的requirements文件輸出到{root}/docs/requirements.md
  - 詢問用戶是否滿意
  - 若User滿意、則輸出最終Requirements Document
  - 若User不滿意，則重複上述步驟，直到User滿意為止
  - 運行{root}/sunnycore/scripts/shard-requirements.sh將requirements.md分割為多個文件

  <questions>
  - 最終Requirements Document是否完整包含所有需求？
  - template中所有需要填入的項目都已填入？
  - User是否對最終輸出滿意？
  </questions>

  <checks>
  - [ ] 確認Requirements Document格式正確
  - [ ] 驗證所有需求都已包含在最終Document中
  - [ ] 確保User最終滿意度確認
  - [ ] 確認shard-requirements.sh已正確運行
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