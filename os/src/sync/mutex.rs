use super::UPSafeCell;
use crate::task::TaskControlBlock;
use crate::task::{block_current_and_run_next, suspend_current_and_run_next};
use crate::task::{current_task, wakeup_task};
use alloc::{collections::VecDeque, sync::Arc};

pub trait Mutex: Sync + Send {
    fn lock(&self);
    fn unlock(&self);
    fn is_locked(&self) -> bool;
    fn update_task_mutex_info(&self);
}

pub struct MutexSpin {
    locked: UPSafeCell<bool>,
    id: usize,
}

impl MutexSpin {
    pub fn new(id_: usize) -> Self {
        Self {
            locked: unsafe { UPSafeCell::new(false) },
            id: id_,
        }
    }
}

impl Mutex for MutexSpin {
    fn lock(&self) {
        loop {
            let mut locked = self.locked.exclusive_access();
            if *locked {
                drop(locked);
                suspend_current_and_run_next();
                continue;
            } else {
                let current_task = current_task().unwrap();
                let mut current_task_inner = current_task.inner_exclusive_access();
                current_task_inner.mutex_allocation[self.id] = 1;
                current_task_inner.mutex_need[self.id] = 0;
                drop(current_task_inner);
                *locked = true;
                return;
            }
        }
    }

    fn unlock(&self) {
        let mut locked = self.locked.exclusive_access();
        *locked = false;
    }

    fn is_locked(&self) -> bool {
        let locked = self.locked.exclusive_access();
        *locked
    }

    fn update_task_mutex_info(&self) {
        let locked = self.locked.exclusive_access();
        let current_task = current_task().unwrap();
        if *locked {
            current_task.inner_exclusive_access().mutex_need[self.id] = 1;
        } else {
            current_task.inner_exclusive_access().mutex_need[self.id] = 0;
            current_task.inner_exclusive_access().mutex_allocation[self.id] = 1;
        }
    }
}

pub struct MutexBlocking {
    inner: UPSafeCell<MutexBlockingInner>,
    id: usize,
}

pub struct MutexBlockingInner {
    locked: bool,
    wait_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl MutexBlocking {
    pub fn new(id_: usize) -> Self {
        Self {
            inner: unsafe {
                UPSafeCell::new(MutexBlockingInner {
                    locked: false,
                    wait_queue: VecDeque::new(),
                })
            },
            id: id_,
        }
    }
}

impl Mutex for MutexBlocking {
    fn lock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        if mutex_inner.locked {
            mutex_inner.wait_queue.push_back(current_task().unwrap());
            drop(mutex_inner);
            block_current_and_run_next();
        } else {
            mutex_inner.locked = true;
        }
    }

    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            wakeup_task(waking_task);
        } else {
            mutex_inner.locked = false;
        }
    }

    fn is_locked(&self) -> bool {
        self.inner.exclusive_access().locked
    }

    fn update_task_mutex_info(&self) {
        let mutex_inner = self.inner.exclusive_access();
        let current_task = current_task().unwrap();
        if mutex_inner.locked {
            current_task.inner_exclusive_access().mutex_need[self.id] = 1;
        } else {
            current_task.inner_exclusive_access().mutex_allocation[self.id] = 1;
        }
    }
}
