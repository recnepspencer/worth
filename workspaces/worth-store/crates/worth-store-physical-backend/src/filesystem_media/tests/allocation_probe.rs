//! Counts the bytes an action allocates. The counter is the process's, and
//! tests run in parallel, so a probe test reruns itself alone in a child
//! process of this test binary: there the harness's main thread only waits
//! for the one test thread, and every byte the counter sees is the action's.

use std::alloc::System;

use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};

#[global_allocator]
static ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const PROBE_TEST: &str = "WORTH_STORE_BACKEND_ALLOCATION_PROBE";

/// True in the child that runs `test` alone, where its probe may count. In
/// the parent, runs that child, asserts it passed, and returns false: the
/// test returns without probing. `module` is the test's `module_path!()`.
pub(super) fn alone_in_its_process(module: &str, test: &str) -> bool {
    let in_crate = module.split_once("::").map_or("", |(_, path)| path);
    let test = format!("{in_crate}::{test}");
    let test = test.as_str();
    if std::env::var_os(PROBE_TEST).is_some_and(|running| running == test) {
        return true;
    }
    let child = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", test, "--test-threads=1"])
        .env(PROBE_TEST, test)
        .output()
        .expect("spawn the probe test alone");
    let stdout = String::from_utf8_lossy(&child.stdout);
    assert!(
        child.status.success() && stdout.contains("test result: ok. 1 passed"),
        "the probe test did not pass alone: {test}\n{stdout}{}",
        String::from_utf8_lossy(&child.stderr)
    );
    false
}

/// The bytes `action` allocates, counted once over its first run. Call it
/// only where `alone_in_its_process` returned true.
pub(super) fn allocated_bytes_during(action: impl FnOnce()) -> usize {
    assert!(
        std::env::var_os(PROBE_TEST).is_some(),
        "an allocation probe counts only in a test running alone"
    );
    let region = Region::new(ALLOCATOR);
    action();
    region.change().bytes_allocated
}
