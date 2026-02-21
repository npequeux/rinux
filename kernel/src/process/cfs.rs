//! Completely Fair Scheduler (CFS-inspired)
//!
//! Linux CFS-inspired scheduler with virtual runtime tracking.

use super::task::{Task, TaskState, Priority};
use crate::types::Pid;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

/// Minimum granularity in nanoseconds (time slice won't go below this)
const MIN_GRANULARITY_NS: u64 = 1_000_000; // 1ms

/// Target latency in nanoseconds (how often each task should run)
const TARGET_LATENCY_NS: u64 = 6_000_000; // 6ms

/// Weight for nice level 0 (default priority)
const NICE_0_WEIGHT: u64 = 1024;

/// CFS task info
#[derive(Clone)]
pub struct CfsTask {
    /// Task information
    pub task: Task,
    /// Virtual runtime (in nanoseconds)
    pub vruntime: u64,
    /// Weight based on priority
    pub weight: u64,
    /// Time slice in nanoseconds
    pub time_slice: u64,
    /// CPU affinity mask
    pub cpu_affinity: u64,
}

impl CfsTask {
    /// Create a new CFS task
    pub fn new(task: Task) -> Self {
        let weight = priority_to_weight(task.priority);
        Self {
            task,
            vruntime: 0,
            weight,
            time_slice: 0,
            cpu_affinity: u64::MAX, // All CPUs by default
        }
    }

    /// Calculate time slice based on weight and number of tasks
    pub fn calculate_time_slice(&mut self, total_weight: u64, num_tasks: usize) {
        if num_tasks == 0 || total_weight == 0 {
            self.time_slice = MIN_GRANULARITY_NS;
            return;
        }

        // Time slice = (target_latency * weight) / total_weight
        let time_slice = (TARGET_LATENCY_NS * self.weight) / total_weight;
        self.time_slice = time_slice.max(MIN_GRANULARITY_NS);
    }
}

/// Convert priority (0-255) to weight
fn priority_to_weight(priority: Priority) -> u64 {
    // Map priority to nice value (-20 to +19)
    // Priority 0-39 => nice -20 to -1 (higher priority)
    // Priority 40-159 => nice 0 to +19 (normal to lower priority)
    // Priority 160-255 => nice +19 (lowest priority)
    
    let nice = if priority < 120 {
        -20i32 + (priority as i32 * 40 / 120)
    } else {
        (priority as i32 - 120) * 19 / 135
    };

    // Weight calculation (simplified)
    // Real CFS uses a more complex formula
    if nice <= 0 {
        NICE_0_WEIGHT * 2u64.pow(nice.unsigned_abs())
    } else {
        NICE_0_WEIGHT / 2u64.pow(nice as u32)
    }
}

/// CFS Run queue
pub struct CfsRunQueue {
    /// Tasks ordered by virtual runtime (red-black tree simulation with BTreeMap)
    tasks: BTreeMap<u64, CfsTask>,
    /// PID to vruntime mapping for quick lookup
    pid_to_vruntime: BTreeMap<Pid, u64>,
    /// Minimum virtual runtime (leftmost task)
    min_vruntime: u64,
    /// Total weight of all tasks
    total_weight: u64,
    /// Current running task
    current: Option<(Pid, u64)>, // (pid, vruntime_key)
}

impl CfsRunQueue {
    pub const fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            pid_to_vruntime: BTreeMap::new(),
            min_vruntime: 0,
            total_weight: 0,
            current: None,
        }
    }

    /// Enqueue a task
    pub fn enqueue(&mut self, mut cfs_task: CfsTask) {
        // Set vruntime to max(current vruntime, min_vruntime) for new tasks
        if cfs_task.vruntime < self.min_vruntime {
            cfs_task.vruntime = self.min_vruntime;
        }

        let pid = cfs_task.task.pid;
        let vruntime = cfs_task.vruntime;

        // Calculate time slice
        let num_tasks = self.tasks.len() + 1;
        let new_total_weight = self.total_weight + cfs_task.weight;
        cfs_task.calculate_time_slice(new_total_weight, num_tasks);

        // Update total weight
        self.total_weight += cfs_task.weight;

        // Insert into red-black tree (BTreeMap)
        self.tasks.insert(vruntime, cfs_task);
        self.pid_to_vruntime.insert(pid, vruntime);

        // Recalculate time slices for all tasks
        self.recalculate_time_slices();
    }

    /// Dequeue the leftmost (minimum vruntime) task
    pub fn dequeue_next(&mut self) -> Option<CfsTask> {
        if let Some((vruntime, task)) = self.tasks.iter().next() {
            let vruntime = *vruntime;
            let task = task.clone();
            
            self.tasks.remove(&vruntime);
            self.pid_to_vruntime.remove(&task.task.pid);
            self.total_weight = self.total_weight.saturating_sub(task.weight);

            // Update min_vruntime
            if let Some((new_min, _)) = self.tasks.iter().next() {
                self.min_vruntime = *new_min;
            } else {
                // Keep current min_vruntime if no tasks left
            }

            self.recalculate_time_slices();
            Some(task)
        } else {
            None
        }
    }

    /// Remove a specific task
    pub fn remove(&mut self, pid: Pid) {
        if let Some(vruntime) = self.pid_to_vruntime.remove(&pid) {
            if let Some(task) = self.tasks.remove(&vruntime) {
                self.total_weight = self.total_weight.saturating_sub(task.weight);
                self.recalculate_time_slices();
            }
        }

        if let Some((current_pid, _)) = self.current {
            if current_pid == pid {
                self.current = None;
            }
        }
    }

    /// Get task by PID
    pub fn get_task(&self, pid: Pid) -> Option<&CfsTask> {
        self.pid_to_vruntime
            .get(&pid)
            .and_then(|vruntime| self.tasks.get(vruntime))
    }

    /// Update a task's vruntime after execution
    pub fn update_vruntime(&mut self, pid: Pid, runtime_ns: u64) {
        if let Some(old_vruntime) = self.pid_to_vruntime.get(&pid).copied() {
            if let Some(mut task) = self.tasks.remove(&old_vruntime) {
                // Calculate new vruntime based on weight
                // vruntime_delta = runtime * NICE_0_WEIGHT / weight
                let vruntime_delta = (runtime_ns * NICE_0_WEIGHT) / task.weight;
                task.vruntime += vruntime_delta;

                // Update min_vruntime
                if task.vruntime > self.min_vruntime {
                    self.min_vruntime = task.vruntime;
                }

                let new_vruntime = task.vruntime;
                self.tasks.insert(new_vruntime, task);
                self.pid_to_vruntime.insert(pid, new_vruntime);
            }
        }
    }

    /// Recalculate time slices for all tasks
    fn recalculate_time_slices(&mut self) {
        let total_weight = self.total_weight;
        let num_tasks = self.tasks.len();

        for task in self.tasks.values_mut() {
            task.calculate_time_slice(total_weight, num_tasks);
        }
    }

    /// Get minimum vruntime
    pub fn min_vruntime(&self) -> u64 {
        self.min_vruntime
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Get number of tasks
    pub fn len(&self) -> usize {
        self.tasks.len()
    }
}

/// Global CFS scheduler
static CFS_SCHEDULER: Mutex<CfsRunQueue> = Mutex::new(CfsRunQueue::new());

/// Current task runtime counter (nanoseconds)
static CURRENT_RUNTIME_NS: AtomicU64 = AtomicU64::new(0);

/// Scheduler initialization flag
static CFS_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize CFS scheduler
pub fn init() {
    if CFS_INITIALIZED.load(Ordering::Acquire) {
        return;
    }

    let mut queue = CFS_SCHEDULER.lock();

    // Create idle task (PID 0)
    let idle_task = Task::new(0);
    let cfs_task = CfsTask::new(idle_task);
    queue.enqueue(cfs_task);

    CFS_INITIALIZED.store(true, Ordering::Release);

    crate::printk::printk("  CFS scheduler initialized\n");
}

/// Schedule next task
pub fn schedule() -> Option<Pid> {
    let mut queue = CFS_SCHEDULER.lock();

    // Update current task's vruntime if it ran
    if let Some((current_pid, _)) = queue.current {
        let runtime_ns = CURRENT_RUNTIME_NS.swap(0, Ordering::Relaxed);
        if runtime_ns > 0 {
            queue.update_vruntime(current_pid, runtime_ns);
        }
    }

    // Pick next task
    if let Some(next_task) = queue.dequeue_next() {
        let next_pid = next_task.task.pid;
        queue.current = Some((next_pid, next_task.vruntime));
        
        // Re-enqueue for next scheduling
        queue.enqueue(next_task);

        Some(next_pid)
    } else {
        queue.current = None;
        None
    }
}

/// Add runtime to current task
pub fn add_runtime(runtime_ns: u64) {
    CURRENT_RUNTIME_NS.fetch_add(runtime_ns, Ordering::Relaxed);
}

/// Add task to CFS scheduler
pub fn add_task(task: Task) {
    let mut queue = CFS_SCHEDULER.lock();
    let cfs_task = CfsTask::new(task);
    queue.enqueue(cfs_task);
}

/// Remove task from CFS scheduler
pub fn remove_task(pid: Pid) {
    let mut queue = CFS_SCHEDULER.lock();
    queue.remove(pid);
}

/// Get current task PID
pub fn current_pid() -> Option<Pid> {
    let queue = CFS_SCHEDULER.lock();
    queue.current.map(|(pid, _)| pid)
}

/// Set CPU affinity for a task
pub fn set_cpu_affinity(pid: Pid, cpu_mask: u64) -> Result<(), &'static str> {
    let mut queue = CFS_SCHEDULER.lock();
    
    if let Some(vruntime) = queue.pid_to_vruntime.get(&pid).copied() {
        if let Some(task) = queue.tasks.get_mut(&vruntime) {
            task.cpu_affinity = cpu_mask;
            return Ok(());
        }
    }
    
    Err("Task not found")
}

/// Check if task should be preempted
pub fn should_preempt() -> bool {
    let queue = CFS_SCHEDULER.lock();
    
    if let Some((current_pid, current_vruntime)) = queue.current {
        // Check if current task has exceeded its time slice
        let runtime_ns = CURRENT_RUNTIME_NS.load(Ordering::Relaxed);
        
        if let Some(current_task) = queue.tasks.get(&current_vruntime) {
            if runtime_ns >= current_task.time_slice {
                return true;
            }

            // Check if there's a task with significantly lower vruntime
            if let Some((min_vruntime, _)) = queue.tasks.iter().next() {
                let vruntime_diff = current_vruntime.saturating_sub(*min_vruntime);
                // Preempt if difference is more than target latency
                if vruntime_diff > TARGET_LATENCY_NS {
                    return true;
                }
            }
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_to_weight() {
        let weight_0 = priority_to_weight(0);
        let weight_120 = priority_to_weight(120);
        let weight_255 = priority_to_weight(255);
        
        assert!(weight_0 > weight_120);
        assert!(weight_120 > weight_255);
    }

    #[test]
    fn test_cfs_task_creation() {
        let task = Task::new(1);
        let cfs_task = CfsTask::new(task);
        
        assert_eq!(cfs_task.task.pid, 1);
        assert_eq!(cfs_task.vruntime, 0);
        assert!(cfs_task.weight > 0);
    }

    #[test]
    fn test_cfs_runqueue_enqueue_dequeue() {
        let mut queue = CfsRunQueue::new();
        
        let task1 = Task::new(1);
        let cfs_task1 = CfsTask::new(task1);
        queue.enqueue(cfs_task1);
        
        assert_eq!(queue.len(), 1);
        
        let dequeued = queue.dequeue_next();
        assert!(dequeued.is_some());
        assert_eq!(dequeued.unwrap().task.pid, 1);
        assert_eq!(queue.len(), 0);
    }
}
