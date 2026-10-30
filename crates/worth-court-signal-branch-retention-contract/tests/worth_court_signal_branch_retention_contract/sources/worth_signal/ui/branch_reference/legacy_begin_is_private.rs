use worth_signal::facade::SignalRuntime;

fn try_legacy_begin(runtime: &mut SignalRuntime<(), (), (), (), ()>) {
    let _ = runtime.begin(&mut ());
}

fn main() {}
