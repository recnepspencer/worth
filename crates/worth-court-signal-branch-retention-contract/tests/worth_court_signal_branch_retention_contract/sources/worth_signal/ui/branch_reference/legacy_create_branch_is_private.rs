use worth_signal::facade::SignalRuntime;

fn try_legacy_create(runtime: &mut SignalRuntime<(), (), (), (), ()>) {
    let _ = runtime.create_branch("legacy");
}

fn main() {}
