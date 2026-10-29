use worth_signal::facade::SignalRuntime;

fn try_legacy_transaction(runtime: &mut SignalRuntime<(), (), (), (), ()>) {
    let _ = runtime.transaction(&mut (), |_| Ok(()));
}

fn main() {}
