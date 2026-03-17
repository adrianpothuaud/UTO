//! Latency instrumentation and guardrails for Phase 3 intent resolution.

use std::time::Instant;

/// Latency tracking for intent resolution operations.
#[derive(Debug, Clone)]
pub struct LatencyTracker {
    /// Individual operation durations in milliseconds.
    pub samples: Vec<u64>,
    /// Operation name for diagnostics.
    pub operation: String,
}

impl LatencyTracker {
    /// Creates a new latency tracker.
    pub fn new(operation: &str) -> Self {
        Self {
            samples: Vec::new(),
            operation: operation.to_string(),
        }
    }

    /// Records a measurement (in milliseconds).
    pub fn record(&mut self, duration_ms: u64) {
        self.samples.push(duration_ms);
    }

    /// Computes the median of recorded samples.
    pub fn median(&self) -> Option<u64> {
        if self.samples.is_empty() {
            return None;
        }
        let mut sorted = self.samples.clone();
        sorted.sort_unstable();
        let mid = sorted.len() / 2;
        Some(if sorted.len().is_multiple_of(2) {
            (sorted[mid - 1] + sorted[mid]) / 2
        } else {
            sorted[mid]
        })
    }

    /// Computes the 95th percentile of recorded samples.
    pub fn p95(&self) -> Option<u64> {
        if self.samples.is_empty() {
            return None;
        }
        let mut sorted = self.samples.clone();
        sorted.sort_unstable();
        let idx = ((sorted.len() as f64) * 0.95).ceil() as usize;
        Some(sorted[idx.saturating_sub(1)])
    }

    /// Computes mean latency.
    pub fn mean(&self) -> Option<u64> {
        if self.samples.is_empty() {
            return None;
        }
        let sum: u64 = self.samples.iter().sum();
        Some(sum / self.samples.len() as u64)
    }

    /// Returns a summary string for logs.
    pub fn summary(&self) -> String {
        match (self.median(), self.p95(), self.mean()) {
            (Some(med), Some(p95), Some(mean)) => format!(
                "{}: median={:.0}ms, p95={:.0}ms, mean={:.0}ms, samples={}",
                self.operation,
                med as f32,
                p95 as f32,
                mean as f32,
                self.samples.len()
            ),
            _ => format!("{}: no samples", self.operation),
        }
    }

    /// Checks if the current metrics meet the expected SLA.
    pub fn meets_sla(&self, max_median_ms: u64, max_p95_ms: u64) -> bool {
        match (self.median(), self.p95()) {
            (Some(med), Some(p95)) => med <= max_median_ms && p95 <= max_p95_ms,
            _ => true,
        }
    }
}

/// Stores a scoped latency measurement for stopwatch-like timing.
pub struct ScopedLatency {
    start: Instant,
    tracker: *mut LatencyTracker,
}

impl ScopedLatency {
    /// Creates a new scoped latency measurement linked to a tracker.
    ///
    /// When this value is dropped, the elapsed time is automatically recorded
    /// in the tracker.
    pub fn new(tracker: &mut LatencyTracker) -> Self {
        Self {
            start: Instant::now(),
            tracker,
        }
    }
}

impl Drop for ScopedLatency {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_millis() as u64;
        unsafe {
            (*self.tracker).record(elapsed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latency_tracker_computes_median() {
        let mut tracker = LatencyTracker::new("test-op");
        tracker.record(10);
        tracker.record(20);
        tracker.record(30);

        assert_eq!(tracker.median(), Some(20));
    }

    #[test]
    fn latency_tracker_computes_p95() {
        let mut tracker = LatencyTracker::new("test-op");
        // Add 100 samples: 1..100
        for i in 1..=100 {
            tracker.record(i);
        }

        let p95 = tracker.p95().expect("p95 should be computed");
        // 95th percentile of 1..100 should be around 95
        assert!((94..=96).contains(&p95), "p95={}", p95);
    }

    #[test]
    fn latency_tracker_meets_sla() {
        let mut tracker = LatencyTracker::new("test-op");
        tracker.record(10);
        tracker.record(15);
        tracker.record(20);

        assert!(tracker.meets_sla(20, 30));
        assert!(!tracker.meets_sla(10, 30));
        assert!(!tracker.meets_sla(20, 15));
    }

    #[test]
    fn latency_tracker_summary_includes_percentiles() {
        let mut tracker = LatencyTracker::new("test-select");
        tracker.record(5);
        tracker.record(10);

        let summary = tracker.summary();
        assert!(summary.contains("test-select"));
        assert!(summary.contains("median"));
        assert!(summary.contains("p95"));
        assert!(summary.contains("samples=2"));
    }

    #[test]
    fn scoped_latency_records_on_drop() {
        let mut tracker = LatencyTracker::new("scoped-test");
        {
            let _scoped = ScopedLatency::new(&mut tracker);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        assert_eq!(tracker.samples.len(), 1);
        assert!(
            tracker.samples[0] >= 4,
            "recorded: {}, expected ~5ms",
            tracker.samples[0]
        );
    }
}
