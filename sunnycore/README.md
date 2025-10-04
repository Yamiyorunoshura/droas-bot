# 開發流程

## 🌱 Greenfield 專案流程

> 適用於從零開始的新專案開發

### 📋 階段一：需求分析

| 步驟 | 命令 | 說明 |
|------|------|------|
| 1 | `/sunnycore_pm *create-requirements` | 互動式創建需求文檔 |
| 2 | `/sunnycore_pm *create-architecture` | 互動式創建架構文檔 |
| 3 | `/sunnycore_pm *create-tasks` | 互動式創建任務文檔 |

### 💻 階段二：開發迭代

| 步驟 | 命令 | 說明 |
|------|------|------|
| 4 | `/sunnycore_pm *plan-tasks {task_id}` | 創建特定 task 的計劃文檔 |
| 5 | `/sunnycore_dev *develop-tasks {task_id}` | 創建特定 task 的開發文檔 |
| 6 | `/sunnycore_qa *review {task_id}` | 審查特定 task 的文檔 |

**流程控制：**
- ✅ **Review 通過**：檢查是否還有其他任務
  - 有任務 → 回到步驟 4
  - 無任務 → 進入階段三
- ❌ **Review 未通過**：使用 `/sunnycore_dev *brownfield-tasks {task_id}` 進行重開發，回到步驟 5

### 📊 階段三：總結與文檔化

| 步驟 | 命令 | 說明 |
|------|------|------|
| 7 | `/sunnycore_po *conclude` | 總結文檔 |
| 8 | `/sunnycore_po *curate-knowledge` | 整理知識文檔 |
| 9 | `/sunnycore_po *document-project` | 產出專案架構文件 |

### 🔄 流程圖

```mermaid
flowchart TD
    Start([開始 Greenfield 專案]) --> A1
    
    subgraph Phase1 ["📋 階段一：需求分析"]
        A1["1️⃣ /sunnycore_pm<br/>*create-requirements<br/>📝 建立需求"]
        A2["2️⃣ /sunnycore_pm<br/>*create-architecture<br/>🏗️ 建立架構"]
        A3["3️⃣ /sunnycore_pm<br/>*create-tasks<br/>📌 建立任務"]
        A1 --> A2 --> A3
    end
    
    subgraph Phase2 ["💻 階段二：開發迭代"]
        B1["4️⃣ /sunnycore_pm<br/>*plan-tasks<br/>🗓️ 規劃任務"]
        B2["5️⃣ /sunnycore_dev<br/>*develop-tasks<br/>⚙️ 開發實作"]
        B3["6️⃣ /sunnycore_qa<br/>*review<br/>🔍 文件審查"]
        B1 --> B2 --> B3
    end
    
    D1{"✅ Review<br/>通過?"}
    D2{"📝 還有<br/>任務?"}
    R1["🔄 /sunnycore_dev<br/>*brownfield-tasks<br/>重新開發"]
    
    subgraph Phase3 ["📊 階段三：總結"]
        C1["7️⃣ /sunnycore_po<br/>*conclude<br/>📋 總結文檔"]
        C2["8️⃣ /sunnycore_po<br/>*curate-knowledge<br/>📚 整理知識"]
        C3["9️⃣ /sunnycore_po<br/>*document-project<br/>📖 產出專案文件"]
        C1 --> C2 --> C3
    end
    
    Done([✨ 完成開發 Cycle])
    
    A3 --> B1
    B3 --> D1
    D1 -->|"❌ 否"| R1
    R1 --> B2
    D1 -->|"✅ 是"| D2
    D2 -->|"📝 是"| B1
    D2 -->|"✅ 否"| C1
    C3 --> Done
    
    style Start fill:#e1f5e1
    style Done fill:#e1f5e1
    style D1 fill:#fff4e6
    style D2 fill:#fff4e6
    style R1 fill:#ffe6e6
    style Phase1 fill:#f0f8ff
    style Phase2 fill:#f5f0ff
    style Phase3 fill:#fff0f5
```

---

## 🏗️ Brownfield 專案流程

> 適用於現有專案的擴展與維護

### 📋 階段一：需求分析

| 步驟 | 命令 | 說明 |
|------|------|------|
| 1 | `/sunnycore_po *document-project` | 確保專案架構文件是最新的 |
| 2 | `/sunnycore_pm *create-brownfield-requirements` | 互動式創建需求文檔 |
| 3 | `/sunnycore_pm *create-brownfield-architecture` | 互動式創建架構文檔 |
| 4 | `/sunnycore_pm *create-brownfield-tasks` | 互動式創建任務文檔 |

### 💻 階段二：開發迭代

| 步驟 | 命令 | 說明 |
|------|------|------|
| 5 | `/sunnycore_pm *plan-tasks {task_id}` | 創建特定 task 的計劃文檔 |
| 6 | `/sunnycore_dev *develop-tasks {task_id}` | 創建特定 task 的開發文檔 |
| 7 | `/sunnycore_qa *review {task_id}` | 審查特定 task 的文檔 |

**流程控制：**
- ✅ **Review 通過**：檢查是否還有其他任務
  - 有任務 → 回到步驟 5
  - 無任務 → 進入階段三
- ❌ **Review 未通過**：使用 `/sunnycore_dev *brownfield-tasks {task_id}` 進行重開發，回到步驟 6

### 📊 階段三：總結與文檔化

| 步驟 | 命令 | 說明 |
|------|------|------|
| 8 | `/sunnycore_po *conclude` | 總結文檔 |
| 9 | `/sunnycore_po *curate-knowledge` | 整理知識文檔 |
| 10 | `/sunnycore_po *document-project` | 更新專案架構文件 |

### 🔄 流程圖

```mermaid
flowchart TD
    Start([開始 Brownfield 專案]) --> A0
    
    subgraph Phase1 ["📋 階段一：需求分析"]
        A0["1️⃣ /sunnycore_po<br/>*document-project<br/>📖 更新專案文件"]
        A1["2️⃣ /sunnycore_pm<br/>*create-brownfield-requirements<br/>📝 建立需求"]
        A2["3️⃣ /sunnycore_pm<br/>*create-brownfield-architecture<br/>🏗️ 建立架構"]
        A3["4️⃣ /sunnycore_pm<br/>*create-brownfield-tasks<br/>📌 建立任務"]
        A0 --> A1 --> A2 --> A3
    end
    
    subgraph Phase2 ["💻 階段二：開發迭代"]
        B1["5️⃣ /sunnycore_pm<br/>*plan-tasks<br/>🗓️ 規劃任務"]
        B2["6️⃣ /sunnycore_dev<br/>*develop-tasks<br/>⚙️ 開發實作"]
        B3["7️⃣ /sunnycore_qa<br/>*review<br/>🔍 文件審查"]
        B1 --> B2 --> B3
    end
    
    D1{"✅ Review<br/>通過?"}
    D2{"📝 還有<br/>任務?"}
    R1["🔄 /sunnycore_dev<br/>*brownfield-tasks<br/>重新開發"]
    
    subgraph Phase3 ["📊 階段三：總結"]
        C1["8️⃣ /sunnycore_po<br/>*conclude<br/>📋 總結文檔"]
        C2["9️⃣ /sunnycore_po<br/>*curate-knowledge<br/>📚 整理知識"]
        C3["🔟 /sunnycore_po<br/>*document-project<br/>📖 更新專案文件"]
        C1 --> C2 --> C3
    end
    
    Done([✨ 完成開發 Cycle])
    
    A3 --> B1
    B3 --> D1
    D1 -->|"❌ 否"| R1
    R1 --> B2
    D1 -->|"✅ 是"| D2
    D2 -->|"📝 是"| B1
    D2 -->|"✅ 否"| C1
    C3 --> Done
    
    style Start fill:#e1f5e1
    style Done fill:#e1f5e1
    style D1 fill:#fff4e6
    style D2 fill:#fff4e6
    style R1 fill:#ffe6e6
    style Phase1 fill:#f0f8ff
    style Phase2 fill:#f5f0ff
    style Phase3 fill:#fff0f5
```

---

## 📝 流程說明

### 🌱 Greenfield vs 🏗️ Brownfield

| 特性 | Greenfield | Brownfield |
|------|-----------|------------|
| **適用場景** | 全新專案 | 現有專案擴展/維護 |
| **起始步驟** | 直接建立需求 | 先更新專案文件 |
| **需求命令** | `*create-requirements` | `*create-brownfield-requirements` |
| **架構命令** | `*create-architecture` | `*create-brownfield-architecture` |
| **任務命令** | `*create-tasks` | `*create-brownfield-tasks` |
| **總步驟數** | 9 步 | 10 步 |

### 🎯 關鍵決策點

1. **Review 檢查點**
   - 每個任務完成後都需要經過 QA 審查
   - 確保代碼質量和符合需求

2. **任務迭代**
   - Review 通過：繼續下一個任務或進入總結階段
   - Review 未通過：使用 brownfield-tasks 命令重新開發

3. **文檔更新**
   - Greenfield：結束時產出專案文件
   - Brownfield：開始前和結束時都要更新專案文件

### 💡 最佳實踐

- 📌 每個階段完成後進行檢查點確認
- 🔄 保持文檔與代碼同步
- ✅ 確保所有 Review 問題都已解決再進入下一階段
- 📚 及時整理和歸檔知識文檔
