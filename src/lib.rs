/*!
# cuda-temporal

Time-aware reasoning for agents.

Agents don't just act — they act IN TIME. Tasks have deadlines.
Events have causes. Schedules have conflicts. The future has uncertainty.

This crate gives agents temporal intelligence:
- Time intervals with overlap/containment/adjacency
- Causal chains (A caused B caused C)
- Deadline management with urgency escalation
- Temporal scheduling with conflict detection
- Future prediction with confidence decay
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A time interval [start, end]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Interval {
    pub start: u64,  // ms since epoch
    pub end: u64,
}

impl Interval {
    pub fn new(start: u64, end: u64) -> Self { Interval { start, end } }
    pub fn duration(&self) -> u64 { self.end.saturating_sub(self.start) }
    pub fn contains_time(&self, t: u64) -> bool { t >= self.start && t <= self.end }
    pub fn contains(&self, other: &Interval) -> bool { other.start >= self.start && other.end <= self.end }
    pub fn overlaps(&self, other: &Interval) -> bool { self.start < other.end && other.start < self.end }
    pub fn before(&self, other: &Interval) -> bool { self.end <= other.start }
    pub fn after(&self, other: &Interval) -> bool { self.start >= other.end }
    pub fn gap(&self, other: &Interval) -> Option<u64> {
        if self.before(other) { Some(other.start.saturating_sub(self.end)) }
        else if self.after(other) { Some(self.start.saturating_sub(other.end)) }
        else { None }
    }
    pub fn merge(&self, other: &Interval) -> Interval { Interval { start: self.start.min(other.start), end: self.end.max(other.end) } }
}

/// A temporal event with cause tracking
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalEvent {
    pub id: u64,
    pub event_type: String,
    pub time: u64,
    pub confidence: f64,
    pub caused_by: Option<u64>,  // parent event
    pub effects: Vec<u64>,       // child events
    pub tags: Vec<String>,
}

/// Causal chain — linked events
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CausalChain {
    pub events: HashMap<u64, TemporalEvent>,
    pub roots: Vec<u64>,        // events with no cause
}

impl CausalChain {
    pub fn new() -> Self { CausalChain { events: HashMap::new(), roots: vec![] } }

    pub fn add_event(&mut self, event: TemporalEvent) {
        if let Some(cause) = event.caused_by {
            if let Some(parent) = self.events.get_mut(&cause) {
                parent.effects.push(event.id);
            }
        } else {
            self.roots.push(event.id);
        }
        self.events.insert(event.id, event);
    }

    /// Get causal ancestors of an event
    pub fn ancestors(&self, event_id: u64) -> Vec<&TemporalEvent> {
        let mut chain = vec![];
        let mut current = event_id;
        while let Some(event) = self.events.get(&current) {
            chain.push(event);
            match event.caused_by {
                Some(parent) => current = parent,
                None => break,
            }
        }
        chain
    }

    /// Get all descendants
    pub fn descendants(&self, event_id: u64) -> Vec<&TemporalEvent> {
        let mut result = vec![];
        let mut queue = vec![event_id];
        let mut visited = std::collections::HashSet::new();
        while let Some(id) = queue.pop() {
            if visited.contains(&id) { continue; }
            visited.insert(id);
            if let Some(event) = self.events.get(&id) {
                for &child in &event.effects {
                    queue.push(child);
                }
                if id != event_id { result.push(event); }
            }
        }
        result
    }

    /// Verify causality: causes must precede effects in time
    pub fn verify_causality(&self) -> Vec<String> {
        let mut violations = vec![];
        for event in self.events.values() {
            if let Some(cause_id) = event.caused_by {
                if let Some(cause) = self.events.get(&cause_id) {
                    if cause.time >= event.time {
                        violations.push(format!("Event {} caused by {} but time {} >= {}", event.id, cause_id, cause.time, event.time));
                    }
                }
            }
        }
        violations
    }

    /// Depth of causal chain (longest root-to-leaf path)
    pub fn max_depth(&self) -> usize {
        fn depth(events: &HashMap<u64, TemporalEvent>, id: u64, cache: &mut HashMap<u64, usize>) -> usize {
            if let Some(&d) = cache.get(&id) { return d; }
            let event = match events.get(&id) { Some(e) => e, None => return 0 };
            let d = match event.caused_by {
                Some(parent) => 1 + depth(events, parent, cache),
                None => 1,
            };
            cache.insert(id, d);
            d
        }
        let mut cache = HashMap::new();
        self.events.keys().map(|&id| depth(&self.events, id, &mut cache)).max().unwrap_or(0)
    }
}

/// A scheduled task with deadline
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: u64,
    pub name: String,
    pub interval: Interval,
    pub priority: f64,      // 0-1
    pub deadline: Option<u64>,
    pub dependencies: Vec<u64>, // must complete first
    pub completed: bool,
    pub confidence: f64,
}

/// Temporal scheduler
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalScheduler {
    pub tasks: HashMap<u64, ScheduledTask>,
    pub schedule: Vec<Vec<u64>>,  // time slots -> task ids
}

impl TemporalScheduler {
    pub fn new() -> Self { TemporalScheduler { tasks: HashMap::new(), schedule: vec![] } }

    pub fn add_task(&mut self, task: ScheduledTask) {
        self.tasks.insert(task.id, task);
    }

    /// Get urgency score for a task (0 = no rush, 1 = critical)
    pub fn urgency(&self, task_id: u64, now: u64) -> f64 {
        let task = match self.tasks.get(&task_id) { Some(t) => t, None => return 0.0 };
        if task.completed { return 0.0; }

        let deadline_urgency = match task.deadline {
            Some(dl) => {
                let remaining = dl.saturating_sub(now) as f64;
                let total = task.interval.duration() as f64;
                if total < 1.0 { 0.0 } else { (1.0 - remaining / total).clamp(0.0, 1.0) }
            }
            None => 0.0,
        };

        let priority_urgency = task.priority;
        let deadline_urgency * 0.6 + priority_urgency * 0.4
    }

    /// Find scheduling conflicts (overlapping intervals)
    pub fn conflicts(&self) -> Vec<(u64, u64)> {
        let task_list: Vec<_> = self.tasks.values().filter(|t| !t.completed).collect();
        let mut conflicts = vec![];
        for i in 0..task_list.len() {
            for j in (i+1)..task_list.len() {
                if task_list[i].interval.overlaps(&task_list[j].interval) {
                    conflicts.push((task_list[i].id, task_list[j].id));
                }
            }
        }
        conflicts
    }

    /// Get tasks sorted by urgency
    pub fn by_urgency(&self, now: u64) -> Vec<(u64, f64)> {
        let mut scored: Vec<_> = self.tasks.keys()
            .filter(|&&id| !self.tasks[&id].completed)
            .map(|&id| (id, self.urgency(id, now)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored
    }

    /// Check if all dependencies are completed
    pub fn dependencies_met(&self, task_id: u64) -> bool {
        let task = match self.tasks.get(&task_id) { Some(t) => t, None => return false };
        task.dependencies.iter().all(|&dep| self.tasks.get(&dep).map_or(false, |d| d.completed))
    }

    /// Next executable task (highest urgency, deps met)
    pub fn next_task(&self, now: u64) -> Option<u64> {
        self.by_urgency(now).into_iter()
            .find(|&(id, _)| self.dependencies_met(id))
            .map(|(id, _)| id)
    }
}

/// Future prediction with confidence decay
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FuturePrediction {
    pub predicted_time: u64,
    pub predicted_value: f64,
    pub confidence: f64,
    pub horizon: u64, // how far in the future
}

/// Temporal reasoner
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalReasoner {
    pub now: u64,
}

impl TemporalReasoner {
    pub fn new(now: u64) -> Self { TemporalReasoner { now } }

    /// Predict with exponential confidence decay over time
    pub fn predict(&self, value: f64, confidence: f64, horizon_ms: u64, half_life_ms: u64) -> FuturePrediction {
        let decay = 0.5_f64.powf(horizon_ms as f64 / half_life_ms as f64);
        FuturePrediction { predicted_time: self.now + horizon_ms, predicted_value: value, confidence: confidence * decay, horizon: horizon_ms }
    }

    /// Is a deadline approaching? (within threshold)
    pub fn deadline_approaching(&self, deadline: u64, threshold_ms: u64) -> bool {
        deadline.saturating_sub(self.now) <= threshold_ms && deadline > self.now
    }

    /// Time remaining until deadline
    pub fn time_remaining(&self, deadline: u64) -> u64 {
        deadline.saturating_sub(self.now)
    }
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_overlap() {
        let a = Interval::new(0, 10);
        let b = Interval::new(5, 15);
        assert!(a.overlaps(&b));
    }

    #[test]
    fn test_interval_no_overlap() {
        let a = Interval::new(0, 5);
        let b = Interval::new(10, 15);
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn test_interval_contains() {
        let outer = Interval::new(0, 20);
        let inner = Interval::new(5, 15);
        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn test_interval_gap() {
        let a = Interval::new(0, 5);
        let b = Interval::new(10, 15);
        assert_eq!(a.gap(&b), Some(5));
    }

    #[test]
    fn test_interval_merge() {
        let a = Interval::new(0, 10);
        let b = Interval::new(5, 15);
        let merged = a.merge(&b);
        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 15);
    }

    #[test]
    fn test_causal_chain() {
        let mut chain = CausalChain::new();
        chain.add_event(TemporalEvent { id: 1, event_type: "start".into(), time: 0, confidence: 1.0, caused_by: None, effects: vec![], tags: vec![] });
        chain.add_event(TemporalEvent { id: 2, event_type: "middle".into(), time: 5, confidence: 0.9, caused_by: Some(1), effects: vec![], tags: vec![] });
        chain.add_event(TemporalEvent { id: 3, event_type: "end".into(), time: 10, confidence: 0.8, caused_by: Some(2), effects: vec![], tags: vec![] });
        assert_eq!(chain.ancestors(3).len(), 3);
        assert_eq!(chain.descendants(1).len(), 2);
        assert_eq!(chain.max_depth(), 3);
    }

    #[test]
    fn test_causality_violation() {
        let mut chain = CausalChain::new();
        chain.add_event(TemporalEvent { id: 1, event_type: "cause".into(), time: 100, confidence: 1.0, caused_by: None, effects: vec![], tags: vec![] });
        chain.add_event(TemporalEvent { id: 2, event_type: "effect".into(), time: 50, confidence: 1.0, caused_by: Some(1), effects: vec![], tags: vec![] });
        let violations = chain.verify_causality();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_scheduler_urgency() {
        let mut sched = TemporalScheduler::new();
        sched.add_task(ScheduledTask { id: 1, name: "urgent".into(), interval: Interval::new(0, 1000), priority: 0.9, deadline: Some(500), dependencies: vec![], completed: false, confidence: 1.0 });
        sched.add_task(ScheduledTask { id: 2, name: "chill".into(), interval: Interval::new(0, 10000), priority: 0.1, deadline: Some(9000), dependencies: vec![], completed: false, confidence: 1.0 });
        let u1 = sched.urgency(1, 400);
        let u2 = sched.urgency(2, 400);
        assert!(u1 > u2);
    }

    #[test]
    fn test_scheduler_conflicts() {
        let mut sched = TemporalScheduler::new();
        sched.add_task(ScheduledTask { id: 1, name: "a".into(), interval: Interval::new(0, 10), priority: 0.5, deadline: None, dependencies: vec![], completed: false, confidence: 1.0 });
        sched.add_task(ScheduledTask { id: 2, name: "b".into(), interval: Interval::new(5, 15), priority: 0.5, deadline: None, dependencies: vec![], completed: false, confidence: 1.0 });
        let conflicts = sched.conflicts();
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn test_next_task_deps() {
        let mut sched = TemporalScheduler::new();
        sched.add_task(ScheduledTask { id: 1, name: "first".into(), interval: Interval::new(0, 100), priority: 0.5, deadline: None, dependencies: vec![], completed: false, confidence: 1.0 });
        sched.add_task(ScheduledTask { id: 2, name: "second".into(), interval: Interval::new(100, 200), priority: 0.8, deadline: None, dependencies: vec![1], completed: false, confidence: 1.0 });
        let next = sched.next_task(0);
        assert_eq!(next, Some(1)); // deps not met for 2
    }

    #[test]
    fn test_predict_decay() {
        let tr = TemporalReasoner::new(0);
        let pred = tr.predict(42.0, 1.0, 1000, 1000);
        assert_eq!(pred.confidence, 0.5); // one half-life
    }

    #[test]
    fn test_deadline_approaching() {
        let tr = TemporalReasoner::new(900);
        assert!(tr.deadline_approaching(1000, 200));
        assert!(!tr.deadline_approaching(2000, 200));
    }

    #[test]
    fn test_time_remaining() {
        let tr = TemporalReasoner::new(100);
        assert_eq!(tr.time_remaining(500), 400);
        assert_eq!(tr.time_remaining(50), 0); // past
    }

    #[test]
    fn test_completed_task_no_urgency() {
        let mut sched = TemporalScheduler::new();
        sched.add_task(ScheduledTask { id: 1, name: "done".into(), interval: Interval::new(0, 100), priority: 1.0, deadline: Some(50), dependencies: vec![], completed: true, confidence: 1.0 });
        assert_eq!(sched.urgency(1, 100), 0.0);
    }
}
