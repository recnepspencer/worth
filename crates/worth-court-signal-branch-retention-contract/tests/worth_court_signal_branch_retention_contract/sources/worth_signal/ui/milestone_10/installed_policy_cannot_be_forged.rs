use worth_signal::facade::{InstalledSignalRuntimePolicy, SignalRuntimePolicy};

fn main() {
    let _ = InstalledSignalRuntimePolicy {
        requested_policy: SignalRuntimePolicy::operational(),
        resolved: unsafe { std::mem::zeroed() },
    };
}
