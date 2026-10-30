use worth_signal::facade::SignalRuntime;

fn try_legacy_capture(runtime: &mut SignalRuntime<(), (), (), (), ()>) {
    let _ = runtime.capture_snapshot();
    let _ = runtime.capture_branch_snapshot(runtime.current_branch());
}

fn main() {}
