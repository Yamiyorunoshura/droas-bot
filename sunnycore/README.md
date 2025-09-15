# 開發流程

## Greenfield

### 需求分析階段
1. ac --pm *create-requirements互動式創建需求文檔
2. ac --pm *create-architecture互動式創建架構文檔
3. ac --pm *create-tasks互動式創建任務文檔

### 開發階段
4. ac --pm *plan-tasks {task_id} 創建特定task的計劃文檔
5. ac --dev *develop-tasks {task_id} 創建特定task的開發文檔
6. ac --qa *review {task_id} 審查特定task的文檔

如果review通過的話，進入第7步，否則回到使用 ac --dev *brownfield-tasks {task_id} 進行重開發
如果還有task的話，回到第4步，否則進入第7步

### 總結階段
7. ac --pm *conclude 總結文檔
8. ac --pm *curate-knowledge 整理知識文檔

#### Mermaid 流程圖
```mermaid
flowchart TD
  subgraph 需求分析
    A1[1. ac --pm *create-requirements 建立需求]
    A2[2. ac --pm *create-architecture 建立架構]
    A3[3. ac --pm *create-tasks 建立任務]
    A1 --> A2 --> A3
  end

  subgraph 開發
    B1[4. ac --pm *plan-tasks {task_id} 規劃任務]
    B2[5. ac --dev *develop-tasks {task_id} 開發實作]
    B3[6. ac --qa *review {task_id} 文件審查]
    B1 --> B2 --> B3
  end

  D1{Review 通過?}
  D2{還有任務?}
  R1[ac --dev *brownfield-tasks {task_id} 重開發]
  C1[7. ac --pm *conclude 總結]
  C2[8. ac --pm *curate-knowledge 整理知識]
  Done[完成開發 cycle]

  A3 --> B1
  B3 --> D1
  D1 -- 否 --> R1 --> B2
  D1 -- 是 --> D2
  D2 -- 是 --> B1
  D2 -- 否 --> C1 --> C2 --> Done
```

完成一個開發cycle

## Brownfield

### 需求分析階段
1. ac --pm *create-brownfield-requirements互動式創建需求文檔
2. ac --pm *create-brownfield-architecture互動式創建架構文檔
3. ac --pm *create-brownfield-tasks互動式創建任務文檔

### 開發階段
4. ac --pm *plan-tasks {task_id} 創建特定task的計劃文檔
5. ac --dev *develop-tasks {task_id} 創建特定task的開發文檔
6. ac --qa *review {task_id} 審查特定task的文檔

如果review通過的話，進入第7步，否則回到使用 ac --dev *brownfield-tasks {task_id} 進行重開發
如果還有task的話，回到第4步，否則進入第7步

### 總結階段
7. ac --pm *conclude 總結文檔
8. ac --pm *curate-knowledge 整理知識文檔

#### Mermaid 流程圖
```mermaid
flowchart TD
  subgraph 需求分析
    A1[1. ac --pm *create-brownfield-requirements 建立需求]
    A2[2. ac --pm *create-brownfield-architecture 建立架構]
    A3[3. ac --pm *create-brownfield-tasks 建立任務]
    A1 --> A2 --> A3
  end

  subgraph 開發
    B1[4. ac --pm *plan-tasks {task_id} 規劃任務]
    B2[5. ac --dev *develop-tasks {task_id} 開發實作]
    B3[6. ac --qa *review {task_id} 文件審查]
    B1 --> B2 --> B3
  end

  D1{Review 通過?}
  D2{還有任務?}
  R1[ac --dev *brownfield-tasks {task_id} 重開發]
  C1[7. ac --pm *conclude 總結]
  C2[8. ac --pm *curate-knowledge 整理知識]
  Done[完成開發 cycle]

  A3 --> B1
  B3 --> D1
  D1 -- 否 --> R1 --> B2
  D1 -- 是 --> D2
  D2 -- 是 --> B1
  D2 -- 否 --> C1 --> C2 --> Done
```
