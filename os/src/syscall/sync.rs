use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
use alloc::vec::Vec;

pub fn sys_sleep(ms: usize) -> isize {
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}

pub fn sys_mutex_create(blocking: bool) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        let mutex: Option<Arc<dyn Mutex>> = if !blocking {
            Some(Arc::new(MutexSpin::new(id)))
        } else {
            Some(Arc::new(MutexBlocking::new(id)))
        };
        process_inner.mutex_list[id] = mutex;
        if process_inner.enable_deadlock_detect {
            let task_count = process_inner.tasks.len();
            for i in 0..task_count {
                let task = process_inner.get_task(i);
                let mut task_inner = task.inner_exclusive_access();
                task_inner.mutex_allocation[id] = 0;
                task_inner.mutex_need[id] = 0;
            }
        }
        id as isize
    } else {
        let new_id = process_inner.mutex_list.len();
        let mutex: Option<Arc<dyn Mutex>> = if !blocking {
            Some(Arc::new(MutexSpin::new(new_id)))
        } else {
            Some(Arc::new(MutexBlocking::new(new_id)))
        };
        process_inner.mutex_list.push(mutex);
        if process_inner.enable_deadlock_detect {
            let task_count = process_inner.tasks.len();
            for i in 0..task_count {
                let task = process_inner.get_task(i);
                let mut task_inner = task.inner_exclusive_access();
                task_inner.mutex_allocation.push(0);
                task_inner.mutex_need.push(0);
            }
        }
        new_id as isize
    }
}

pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    let enable_deadlock_detect = process_inner.enable_deadlock_detect;

    if enable_deadlock_detect {
        mutex.update_task_mutex_info();
    }

    drop(process_inner);
    drop(process);

    if enable_deadlock_detect && detect_deadlock_for_mutex() {
        // return -0xDEAD, as spec
        return -0xDEAD;
    }
    mutex.lock();
    0
}

pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}

pub fn sys_semaphore_create(res_count: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count, id)));
        if process_inner.enable_deadlock_detect {
            let task_count = process_inner.tasks.len();
            for i in 0..task_count {
                let task = process_inner.get_task(i);
                let mut task_inner = task.inner_exclusive_access();
                task_inner.sem_allocation[id] = 0;
                task_inner.sem_need[id] = 0;
            }
        }
        id as isize
    } else {
        let new_id = process_inner.semaphore_list.len();
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count, new_id))));
        if process_inner.enable_deadlock_detect {
            let task_count = process_inner.tasks.len();
            for i in 0..task_count {
                let task = process_inner.get_task(i);
                let mut task_inner = task.inner_exclusive_access();
                task_inner.sem_allocation.push(0);
                task_inner.sem_need.push(0);
            }
        }
        new_id as isize
    };
    id as isize
}

pub fn sys_semaphore_up(sem_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    let enable_deadlock_detect = process_inner.enable_deadlock_detect;

    drop(process_inner);
    sem.up();

    if enable_deadlock_detect && detect_deadlock_for_sem() {
        // return -0xDEAD, as spec
        return -0xDEAD;
    }

    0
}

pub fn sys_semaphore_down(sem_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    let enable_deadlock_detect = process_inner.enable_deadlock_detect;

    if enable_deadlock_detect {
        sem.update_task_sem_info_down();
    }

    drop(process_inner);
    if enable_deadlock_detect && detect_deadlock_for_sem() {
        // return -0xDEAD, as spec
        return -0xDEAD;
    }
    sem.down();
    0
}

pub fn sys_condvar_create() -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}

pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}

pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}

pub fn sys_enable_deadlock_detect(enabled: usize) -> isize {
    if enabled == 1 {
        let process = current_process();
        let mut process_inner = process.inner_exclusive_access();
        process_inner.enable_deadlock_detect = true;
        return 1; // enable deadlock detection
    }
    0 // disable deadlock detection
}

// Basically this algorithm is similar for mutex and semaphores
// Since mutex can be viewed as a semaphore with N=1

fn detect_deadlock_for_mutex() -> bool {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let task_count = process_inner.tasks.len();

    // Algorithm:
    // First we have two Vectors: work / finish
    // Allocation matrix: allocation[i, j]
    // Need matrix: need[i, j]

    let mut work: Vec<usize> = Vec::new();
    let mut finish: Vec<bool> = Vec::new();

    // Init: finish all false, work: sem count
    for _i in 0..task_count {
        finish.push(false);
    }
    for i in 0..process_inner.mutex_list.len() {
        if let Some(mtx) = &mut process_inner.mutex_list[i] {
            if !mtx.is_locked() {
                work.push(1);
                continue;
            }
        }
        work.push(0);
    }

    // Repeatedly update, until no task is updated
    let mut updated = true;
    while updated {
        let tasks = &mut process_inner.tasks;
        // Find the first thread with finish[i]==false and work[j] >= need[i, j]
        updated = false;

        for i in 0..task_count {
            if finish[i] {
                continue;
            }

            if let Some(task) = &mut tasks[i] {
                let mut satisfied = true;
                let task_inner = task.inner_exclusive_access();
                // For all j
                for j in 0..work.len() {
                    if task_inner.mutex_need[j] > work[j] {
                        satisfied = false;
                        break;
                    }
                }

                if satisfied {
                    // update work[j], set finish[i] to true
                    for j in 0..work.len() {
                        work[j] += task_inner.mutex_allocation[j];
                    }

                    finish[i] = true;
                    updated = true;
                }

                drop(task_inner);
            }
        }
    }

    for i in 0..task_count {
        if !finish[i] {
            return true; // has unfinished job: deadlock
        }
    }
    false
}

pub fn detect_deadlock_for_sem() -> bool {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let task_count = process_inner.tasks.len();

    // Algorithm:
    // First we have two Vectors: work / finish
    // Allocation matrix: allocation[i, j]
    // Need matrix: need[i, j]

    let mut work: Vec<usize> = Vec::new();
    let mut finish: Vec<bool> = Vec::new();

    // Init: finish all false, work: sem count
    for _i in 0..task_count {
        finish.push(false);
    }
    for i in 0..process_inner.semaphore_list.len() {
        if let Some(sem) = &mut process_inner.semaphore_list[i] {
            let sem_inner = sem.inner.exclusive_access();
            if sem_inner.count > 0 {
                work.push(sem_inner.count as usize);
                continue;
            }
        }
        work.push(0);
    }

    // Repeatedly update, until no task is updated
    let mut updated = true;
    while updated {
        let tasks = &mut process_inner.tasks;
        // Find the first thread with finish[i]==false and work[j] >= need[i, j]
        updated = false;

        for i in 0..task_count {
            if finish[i] {
                continue;
            }

            if let Some(task) = &mut tasks[i] {
                let mut satisfied = true;
                let task_inner = task.inner_exclusive_access();
                // For all j
                for j in 0..work.len() {
                    if task_inner.sem_need[j] > work[j] {
                        satisfied = false;
                        break;
                    }
                }

                if satisfied {
                    // update work[j], set finish[i] to true
                    for j in 0..work.len() {
                        work[j] += task_inner.sem_allocation[j];
                    }

                    finish[i] = true;
                    updated = true;
                }

                drop(task_inner);
            }
        }
    }

    for i in 0..task_count {
        if !finish[i] {
            return true; // has unfinished job: deadlock
        }
    }
    false
}
