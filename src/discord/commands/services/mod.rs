//! 命令處理共享服務模組
//! 
//! 提供各種命令處理器所需的共享服務，包括HTTP、圖片處理、資產管理等。

pub mod http_service;

pub use http_service::{HttpService, HttpServiceConfig, create_shared_http_service};

/// 所有共享服務的集合
#[derive(Clone)]
pub struct SharedServices {
    /// HTTP服務
    pub http_service: std::sync::Arc<HttpService>,
}

impl SharedServices {
    /// 創建新的共享服務實例
    pub fn new() -> crate::discord::commands::framework::CommandResult<Self> {
        let http_service = create_shared_http_service()?;
        
        Ok(Self {
            http_service,
        })
    }
}

impl Default for SharedServices {
    fn default() -> Self {
        Self::new().expect("無法創建默認共享服務")
    }
}