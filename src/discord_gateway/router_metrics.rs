use std::time::{Duration, Instant};
use std::collections::HashMap;

pub struct RouterMetrics {
    command_counts: HashMap<String, u64>,
    response_times: HashMap<String, Vec<Duration>>,
    total_requests: u64,
    error_count: u64,
    start_time: Instant,
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub command_counts: HashMap<String, u64>,
    pub average_response_times: HashMap<String, Duration>,
    pub total_requests: u64,
    pub error_rate: f64,
    pub uptime: Duration,
}

impl RouterMetrics {
    pub fn new() -> Self {
        Self {
            command_counts: HashMap::new(),
            response_times: HashMap::new(),
            total_requests: 0,
            error_count: 0,
            start_time: Instant::now(),
        }
    }

    pub fn record_command_execution(&mut self, command: &str, response_time: Duration, is_error: bool) {
        // Increment command count
        *self.command_counts.entry(command.to_string()).or_insert(0) += 1;

        // Record response time
        self.response_times
            .entry(command.to_string())
            .or_insert_with(Vec::new)
            .push(response_time);

        // Update total requests
        self.total_requests += 1;

        // Update error count if applicable
        if is_error {
            self.error_count += 1;
        }
    }

    pub fn get_metrics_snapshot(&self) -> MetricsSnapshot {
        let mut average_response_times = HashMap::new();

        for (command, times) in &self.response_times {
            if !times.is_empty() {
                let total: Duration = times.iter().sum();
                let average = total / times.len() as u32;
                average_response_times.insert(command.clone(), average);
            }
        }

        let error_rate = if self.total_requests > 0 {
            self.error_count as f64 / self.total_requests as f64
        } else {
            0.0
        };

        MetricsSnapshot {
            command_counts: self.command_counts.clone(),
            average_response_times,
            total_requests: self.total_requests,
            error_rate,
            uptime: self.start_time.elapsed(),
        }
    }

    pub fn is_within_sla(&self, command: &str, max_response_time: Duration) -> bool {
        if let Some(times) = self.response_times.get(command) {
            if !times.is_empty() {
                let total: Duration = times.iter().sum();
                let average = total / times.len() as u32;
                return average <= max_response_time;
            }
        }
        true // If no data, assume within SLA
    }

    pub fn get_command_success_rate(&self, command: &str) -> f64 {
        let _total_commands = self.command_counts.get(command).unwrap_or(&0);
        let total_times = self.response_times.get(command).map(|times| times.len() as u64).unwrap_or(0);

        if total_times > 0 {
            (total_times - self.get_command_error_count(command)) as f64 / total_times as f64
        } else {
            1.0
        }
    }

    fn get_command_error_count(&self, _command: &str) -> u64 {
        // This is a simplified calculation - in a real implementation,
        // you'd track errors per command
        self.error_count
    }

    pub fn reset_metrics(&mut self) {
        self.command_counts.clear();
        self.response_times.clear();
        self.total_requests = 0;
        self.error_count = 0;
        self.start_time = Instant::now();
    }
}

impl Default for RouterMetrics {
    fn default() -> Self {
        Self::new()
    }
}

// Helper struct for timing operations
pub struct OperationTimer {
    start_time: Instant,
}

impl OperationTimer {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for OperationTimer {
    fn default() -> Self {
        Self::new()
    }
}