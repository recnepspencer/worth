use worth_store::physical_runtime::PhysicalStoreCloseTask;

fn clone_task(task: PhysicalStoreCloseTask) {
    let _ = task.clone();
}

fn construct_from_private_worker(task: PhysicalStoreCloseTask) {
    let _ = task.worker;
}

fn poll_twice(task: PhysicalStoreCloseTask) {
    let _ = task.poll();
    let _ = task.poll();
}

fn poll_borrowed(task: &PhysicalStoreCloseTask) {
    let _ = task.poll();
}

fn main() {}
