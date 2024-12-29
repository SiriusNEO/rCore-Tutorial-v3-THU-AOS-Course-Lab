use crate::sync::UPSafeCell;
use crate::task::{
    block_current_and_run_next, current_process, current_task, wakeup_task, TaskControlBlock,
};
use alloc::{collections::VecDeque, sync::Arc};

pub struct Semaphore {
    pub inner: UPSafeCell<SemaphoreInner>,
    pub id: usize,
}

pub struct SemaphoreInner {
    pub count: isize,
    pub wait_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl Semaphore {
    pub fn new(res_count: usize, id_: usize) -> Self {
        Self {
            inner: unsafe {
                UPSafeCell::new(SemaphoreInner {
                    count: res_count as isize,
                    wait_queue: VecDeque::new(),
                })
            },
            id: id_,
        }
    }

    pub fn up(&self) {
        let mut inner = self.inner.exclusive_access();
        let process = current_process();
        let process_inner = process.inner_exclusive_access();
        inner.count += 1;
        if inner.count <= 0 {
            if let Some(mut task) = inner.wait_queue.pop_front() {
                if process_inner.enable_deadlock_detect {
                    self.update_task_sem_info_up(&mut task);
                }
                wakeup_task(task);
            }
        }
    }

    pub fn down(&self) {
        let mut inner = self.inner.exclusive_access();
        inner.count -= 1;
        if inner.count < 0 {
            inner.wait_queue.push_back(current_task().unwrap());
            drop(inner);
            block_current_and_run_next();
        }
    }

    pub fn update_task_sem_info_up(&self, task: &mut Arc<TaskControlBlock>) {
        task.inner_exclusive_access().sem_allocation[self.id] += 1;
    }

    pub fn update_task_sem_info_down(&self) {
        let inner = self.inner.exclusive_access();
        let current_task = current_task().unwrap();

        if inner.count > 0 {
            current_task.inner_exclusive_access().sem_allocation[self.id] += 1;
        } else {
            current_task.inner_exclusive_access().sem_need[self.id] += 1;
        }
    }
}
