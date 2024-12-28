//! Types related to task management

use super::TaskContext;
use crate::config::MAX_SYSCALL_NUM;

#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub start_time: usize,
    pub syscall_times: [usize; MAX_SYSCALL_NUM],
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
