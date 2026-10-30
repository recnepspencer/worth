use worth_store::physical_runtime::ServingPhysicalRuntime;

fn invoke_removed_routes(runtime: &mut ServingPhysicalRuntime) {
    let _writer = runtime.records_mut();
    let _outcome = runtime.execute_scheduled_writeback(());
}

fn main() {}
