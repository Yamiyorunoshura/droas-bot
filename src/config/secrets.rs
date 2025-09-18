use anyhow::Result;

/// 驗證 Discord Bot 令牌格式
///
/// Discord Bot 令牌遵循特定格式：{application_id}.{timestamp}.{hmac}
/// - 第一部分：17-19 字符的應用ID
/// - 第二部分：6 字符的時間戳
/// - 第三部分：27 字符的HMAC
///
/// # Arguments
/// * `token` - 要驗證的 Discord Bot 令牌
///
/// # Returns
/// * `Ok(())` - 如果令牌格式有效
/// * `Err(...)` - 如果令牌格式無效，包含詳細錯誤信息
pub fn validate_discord_token_format(token: &str) -> Result<()> {
    if token.is_empty() {
        anyhow::bail!("Discord token cannot be empty");
    }

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        anyhow::bail!("Discord token must have 3 parts separated by dots");
    }

    // 檢查第一部分（應用ID）長度 - Discord 的應用 ID 經 base64 編碼後通常是 24 字符
    if parts[0].len() < 18 || parts[0].len() > 30 {
        anyhow::bail!("Discord token first part (application ID) has invalid length");
    }

    // 檢查第二部分（時間戳）長度
    if parts[1].len() != 6 {
        anyhow::bail!("Discord token second part (timestamp) must be 6 characters");
    }

    // 檢查第三部分（HMAC）長度 - Discord 令牌的第三部分通常是 27 字符，但可以有變化
    if parts[2].len() < 20 || parts[2].len() > 40 {
        anyhow::bail!("Discord token third part (HMAC) has invalid length");
    }

    // 檢查是否包含有效的 Base64 字符（包括 URL-safe Base64）
    for (i, part) in parts.iter().enumerate() {
        if !part
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '+' || c == '/')
        {
            anyhow::bail!("Discord token part {} contains invalid characters", i + 1);
        }
    }

    Ok(())
}

/// 通過 Discord API 驗證令牌的有效性
///
/// 這個函數向 Discord API 發送一個測試請求來驗證令牌是否有效。
/// 使用 `/users/@me` 端點來檢查令牌是否能成功認證。
///
/// # Arguments
/// * `token` - 要驗證的 Discord Bot 令牌
///
/// # Returns
/// * `Ok(())` - 如果令牌在 Discord API 中有效
/// * `Err(...)` - 如果令牌無效或API調用失敗
pub async fn validate_discord_token_with_api(token: &str) -> Result<()> {
    if token.is_empty() {
        anyhow::bail!("Cannot validate empty token with API");
    }

    // 首先檢查格式
    validate_discord_token_format(token)?;

    // 創建HTTP客戶端
    let client = reqwest::Client::new();

    // 發送驗證請求到 Discord API
    let response = client
        .get("https://discord.com/api/v10/users/@me")
        .header("Authorization", format!("Bot {}", token))
        .header("User-Agent", "DROAS-Bot/0.1.0")
        .send()
        .await
        .map_err(|e| {
            anyhow::anyhow!("Failed to send verification request to Discord API: {}", e)
        })?;

    match response.status() {
        reqwest::StatusCode::OK => {
            tracing::info!("Discord token validation successful");
            Ok(())
        }
        reqwest::StatusCode::UNAUTHORIZED => {
            anyhow::bail!("Discord token is invalid or expired");
        }
        reqwest::StatusCode::TOO_MANY_REQUESTS => {
            anyhow::bail!("Rate limited by Discord API during token validation");
        }
        status => {
            anyhow::bail!(
                "Unexpected response from Discord API during token validation: {}",
                status
            );
        }
    }
}

/// 安全地記錄令牌相關錯誤，確保不洩露敏感信息
///
/// 這個函數接收包含令牌的錯誤信息，並返回一個清理過的版本，
/// 確保令牌不會出現在日誌或錯誤輸出中。
///
/// # Arguments
/// * `error_message` - 原始錯誤信息
/// * `token` - 需要從錯誤信息中隱藏的令牌
///
/// # Returns
/// * 清理過的錯誤信息，令牌被替換為 "[REDACTED]"
pub fn sanitize_token_error(error_message: &str, token: &str) -> String {
    if token.is_empty() {
        return error_message.to_string();
    }

    error_message.replace(token, "[REDACTED]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_format_validation_valid() {
        let valid_tokens = vec![
            "NDkxNjM4NzExODE0MzY4Mjc3.YH5K_w.valid_token_example_abc123",
            "MTE2NDkxNjM4NzExODE0MzY4.ABC123.another_valid_token_xyz789",
            "NDkwNzI0MDU0NzU2MzUyNTEz.XXXXXX.YYYYYY-ZZZZZZ-WWWWWW-123456", // 更現實的格式
        ];

        for token in valid_tokens {
            let result = validate_discord_token_format(token);
            if let Err(e) = &result {
                println!("令牌 {} 驗證失敗: {}", token, e);
            }
            assert!(result.is_ok(), "令牌 {} 應該被認為是有效格式", token);
        }
    }

    #[test]
    fn test_token_format_validation_invalid() {
        let invalid_tokens = vec![
            "too_short",
            "invalid.format",
            "",
            "no_dots_in_this_token",
            "one.dot",
            "three.dots.but.wrong",
            "NDkxNjM4NzExODE0MzY4Mjc3..empty_middle_part",
        ];

        for token in invalid_tokens {
            let result = validate_discord_token_format(token);
            assert!(result.is_err(), "令牌 {} 應該被認為是無效格式", token);
        }
    }

    #[test]
    fn test_sanitize_token_error() {
        let token = "secret_token_123";
        let error_message =
            "Failed to authenticate with token secret_token_123 due to network error";

        let sanitized = sanitize_token_error(error_message, token);

        assert!(!sanitized.contains(token), "清理後的錯誤信息不應包含令牌");
        assert!(
            sanitized.contains("[REDACTED]"),
            "清理後的錯誤信息應包含 [REDACTED]"
        );
    }

    #[test]
    fn test_sanitize_empty_token() {
        let error_message = "Some error message";
        let sanitized = sanitize_token_error(error_message, "");

        assert_eq!(sanitized, error_message, "空令牌時應返回原始錯誤信息");
    }
}
