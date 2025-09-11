# DROAS Discord 機器人需求規格

版本：1.0.0
最後更新：2025-09-11
語言：繁體中文

## 0. 簡介

- 目的：定義 DROAS Discord 機器人的功能性與非功能性需求、技術方案與交付標準，作為設計、開發、測試與運維之共同依據。
- 範圍：單一 Discord Bot，提供訊息監聽並渲染截圖、群組防護、活躍度計算三大功能，並符合指定的非功能性約束。
- 非目標：
  - 不實作白名單機制
  - 不實作內容敏感詞與審核（需求指定「不過濾敏感內容」）
  - 不提供資料備份策略
- 角色定義：
  - 伺服器擁有者/管理員（Admin）：可設定接收頻道、allowed role、防護敏感度、禁言時長與查看日誌
  - Allowed role：由管理員指定，可執行與管理員相同的管理命令
  - 一般成員：可使用查詢統計與排行榜等資訊指令
  - Bot：以必要權限運行（讀取訊息、管理訊息、對成員 timeout/mute、上傳圖片等）

## 1. 功能性需求

### 1.1 訊息監聽並渲染截圖

- 描述：
  - 監聽公頻訊息事件，動態分析「文字長度、圖片、貼圖」並估算在標準畫布（預設 1080x1920）上的佔用高度。
  - 當累積內容預估高度足以填滿一整張截圖時，自動觸發渲染，將生成的圖片張貼至管理員設定的接收頻道。
  - 每個伺服器僅允許設定一個接收頻道。
  - 不過濾敏感內容（遵循 Discord TOS 與伺服器規則，由管理員自理）。
- 估算與觸發規則（預設可配置）：
  - 畫布：寬 W=1080、高 H=1920、左右邊距 48px、上下邊距 48px
  - 文字：
    - 字體：Noto Sans CJK TC 18px
    - 行高 line_height=28px
    - 平均每行字數 char_per_line≈34（依實際字寬微調）
    - 文本高度估算：sum(ceil(len_i / char_per_line) * line_height)
  - 圖片與貼圖：
    - 圖片採寬度等比縮放，最大預估高度單張 560px（若未知圖尺寸，先以 560px 估算）
    - 貼圖每個約計 160px
  - 預估總高：H_est = header(72) + 文本高度 + Σ圖片高度 + Σ貼圖高度 + 預留內距
  - 觸發閾值：H_est ≥ 0.92 * H 即觸發
  - 邏輯控制：
    - 聚合視窗：最多累積 50 則訊息或 10 分鐘內的訊息（先到先觸發）
    - 觸發後清空累積視窗，從下一則訊息重新計算
    - 若 10 分鐘內未觸發且有累積，強制輸出一張截圖
- 渲染管線（建議方案與備援）：
  - 方案 A（推薦）：Rust + Headless Chrome（chromiumoxide/headless_chrome）以 HTML 模板渲染並截圖
  - 方案 B（備援）：Rust 以標準庫啟動 Node + Puppeteer 外掛程式進行截圖
  - 錯誤處理：渲染失敗重試 2 次；失敗記錄 render_jobs 狀態與錯誤訊息
- 設定與指令：
  - /config set_receive_channel #channel（僅 Admin/allowed role）
  - /render test（產出一張示範截圖驗證字型與樣式）
  - /config show（檢視目前接收頻道與渲染相關設定）
- 驗收標準：
  - 寫入任意文字與貼圖、圖片，當估算高度達閾值，Bot 於 5 秒內發送截圖到已設定頻道
  - 若未設定接收頻道，所有渲染任務拒絕並回報管理員
  - 每伺服器同時僅有一接收頻道設定（唯一性約束）

### 1.2 群組防護（垃圾訊息/惡意連結/機器人攻擊）

- 描述：
  - 即時偵測異常訊息行為並採取禁言（timeout）措施
  - 管理權限由 Admin 或其設定的 allowed role 控制
  - 三級敏感度（低/中/高）影響觸發閾值
  - 禁言時長固定（預設 6 小時），但可由管理員調整
  - 全量記錄防護行動日誌
  - 不需要白名單機制
- 偵測規則（可配置、依敏感度調整）：
  - 訊息速率（同用戶在同頻內）：
    - 低：≥ 8 則/10 秒
    - 中：≥ 5 則/10 秒
    - 高：≥ 3 則/10 秒
  - 重複內容相似度（shingling 或 cosine 相似）：
    - 低：≥ 0.80 連續 ≥ 3 則
    - 中：≥ 0.70 連續 ≥ 2 則
    - 高：≥ 0.60 連續 ≥ 2 則
  - 可疑連結關鍵字/網域（例：discord.gift、nitro、steamcommunity 假冒域、IDN 混淆）：
    - 低：一次出現且伴隨 @everyone/@here 或高頻訊息
    - 中：一次出現即標記加權，如同時滿足速率或重複則觸發
    - 高：單次出現直接觸發（信心提升）
  - 新帳號/新加入：
    - 帳號年齡 < 7 天、進服 < 10 分鐘 且有高頻/連結，觸發權重提高
- 執行動作：
  - 採用 Discord timeout API（建議）或指派 Muted 角色（需伺服器有對應角色與權限）
  - 預設禁言 6 小時，可用 /protect set_mute_duration 調整（1 分鐘 ~ 7 天）
  - 重犯加重：24 小時內再次觸發，時長 ×2（上限 7 天）
- 管理指令：
  - /protect set_sensitivity [low|medium|high]
  - /protect set_mute_duration [數值][m/h/d]
  - /protect logs [limit]（回傳最近 N 筆動作摘要）
  - /moderation unmute @user（解除禁言）
  - /config set_allowed_role @role
- 日誌與稽核：
  - 記錄使用者、規則命中、信心分數、時長、操作人（若手動）、時間戳
  - 可依 guild_id 篩選與時間區間查詢
- 驗收標準：
  - 三種敏感度下，對應範例攻擊樣本能被正確攔截並記錄
  - 禁言成功率 ≥ 99%，錯誤有重試與降級處理

### 1.3 活躍度計算與展示

- 描述：
  - 基於訊息數量進行活躍度統計，提供日/⽉報與排行榜
  - 24 小時無活動後開始衰減，衰減速度適中
  - 前十名排行榜、個人進度條（上限 100）
  - 使用 PostgreSQL 永續化，不提供使用者退出統計選項
- 計算規則：
  - 訊息加分：每則 +2 分（可配置），分數上限 100
  - 衰減：若 idle_hours ≤ 24，無衰減；若 > 24，採指數衰減
    - S(t) = S0 × exp(-k × (idle_hours-24))，建議 k = ln(2)/(7×24) ≈ 0.00413（半衰期約 7 天）
    - 分數向下取整，最低 0，最高 100
  - 日/月統計：以 UTC 或伺服器時區（可配置）將訊息量聚合
- 呈現：
  - /activity top [daily|monthly]：顯示前十名（暱稱、訊息數、活躍度）
  - /activity me：顯示本人日/月訊息數與活躍度進度條（例：██████░░░░ 60/100）
  - /activity summary：顯示伺服器整體活躍度概況
- 驗收標準：
  - 實際訊息寫入後，查詢與排行榜於 3 秒內可見更新
  - 24 小時後未發言，分數開始衰減；再次發言即恢復累積

## 2. 非功能性需求

- 語言與框架：Rust（stable），非主要功能可呼叫外部程序（例如截圖）
- 部署：雲伺服器部署（Linux/容器化建議）
- 效能與容量：至少支援數百用戶並發；事件處理 P95 < 200ms（不含渲染）；渲染任務佇列化處理
- 資料庫：PostgreSQL（連線池、索引、遷移機制）
- 監控：系統資源（CPU/RAM/磁碟/網路）、Discord API 調用次數與限流、錯誤率、事件延遲
- 安全：Token 管理（環境變數/祕密管理）、權限最小化、速率限制與重試
- 備份：不需要資料備份（按需求指定）
- 可觀測性：結構化日誌（JSON）與 Prometheus 指標輸出
- 可維運性：健康檢查、滾動更新、配置熱載入（選配）

## 3. 架構與技術選型

- 事件處理：serenity 或 twilight + tokio（async）
- 指令系統：poise（slash commands）或 serenity 的 application commands
- DB：sqlx（postgres, runtime=tokio, tls=rustls），遷移工具 sqlx-cli/refinery
- 佇列/任務：tokio 任務 + 優先序隊列（渲染任務）
- 渲染：優先 chromiumoxide/headless_chrome；備援 Node+Puppeteer 子程序
- 設定：config/figment + 環境變數
- 日誌：tracing, tracing-subscriber（JSON）
- 指標：metrics + metrics-exporter-prometheus（/metrics）
- 主要元件：
  - Gateway/Event Listener、Command Handler
  - Render Planner/Worker
  - Protection Engine
  - Activity Service
  - Persistence 層（Repository）
  - Telemetry 層（Logging/Metrics）
- 速率與錯誤處理：
  - Discord API 依官方限流回退
  - DB 重試策略（指數退避，最大 3 次）
  - 渲染失敗重試 2 次，最終記錄失敗並通知管理員

## 4. 資料庫設計（PostgreSQL，建議 DDL）

```sql
-- 伺服器配置
create table if not exists guild_config (
  guild_id            bigint primary key,
  receive_channel_id  bigint not null,
  allowed_role_id     bigint,
  protection_sensitivity text not null check (protection_sensitivity in ('low','medium','high')),
  mute_duration_seconds integer not null default 21600, -- 6h
  screenshot_w        integer not null default 1080,
  screenshot_h        integer not null default 1920,
  timezone            text not null default 'UTC',
  created_at          timestamptz not null default now(),
  updated_at          timestamptz not null default now()
);

-- 活躍度訊息基礎事件
create table if not exists activity_messages (
  message_id    bigint primary key,
  guild_id      bigint not null,
  channel_id    bigint not null,
  author_id     bigint not null,
  created_at    timestamptz not null,
  is_deleted    boolean not null default false
);
create index if not exists idx_activity_messages_guild_time on activity_messages(guild_id, created_at);
create index if not exists idx_activity_messages_author_time on activity_messages(author_id, created_at);

-- 活躍度彙總
create table if not exists activity_user_daily (
  guild_id    bigint not null,
  user_id     bigint not null,
  date        date not null,
  msg_count   integer not null default 0,
  primary key (guild_id, user_id, date)
);

create table if not exists activity_user_monthly (
  guild_id    bigint not null,
  user_id     bigint not null,
  yyyymm      integer not null, -- e.g. 202509
  msg_count   integer not null default 0,
  primary key (guild_id, user_id, yyyymm)
);

create table if not exists activity_user_score (
  guild_id         bigint not null,
  user_id          bigint not null,
  score            integer not null default 0 check (score between 0 and 100),
  last_message_at  timestamptz,
  updated_at       timestamptz not null default now(),
  primary key (guild_id, user_id)
);

-- 防護行為日誌
create table if not exists protection_action_log (
  id                bigserial primary key,
  guild_id          bigint not null,
  user_id           bigint not null,
  action            text not null check (action in ('mute','unmute','skip')),
  reason            text,
  rule              text,
  sensitivity       text not null,
  confidence        numeric(4,3),
  duration_seconds  integer,
  moderator_id      bigint, -- 若為手動操作
  created_at        timestamptz not null default now()
);
create index if not exists idx_protection_log_guild_time on protection_action_log(guild_id, created_at);

-- 渲染任務
create table if not exists render_jobs (
  id                 uuid primary key,
  guild_id           bigint not null,
  channel_id         bigint not null,
  msg_id_start       bigint,
  msg_id_end         bigint,
  estimated_height   integer,
  status             text not null check (status in ('queued','rendering','success','failed')),
  error              text,
  created_at         timestamptz not null default now(),
  updated_at         timestamptz not null default now()
);
create index if not exists idx_render_jobs_guild_status on render_jobs(guild_id, status);
```

## 5. 指令與權限

- 權限控制：僅 Admin 或 allowed role 可變更配置與執行控管類指令；一般成員可查詢統計
- 設定與管理：
  - /config set_receive_channel #channel
  - /config set_allowed_role @role
  - /config show
  - /protect set_sensitivity [low|medium|high]
  - /protect set_mute_duration [數值][m/h/d]
  - /protect logs [limit]
  - /moderation unmute @user
  - /render test
- 統計與展示：
  - /activity top [daily|monthly]
  - /activity me
  - /activity summary

## 6. 系統監控與告警

- 指標（Prometheus）：
  - droas_messages_seen_total、droas_render_jobs_total、droas_render_failures_total
  - droas_protection_actions_total、droas_activity_updates_total
  - droas_discord_api_requests_total、droas_rate_limit_hits_total
  - droas_event_latency_seconds_bucket、droas_db_query_seconds_bucket
  - process_cpu_seconds_total、process_resident_memory_bytes
- 日誌：JSON 格式（含 guild_id、channel_id、user_id、event、error_code）
- 告警建議：
  - 渲染失敗率 > 5%（5 分鐘）告警
  - API 限流命中率突增告警
  - 事件延遲 P95 > 1s 告警

## 7. 部署與設定

- 雲伺服器：Linux（Ubuntu 22.04+），建議容器化（Docker）
- 規格建議（依量級調整）：
  - 輕量：2 vCPU / 4GB RAM / 20GB SSD
  - 有截圖高峰：4 vCPU / 8GB RAM / 40GB SSD（需 headless Chrome）
- 環境變數：
  - DISCORD_TOKEN, DATABASE_URL, RUST_LOG, METRICS_PORT
  - RENDERER_MODE=[chrome|puppeteer], CHROME_PATH（若需）
  - TZ（時區，預設 UTC 或 Asia/Taipei）
- 服務：
  - systemd 或容器 orchestrator（healthcheck, restart=always）
  - 滾動更新、灰度釋出

## 8. 測試計畫

- 單元測試：估高演算法、相似度計算、衰減公式、SQL repository
- 整合測試：指令權限、DB 遷移、渲染任務佇列
- 負載測試：訊息洪峰（100 msg/s）、渲染併發（隊列限速）
- 安全測試：惡意連結、IDN 混淆、速率限制
- 驗收測試：功能清單逐條驗收（見各功能驗收標準）

## 9. 風險與緩解

- 渲染相依 headless 瀏覽器：提供 Node+Puppeteer 備援；失敗重試與降級（僅文字圖片拼貼）
- Discord 限流：內建速率與重試，關鍵操作排隊
- DB 壓力：索引與彙總表設計；批次寫入；監控慢查詢
- 過度攔截：提供敏感度切換與解除禁言指令；動作日誌可審核

## 10. 里程碑（建議）

- M1（週 1-2）：骨架、指令、DB 遷移、活動計算基礎
- M2（週 3-4）：渲染估算與截圖、接收頻道設定、render 任務佇列
- M3（週 5-6）：群組防護引擎、日誌與告警
- M4（週 7）：性能調優、文件與驗收

## 11. 開放問題

- 渲染主題（深色/淺色）與字型授權是否需客製？
- 伺服器時區預設值？是否允許每伺服器自訂？
- 防護誤攔截的人為審核流程是否需消息通知？
