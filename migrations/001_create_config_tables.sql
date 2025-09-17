-- 公會配置管理資料庫遷移
-- Version: 001
-- Description: 創建公會配置和背景圖片資源管理表

-- 背景圖片資源表
-- 儲存上傳的背景圖片文件信息
CREATE TABLE IF NOT EXISTS background_assets (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL UNIQUE,
    media_type TEXT NOT NULL,
    file_size INTEGER NOT NULL CHECK (file_size > 0),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 為文件路径創建索引以提高查詢性能
CREATE INDEX IF NOT EXISTS idx_background_assets_file_path ON background_assets(file_path);

-- 公會配置表
-- 儲存每個公會的個別配置設定
CREATE TABLE IF NOT EXISTS guild_configs (
    guild_id INTEGER PRIMARY KEY,
    welcome_channel_id INTEGER,
    background_ref TEXT,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 外鍵約束：背景圖片引用必須存在於 background_assets 表中
    FOREIGN KEY (background_ref) REFERENCES background_assets(id) ON DELETE SET NULL
);

-- 為配置查詢創建索引
CREATE INDEX IF NOT EXISTS idx_guild_configs_welcome_channel ON guild_configs(welcome_channel_id);
CREATE INDEX IF NOT EXISTS idx_guild_configs_background_ref ON guild_configs(background_ref);
CREATE INDEX IF NOT EXISTS idx_guild_configs_updated_at ON guild_configs(updated_at);

-- 創建觸發器以自動更新 updated_at 時間戳
CREATE TRIGGER IF NOT EXISTS update_guild_configs_updated_at 
    AFTER UPDATE ON guild_configs
    FOR EACH ROW
    WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE guild_configs SET updated_at = CURRENT_TIMESTAMP WHERE guild_id = NEW.guild_id;
END;

-- 插入一些測試數據（僅在開發環境中）
-- 注意：生產環境部署時應該移除這些測試數據

-- 測試背景圖片資源
INSERT OR IGNORE INTO background_assets (id, file_path, media_type, file_size, created_at) VALUES
    ('default_bg_001', '/assets/backgrounds/default.png', 'image/png', 1024000, CURRENT_TIMESTAMP),
    ('sample_bg_002', '/assets/backgrounds/sample.jpeg', 'image/jpeg', 2048000, CURRENT_TIMESTAMP);

-- 測試公會配置
INSERT OR IGNORE INTO guild_configs (guild_id, welcome_channel_id, background_ref, created_at, updated_at) VALUES
    (123456789, 987654321, 'default_bg_001', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    (111111111, 222222222, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);