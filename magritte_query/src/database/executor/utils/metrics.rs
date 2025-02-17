use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct ExecutorMetrics {
    pub active_connections: AtomicUsize,
    pub idle_connections: AtomicUsize,
    pub queries_executed: AtomicUsize,
    pub queries_failed: AtomicUsize,
    pub total_query_time: AtomicUsize,
}

impl ExecutorMetrics {
    pub fn new() -> Self {
        Self {
            active_connections: AtomicUsize::new(0),
            idle_connections: AtomicUsize::new(0),
            queries_executed: AtomicUsize::new(0),
            queries_failed: AtomicUsize::new(0),
            total_query_time: AtomicUsize::new(0),
        }
    }

    pub fn update_success(&self, duration: usize) {
        self.queries_executed.fetch_add(1, Ordering::Relaxed);
        self.total_query_time.fetch_add(duration, Ordering::Relaxed);
    }

    pub fn update_failure(&self) {
        self.queries_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Get total number of queries (both successful and failed)
    pub fn total_queries(&self) -> usize {
        self.queries_executed.load(Ordering::Relaxed) + self.queries_failed.load(Ordering::Relaxed)
    }

    /// Get average query duration in microseconds
    pub fn average_query_duration(&self) -> Option<usize> {
        let total = self.queries_executed.load(Ordering::Relaxed);
        if total == 0 {
            None
        } else {
            Some(self.total_query_time.load(Ordering::Relaxed) / total)
        }
    }

    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        let total = self.total_queries();
        if total == 0 {
            100.0
        } else {
            let successful = self.queries_executed.load(Ordering::Relaxed);
            (successful as f64 / total as f64) * 100.0
        }
    }
}