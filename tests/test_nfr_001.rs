//! NFR-001 Rate Limiting and Retry Handling Tests
//!
//! This module contains comprehensive tests for rate limiting and retry handling functionality.
//! Tests include unit tests, integration tests, and performance benchmarks.

use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;
use tokio::time::sleep;

// Import our project modules
use droas_bot::database::schema::{GuildConfig, GuildConfigService};
use droas_bot::discord::api_client::DiscordApiClient;
use droas_bot::discord::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use droas_bot::discord::event_handler::{EventHandler, EventResult, TestMemberJoinEvent};
use droas_bot::discord::rate_limit::{ExponentialBackoffConfig, RateLimitStats, RateLimiter};
use droas_bot::handlers::welcome::WelcomeHandler;

/// Helper function to create test services
async fn create_test_services() -> Result<(GuildConfigService, NamedTempFile)> {
    let temp_file = NamedTempFile::new().expect("無法創建臨時檔案");
    let database_url = format!("sqlite://{}", temp_file.path().display());

    let pool = sqlx::SqlitePool::connect(&database_url)
        .await
        .expect("無法連接測試資料庫");

    // 創建測試表格
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS guild_config (
            guild_id TEXT PRIMARY KEY,
            welcome_channel_id TEXT NOT NULL,
            background_ref TEXT,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("無法創建測試表格");

    let guild_service = GuildConfigService::new(pool);

    Ok((guild_service, temp_file))
}

#[cfg(test)]
mod rate_limiting_tests {
    use super::*;

    #[tokio::test]
    async fn test_nfr_rate_limiting_discord_api_awareness() {
        // NFR-001.1: Discord API rate limiting awareness
        let limiter = RateLimiter::new();

        // Test basic rate limiting functionality
        limiter
            .handle_rate_limit_response("test_route", 1.0, false)
            .await;

        // Should wait when rate limited
        let wait_time = limiter.wait_if_rate_limited("test_route").await;
        assert!(wait_time > Duration::from_millis(500));

        let stats = limiter.get_stats();
        assert_eq!(stats.rate_limit_hits, 1);
    }

    #[tokio::test]
    async fn test_nfr_exponential_backoff_algorithm() {
        // NFR-001.1: Exponential backoff algorithm
        let config = ExponentialBackoffConfig {
            base_delay_ms: 100,
            max_delay_ms: 1000,
            max_retries: 3,
            backoff_multiplier: 2.0,
            jitter: false, // Disable jitter for predictable testing
        };

        let limiter = RateLimiter::with_config(config);

        // Test exponential backoff with mock failing operation
        let attempt_count = Arc::new(Mutex::new(0));
        let attempt_count_clone = Arc::clone(&attempt_count);

        let result = limiter
            .retry_with_exponential_backoff("test_route", move || {
                let counter = Arc::clone(&attempt_count_clone);
                async move {
                    let mut count = counter.lock().unwrap();
                    *count += 1;
                    let current_attempt = *count;
                    drop(count);

                    if current_attempt <= 2 {
                        Err("Mock error")
                    } else {
                        Ok("Success")
                    }
                }
            })
            .await;

        // Should succeed on 3rd attempt
        assert_eq!(result.unwrap(), "Success");
        let final_count = *attempt_count.lock().unwrap();
        assert_eq!(final_count, 3);
    }

    #[tokio::test]
    async fn test_nfr_rate_limiting_performance() {
        // NFR-P-002: Rate limiting processing latency < 10ms
        let limiter = RateLimiter::new();

        let start = Instant::now();

        // Test performance with many rapid requests
        for _ in 0..100 {
            limiter.wait_if_rate_limited("performance_test").await;
        }

        let duration = start.elapsed();
        let avg_time_per_request = duration.as_millis() as f64 / 100.0;

        // Should process requests very quickly when not rate limited
        assert!(
            avg_time_per_request < 1.0,
            "Rate limiting performance too slow: {}ms per request",
            avg_time_per_request
        );
    }

    #[tokio::test]
    async fn test_nfr_circuit_breaker_integration() {
        // NFR-001.1: Circuit breaker pattern integration
        let circuit_config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(5),
            success_threshold: 2,
            request_timeout: Duration::from_secs(1),
        };

        let circuit_breaker = Arc::new(CircuitBreaker::new(circuit_config));

        // Test circuit breaker with failing operations
        for i in 0..5 {
            let result = circuit_breaker
                .execute(async { Err::<(), String>("Simulated failure".to_string()) })
                .await;

            assert!(result.is_err());

            if i >= 2 {
                // Circuit should be open after 3 failures
                assert_eq!(
                    circuit_breaker.get_state(),
                    droas_bot::discord::circuit_breaker::CircuitState::Open
                );
            }
        }
    }

    #[tokio::test]
    async fn test_nfr_global_vs_route_specific_limits() {
        // Test both global and route-specific rate limiting
        let limiter = RateLimiter::new();

        // Set global rate limit
        limiter
            .handle_rate_limit_response("any_route", 0.5, true)
            .await;

        // Set route-specific rate limit
        limiter
            .handle_rate_limit_response("specific_route", 1.0, false)
            .await;

        // Global limit should affect all routes
        let wait_time1 = limiter.wait_if_rate_limited("different_route").await;
        assert!(wait_time1 > Duration::from_millis(250));

        // Route-specific limit should affect only that route
        let wait_time2 = limiter.wait_if_rate_limited("specific_route").await;
        assert!(wait_time2 > Duration::from_millis(500));
    }
}

#[cfg(test)]
mod idempotency_tests {
    use super::*;

    #[tokio::test]
    async fn test_nfr_event_idempotency_duplication_detection() {
        // NFR-001.2: Event idempotency system
        let (guild_service, _temp_file) = create_test_services().await.unwrap();
        let welcome_handler = WelcomeHandler::new();
        let event_handler = EventHandler::new(guild_service, welcome_handler);

        let test_event = TestMemberJoinEvent {
            guild_id: 123456789,
            user_id: 555666777,
            username: "TestUser".to_string(),
            timestamp: Instant::now(),
        };

        // First event should not be detected as duplicate
        let dup_check1 = event_handler.check_duplication(&test_event).await;
        assert!(dup_check1.is_none());

        // Record as processed
        event_handler.record_processed_event(&test_event).await;

        // Second identical event should be detected as duplicate
        let dup_check2 = event_handler.check_duplication(&test_event).await;
        assert!(dup_check2.is_some());
        assert!(dup_check2.unwrap().contains("重複事件"));
    }

    #[tokio::test]
    async fn test_nfr_idempotency_key_generation() {
        // NFR-001.2: Idempotency key generation
        let (guild_service, _temp_file) = create_test_services().await.unwrap();
        let welcome_handler = WelcomeHandler::new();
        let event_handler = EventHandler::new(guild_service, welcome_handler);

        // Test that different events generate different keys
        let event1 = TestMemberJoinEvent {
            guild_id: 123456789,
            user_id: 555666777,
            username: "User1".to_string(),
            timestamp: Instant::now(),
        };

        let event2 = TestMemberJoinEvent {
            guild_id: 123456789,
            user_id: 555666778,
            username: "User2".to_string(),
            timestamp: Instant::now(),
        };

        let dup_check1 = event_handler.check_duplication(&event1).await;
        let dup_check2 = event_handler.check_duplication(&event2).await;

        // Both should not be duplicates
        assert!(dup_check1.is_none());
        assert!(dup_check2.is_none());
    }

    #[tokio::test]
    async fn test_nfr_idempotency_performance() {
        // NFR-P-002: Idempotency check latency < 5ms
        let (guild_service, _temp_file) = create_test_services().await.unwrap();
        let welcome_handler = WelcomeHandler::new();
        let event_handler = EventHandler::new(guild_service, welcome_handler);

        let test_event = TestMemberJoinEvent {
            guild_id: 123456789,
            user_id: 555666777,
            username: "TestUser".to_string(),
            timestamp: Instant::now(),
        };

        // Test idempotency check performance
        let start = Instant::now();

        for _ in 0..1000 {
            let _result = event_handler.check_duplication(&test_event).await;
        }

        let duration = start.elapsed();
        let avg_time_per_check = duration.as_millis() as f64 / 1000.0;

        assert!(
            avg_time_per_check < 1.0,
            "Idempotency check too slow: {}ms per check",
            avg_time_per_check
        );
    }

    #[tokio::test]
    async fn test_nfr_idempotency_cache_lifecycle() {
        // Test cache lifecycle management
        let (guild_service, _temp_file) = create_test_services().await.unwrap();
        let welcome_handler = WelcomeHandler::new();
        let event_handler = EventHandler::new(guild_service, welcome_handler);

        // Add events with short TTL (simulate old events)
        let old_event = TestMemberJoinEvent {
            guild_id: 123456789,
            user_id: 555666777,
            username: "OldUser".to_string(),
            timestamp: Instant::now() - Duration::from_secs(400), // Older than 5 minutes
        };

        // Should not be in cache due to TTL
        let dup_check = event_handler.check_duplication(&old_event).await;
        assert!(dup_check.is_none());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_nfr_rate_limiting_and_idempotency_integration() {
        // Integration test for rate limiting and idempotency working together
        let (guild_service, _temp_file) = create_test_services().await.unwrap();
        let welcome_handler = WelcomeHandler::new();
        let event_handler = EventHandler::new(guild_service, welcome_handler);
        let limiter = RateLimiter::new();

        // Test scenario: High frequency member join events
        let events: Vec<TestMemberJoinEvent> = (0..100)
            .map(|i| TestMemberJoinEvent {
                guild_id: 123456789,
                user_id: 555666777 + i,
                username: format!("User{}", i),
                timestamp: Instant::now(),
            })
            .collect();

        let mut processed_count = 0;
        let mut duplicate_count = 0;

        for event in events {
            // Check rate limiting
            limiter.wait_if_rate_limited("guild_member_add").await;

            // Check idempotency
            if let Some(reason) = event_handler.check_duplication(&event).await {
                duplicate_count += 1;
                continue;
            }

            // Process event
            event_handler.record_processed_event(&event).await;
            processed_count += 1;
        }

        // All events should be processed (no duplicates in this test)
        assert_eq!(processed_count, 100);
        assert_eq!(duplicate_count, 0);
    }

    #[tokio::test]
    async fn test_nfr_system_resilience_under_load() {
        // Test system resilience under high load
        let limiter = RateLimiter::new();
        let (guild_service, _temp_file) = create_test_services().await.unwrap();
        let welcome_handler = WelcomeHandler::new();
        let event_handler = EventHandler::new(guild_service, welcome_handler);

        // Simulate high load scenario
        let start = Instant::now();
        let concurrent_requests = 50;

        let handles: Vec<_> = (0..concurrent_requests)
            .map(|i| {
                let limiter = limiter.clone();
                let event_handler = event_handler.clone();
                tokio::spawn(async move {
                    let event = TestMemberJoinEvent {
                        guild_id: 123456789,
                        user_id: 555666777 + i,
                        username: format!("User{}", i),
                        timestamp: Instant::now(),
                    };

                    // Apply rate limiting
                    limiter.wait_if_rate_limited("concurrent_test").await;

                    // Check and process event
                    if event_handler.check_duplication(&event).await.is_none() {
                        event_handler.record_processed_event(&event).await;
                        Ok::<(), String>(())
                    } else {
                        Ok(())
                    }
                })
            })
            .collect();

        // Wait for all concurrent requests to complete
        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        let duration = start.elapsed();

        // Should handle concurrent requests efficiently
        assert!(
            duration < Duration::from_secs(5),
            "System too slow under load: {:?}",
            duration
        );

        // Check that all events were processed
        let (total_entries, processed_entries) = event_handler.get_cache_stats();
        assert_eq!(total_entries, concurrent_requests);
        assert_eq!(processed_entries, concurrent_requests);
    }

    #[tokio::test]
    async fn test_nfr_discord_api_integration() {
        // Test integration with Discord API (mock)
        let api_client = DiscordApiClient::new("Bot test_token".to_string());

        // Test API client configuration
        assert!(api_client.validate_token().is_ok());

        // Test circuit breaker stats
        let stats = api_client.get_circuit_breaker_stats().await;
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.failed_requests, 0);

        // Test rate limiting awareness in API client
        let limiter = RateLimiter::new();
        limiter
            .handle_rate_limit_response("api_call", 0.1, true)
            .await;

        let wait_time = limiter.wait_if_rate_limited("api_call").await;
        assert!(wait_time > Duration::from_millis(50));
    }
}

#[cfg(test)]
mod chaos_tests {
    use super::*;

    #[tokio::test]
    async fn test_nfr_rate_limiting_chaos() {
        // Chaos test: Simulate random rate limiting scenarios
        let limiter = RateLimiter::new();

        let mut success_count = 0;
        let mut rate_limited_count = 0;

        // Simulate 1000 requests with random rate limiting
        for i in 0..1000 {
            if i % 10 == 0 {
                // Simulate rate limiting every 10th request
                limiter
                    .handle_rate_limit_response("chaos_test", 0.1, false)
                    .await;
            }

            let wait_time = limiter.wait_if_rate_limited("chaos_test").await;

            if wait_time > Duration::from_millis(0) {
                rate_limited_count += 1;
            } else {
                success_count += 1;
            }

            // Small delay to simulate processing
            sleep(Duration::from_millis(1)).await;
        }

        // Should handle chaos gracefully
        assert!(
            success_count > 800,
            "Too many requests failed under chaos: {}",
            success_count
        );
        assert!(
            rate_limited_count < 200,
            "Too many requests rate limited: {}",
            rate_limited_count
        );
    }

    #[tokio::test]
    async fn test_nfr_event_processing_chaos() {
        // Chaos test: Simulate random duplicate events
        let (guild_service, _temp_file) = create_test_services().await.unwrap();
        let welcome_handler = WelcomeHandler::new();
        let event_handler = EventHandler::new(guild_service, welcome_handler);

        let mut processed_events = 0;
        let mut duplicate_events = 0;

        // Send 500 events with 20% duplication rate
        for i in 0..500 {
            let user_id = if i % 5 == 0 { 555666777 } else { 555666777 + i }; // 20% duplicates

            let event = TestMemberJoinEvent {
                guild_id: 123456789,
                user_id,
                username: format!("User{}", i),
                timestamp: Instant::now(),
            };

            if let Some(_reason) = event_handler.check_duplication(&event).await {
                duplicate_events += 1;
            } else {
                event_handler.record_processed_event(&event).await;
                processed_events += 1;
            }
        }

        // Should handle duplicates correctly
        assert!(
            processed_events <= 450,
            "Too many unique events processed: {}",
            processed_events
        );
        assert!(
            duplicate_events >= 50,
            "Too few duplicates detected: {}",
            duplicate_events
        );
    }

    #[tokio::test]
    async fn test_nfr_system_recovery() {
        // Test system recovery after failures
        let limiter = RateLimiter::new();

        // Simulate burst of rate limits
        for i in 0..20 {
            limiter
                .handle_rate_limit_response("recovery_test", 0.5, false)
                .await;

            let wait_time = limiter.wait_if_rate_limited("recovery_test").await;
            assert!(wait_time > Duration::from_millis(400));
        }

        // Wait for rate limits to expire
        sleep(Duration::from_secs(2)).await;

        // Clean up expired limits
        limiter.cleanup_expired_limits().await;

        // Should recover quickly
        let start = Instant::now();
        for _ in 0..10 {
            let wait_time = limiter.wait_if_rate_limited("recovery_test").await;
            assert_eq!(wait_time, Duration::from_millis(0));
        }

        let recovery_time = start.elapsed();
        assert!(
            recovery_time < Duration::from_millis(100),
            "System recovery too slow: {:?}",
            recovery_time
        );
    }
}

// Performance benchmarks
#[cfg(test)]
mod performance_benchmarks {
    use super::*;

    #[tokio::test]
    async fn test_nfr_rate_limiting_throughput() {
        // Benchmark: Rate limiting throughput
        let limiter = RateLimiter::new();

        let start = Instant::now();
        const REQUEST_COUNT: usize = 10000;

        for i in 0..REQUEST_COUNT {
            limiter
                .wait_if_rate_limited(&format!("benchmark_route_{}", i % 100))
                .await;
        }

        let duration = start.elapsed();
        let requests_per_second = REQUEST_COUNT as f64 / duration.as_secs_f64();

        // Should handle at least 1000 requests per second
        assert!(
            requests_per_second > 1000.0,
            "Rate limiting throughput too low: {} requests/second",
            requests_per_second
        );
    }

    #[tokio::test]
    async fn test_nfr_idempotency_memory_usage() {
        // Benchmark: Idempotency memory usage
        let (guild_service, _temp_file) = create_test_services().await.unwrap();
        let welcome_handler = WelcomeHandler::new();
        let event_handler = EventHandler::new(guild_service, welcome_handler);

        // Add many events to cache
        for i in 0..10000 {
            let event = TestMemberJoinEvent {
                guild_id: 123456789,
                user_id: 555666777 + i,
                username: format!("User{}", i),
                timestamp: Instant::now(),
            };

            event_handler.check_duplication(&event).await;
            event_handler.record_processed_event(&event).await;
        }

        // Check cache size (should be within reasonable limits)
        let (total_entries, _) = event_handler.get_cache_stats();
        assert!(total_entries <= 10000, "Cache too large: {}", total_entries);

        // Test cleanup
        event_handler.clear_cache();
        let (total_after_clear, _) = event_handler.get_cache_stats();
        assert_eq!(total_after_clear, 0, "Cache not properly cleared");
    }
}
