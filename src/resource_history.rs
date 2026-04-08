use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Maximum number of samples to keep per process.
const MAX_SAMPLES: usize = 60;

/// Interval between resource samples.
pub const SAMPLE_INTERVAL: Duration = Duration::from_secs(1);

/// A single resource sample (CPU + memory).
#[derive(Debug, Clone, Copy)]
pub struct ResourceSample {
    pub cpu_percent: f32,
    pub memory_mb: f64,
    pub timestamp: Instant,
}

/// History of resource samples for a single process.
#[derive(Debug, Clone)]
pub struct ProcessHistory {
    pub pid: u32,
    pub samples: VecDeque<ResourceSample>,
    pub last_sample_time: Option<Instant>,
}

impl ProcessHistory {
    pub fn new(pid: u32) -> Self {
        Self {
            pid,
            samples: VecDeque::with_capacity(MAX_SAMPLES),
            last_sample_time: None,
        }
    }

    /// Add a new sample, evicting old ones if at capacity.
    pub fn push(&mut self, cpu_percent: f32, memory_mb: f64) {
        let now = Instant::now();
        self.samples.push_back(ResourceSample {
            cpu_percent,
            memory_mb,
            timestamp: now,
        });
        if self.samples.len() > MAX_SAMPLES {
            self.samples.pop_front();
        }
        self.last_sample_time = Some(now);
    }

    /// Get CPU values as a sparkline-ready slice.
    pub fn cpu_values(&self) -> Vec<u64> {
        self.samples.iter().map(|s| s.cpu_percent as u64).collect()
    }

    /// Get memory values as a sparkline-ready slice.
    pub fn memory_values(&self) -> Vec<u64> {
        self.samples.iter().map(|s| s.memory_mb as u64).collect()
    }

    /// Get the average CPU over the history.
    pub fn avg_cpu(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        self.samples.iter().map(|s| s.cpu_percent).sum::<f32>() / self.samples.len() as f32
    }

    /// Get the average memory over the history.
    pub fn avg_memory(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        self.samples.iter().map(|s| s.memory_mb).sum::<f64>() / self.samples.len() as f64
    }

    /// Get the peak CPU value.
    pub fn peak_cpu(&self) -> f32 {
        self.samples.iter().map(|s| s.cpu_percent).fold(0.0f32, f32::max)
    }

    /// Get the peak memory value.
    pub fn peak_memory(&self) -> f64 {
        self.samples.iter().map(|s| s.memory_mb).fold(0.0f64, f64::max)
    }

    /// Returns true if we should collect a new sample (based on interval).
    pub fn should_sample(&self) -> bool {
        match self.last_sample_time {
            None => true,
            Some(last) => last.elapsed() >= SAMPLE_INTERVAL,
        }
    }
}

/// Tracker for all process resource histories.
#[derive(Debug, Clone)]
pub struct ResourceTracker {
    histories: std::collections::HashMap<u32, ProcessHistory>,
}

impl ResourceTracker {
    pub fn new() -> Self {
        Self {
            histories: std::collections::HashMap::new(),
        }
    }

    /// Record a sample for a process. Creates history if needed.
    pub fn record(&mut self, pid: u32, cpu_percent: f32, memory_mb: f64) {
        let history = self.histories.entry(pid).or_insert_with(|| ProcessHistory::new(pid));
        history.push(cpu_percent, memory_mb);
    }

    /// Get history for a specific PID.
    pub fn get(&self, pid: u32) -> Option<&ProcessHistory> {
        self.histories.get(&pid)
    }

    /// Get mutable history for a specific PID.
    pub fn get_mut(&mut self, pid: u32) -> Option<&mut ProcessHistory> {
        self.histories.get_mut(&pid)
    }

    /// Remove histories for PIDs not in the active set.
    pub fn prune(&mut self, active_pids: &std::collections::HashSet<u32>) {
        self.histories.retain(|pid, _| active_pids.contains(pid));
    }

    /// Record samples for a batch of processes.
    pub fn record_batch(&mut self, entries: &[(u32, f32, f64)]) {
        for &(pid, cpu, mem) in entries {
            self.record(pid, cpu, mem);
        }
    }

    /// Get all tracked PIDs.
    pub fn pids(&self) -> Vec<u32> {
        self.histories.keys().copied().collect()
    }
}

impl Default for ResourceTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a simple text sparkline from values.
/// Uses block characters: ▁▂▃▄▅▆▇█
pub fn sparkline_text(values: &[u64], width: usize) -> String {
    if values.is_empty() {
        return " ".repeat(width);
    }

    let blocks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    
    // Sample or pad to desired width
    let sampled: Vec<u64> = if values.len() <= width {
        let mut v = values.to_vec();
        // Pad with zeros at the start
        while v.len() < width {
            v.insert(0, 0);
        }
        v
    } else {
        // Take evenly spaced samples
        let step = values.len() as f64 / width as f64;
        (0..width)
            .map(|i| values[(i as f64 * step) as usize])
            .collect()
    };

    let max = sampled.iter().copied().max().unwrap_or(1).max(1);

    sampled
        .iter()
        .map(|&v| {
            let idx = ((v as f64 / max as f64) * 7.0).round() as usize;
            blocks[idx.min(7)]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_history() {
        let mut history = ProcessHistory::new(1234);
        assert!(history.should_sample());
        
        history.push(10.0, 100.0);
        history.push(20.0, 150.0);
        history.push(15.0, 120.0);

        assert_eq!(history.cpu_values(), vec![10, 20, 15]);
        assert_eq!(history.memory_values(), vec![100, 150, 120]);
        assert!((history.avg_cpu() - 15.0).abs() < 0.1);
        assert!((history.avg_memory() - 123.33).abs() < 1.0);
        assert_eq!(history.peak_cpu(), 20.0);
        assert_eq!(history.peak_memory(), 150.0);
    }

    #[test]
    fn test_resource_tracker() {
        let mut tracker = ResourceTracker::new();
        tracker.record(1, 10.0, 100.0);
        tracker.record(2, 20.0, 200.0);
        tracker.record(1, 15.0, 110.0);

        let history1 = tracker.get(1).unwrap();
        assert_eq!(history1.samples.len(), 2);

        let history2 = tracker.get(2).unwrap();
        assert_eq!(history2.samples.len(), 1);
    }

    #[test]
    fn test_sparkline_text() {
        let values = vec![0, 10, 50, 100];
        let spark = sparkline_text(&values, 4);
        assert_eq!(spark.chars().count(), 4);
        
        // Empty case
        let empty = sparkline_text(&[], 4);
        assert_eq!(empty, "    ");
    }

    #[test]
    fn test_max_samples_eviction() {
        let mut history = ProcessHistory::new(1);
        for i in 0..70 {
            history.push(i as f32, i as f64);
        }
        assert_eq!(history.samples.len(), MAX_SAMPLES);
    }

    #[test]
    fn test_prune() {
        let mut tracker = ResourceTracker::new();
        tracker.record(1, 10.0, 100.0);
        tracker.record(2, 20.0, 200.0);
        tracker.record(3, 30.0, 300.0);

        let mut active = std::collections::HashSet::new();
        active.insert(1);
        active.insert(3);
        tracker.prune(&active);

        assert!(tracker.get(1).is_some());
        assert!(tracker.get(2).is_none());
        assert!(tracker.get(3).is_some());
    }
}