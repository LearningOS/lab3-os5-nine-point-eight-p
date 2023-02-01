//! Implementation of [`TaskManager`]
//!
//! It is only used to manage processes and schedule process based on ready queue.
//! Other CPU process monitoring functions are in Processor.


use core::cmp::Ordering;

use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::{VecDeque, BinaryHeap};
use alloc::sync::Arc;
use lazy_static::*;

pub trait TaskManager {
    fn new() -> Self;
    /// Add process back to ready queue
    fn add(&mut self, task: Arc<TaskControlBlock>);
    /// Take a process out of the ready queue
    fn fetch(&mut self) -> Option<Arc<TaskControlBlock>>;
}

pub struct SimpleManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

// YOUR JOB: FIFO->Stride
/// A simple FIFO scheduler.
impl TaskManager for SimpleManager {
    fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }
}

/// Task manager based on stride algorithm.
pub struct StrideManager {
    /// A priority queue based on min heap, since StrideManagerBlock has a custom
    /// implementation for ord.
    queue: BinaryHeap<StrideManagerBlock>,
}

pub const BIG_STRIDE: usize = 100;
pub const PRIORITY_INIT: usize = 16;
pub const PASS_INIT: usize = 0;

impl TaskManager for StrideManager {
    fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
        }
    }
    fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.queue.push(task.into());
    }
    fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.queue.pop().map(|wrapper| {
            let task = wrapper.task;
            let mut inner = task.inner_exclusive_access();
            inner.pass += BIG_STRIDE / inner.priority;
            drop(inner);
            task
        })
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<StrideManager> =
        unsafe { UPSafeCell::new(StrideManager::new()) };
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}

/// Simple wrapper for TaskControlBlock that can be ordered.
/// Blocks with low pass values will have a higher priority,
/// so in comparison they seem "bigger"(>) than those with high pass values.
struct StrideManagerBlock {
    pub task: Arc<TaskControlBlock>,
    pub pass: usize,
}

impl PartialEq for StrideManagerBlock {
    fn eq(&self, other: &Self) -> bool {
        self.pass == other.pass
    }
}

impl Eq for StrideManagerBlock {}

impl PartialOrd for StrideManagerBlock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StrideManagerBlock {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min heap
        match self.pass.cmp(&other.pass) {
            Ordering::Less => Ordering::Greater,
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Less,
        }
    }
}

impl From<Arc<TaskControlBlock>> for StrideManagerBlock {
    fn from(value: Arc<TaskControlBlock>) -> Self {
        let pass = value.inner_exclusive_access().pass;
        Self {
            task: value,
            pass,
        }
    }
}