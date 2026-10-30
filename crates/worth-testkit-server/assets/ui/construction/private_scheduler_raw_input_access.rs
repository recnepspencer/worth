use worth_server::{WorthServerOperationScheduler, WorthServerQueryHandoffInput};

fn impossible(scheduler: WorthServerOperationScheduler, input: WorthServerQueryHandoffInput) {
    let _ = scheduler.schedule_shared_read_batch([input]);
}

fn main() {}
