<input>
  <context>
    協助用戶通過結構化的互動流程，將初始想法和需求轉化為完整的功能性和非功能性需求文檔
  </context>
  <templates>
    {root}/sunnycore/templates/requirement-tmpl.yaml
  </templates>
</input>

<output>
  {root}/docs/requirements/*.md - 完整的需求規格文檔，包含功能性需求、非功能性需求和驗收標準
</output>

<constraints importance="Critical">
- 必須使用Todo-list Tool追蹤和管理任務進度
- 必須使用Sequential-thinking Tool進行深度需求分析
- 採用互動式需求澄清流程，確保每個需求都經過充分討論
- 所有輸出必須嚴格符合requirement template格式
- 使用適當的引導性問題協助用戶澄清模糊需求
- 完成所有任務後必須更新todo list狀態
</constraints>

<workflow importance="Critical">
  <stage id="0: Setup and Planning" level_of_think="think">
    - 閱讀整份workflow
    - 進一步閱讀所有步驟和無序列表項
    - 使用todo list tool為每個無序列表項創建一個todo item
    
    <questions>
    - 工作流程的完整範圍是什麼？
    - 需要創建哪些具體的todo items？
    </questions>
    
    <checks>
    - [ ] 所有workflow步驟都已識別
    - [ ] todo list已創建並包含所有必要項目
    </checks>
  </stage>

  <stage id="1: User Requirements Analysis" level_of_think="think hard">
    - 深度思考、分析用戶的輸入
    - 使用Sequential-thinking Tool協助分析用戶需求
    - 將用戶需求分類為Functional Requirements與Non-functional Requirements
    
    <questions>
    - 用戶的核心需求是什麼？
    - 如何準確區分功能性和非功能性需求？
    - 是否遺漏任何關鍵需求？
    </questions>
    
    <checks>
    - [ ] 需求分類是否正確且完整
    - [ ] 初始需求分析是否符合邏輯
    - [ ] 是否需要進一步澄清的模糊點已識別
    </checks>
  </stage>

  <stage id="2: Functional Requirements Clarification" level_of_think="think harder">
    - 與用戶討論每個Functional Requirement
    - 使用Sequential-thinking Tool協助進行逐步的需求澄清
    - 根據用戶反饋適時更新需求
    - 將澄清後的Functional Requirements填入requirement template
    - 適當使用引導性問題幫助用戶澄清需求
    
    <questions>
    - 每個功能需求是否足夠具體和可測量？
    - 驗收標準是否明確定義？
    - 功能需求之間的依賴關係是什麼？
    </questions>
    
    <checks>
    - [ ] 所有Functional Requirements都已充分澄清
    - [ ] 每個需求都有對應的驗收標準
    - 需求格式符合template規範
    </checks>
  </stage>

  <stage id="3: Non-functional Requirements Clarification" level_of_think="think harder">
    - 與用戶討論每個Non-functional Requirement
    - 使用Sequential-thinking Tool協助進行逐步的需求澄清
    - 根據用戶反饋適時更新需求
    - 將澄清後的Non-functional Requirements填入requirement template
    - 適當使用引導性問題幫助用戶澄清需求
    
    <questions>
    - 性能、安全性、可用性等非功能性要求是否明確？
    - 技術限制和業務限制是否都已考慮？
    - 非功能性需求是否可量化和測試？
    </questions>
    
    <checks>
    - [ ] 所有Non-functional Requirements都已充分澄清
    - [ ] 每個非功能性需求都有明確的衡量標準
    - [ ] 需求格式符合template規範
    </checks>
  </stage>

  <stage id="4: Final Validation and Output" level_of_think="think" cache_read_budget="high" write_token_budget="high">
    - 詢問用戶是否滿意完整的Requirements
    - 使用sequential thinking思考如何將完整的Requirements轉換為markdown格式
    - 將轉換後的requirements文件輸出到{root}/docs/requirements.md
    - 運行{root}/sunnycore/scripts/shard-requirements.sh將requirements.md分割為多個文件
    
    <questions>
    - 最終文檔是否涵蓋了所有用戶需求？
    - template中所有需要填入的項目都已填入？
    - 是否存在需要進一步澄清的遺漏點？
    </questions>
    
    <checks>
    - [ ] 用戶確認滿意最終Requirements
    - [ ] template中所有需要填入的項目都已填入
    - [ ] 所有功能性和非功能性需求都已包含
    - [ ] 驗收標準完整且可測試
    </checks>
  </stage>
</workflow>

<example>
用戶輸入："我想建立一個線上書店系統"

Requirements Analyst回應：
"我將協助您將這個想法轉化為詳細的需求規格。讓我使用Sequential-thinking Tool深入分析...

基於初步分析，我識別出以下需求類別：
Functional Requirements:
- 用戶註冊和登入功能
- 書籍搜索和瀏覽功能  
- 購物車和結帳流程
- 訂單管理系統

Non-functional Requirements:
- 系統性能要求
- 安全性要求
- 可用性標準

現在讓我們逐一澄清每個功能需求。首先關於用戶註冊功能，您希望支援哪些註冊方式？是否需要社交媒體登入選項？"
</example>

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