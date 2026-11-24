use super::StringInterner;

#[test]
fn detached_clone_allocation_is_flat_in_prior_symbol_population() {
    let output = worth_harness::facade::process_deadline::output(
        std::process::Command::new(std::env::current_exe().unwrap())
            .arg(
                concat!(module_path!(), "::isolated_detached_clone_allocation_probe")
                    .split_once("::")
                    .unwrap()
                    .1,
            )
            .arg("--exact")
            .arg("--test-threads=1")
            .env("WORTH_SYMBOL_CLONE_ALLOCATION_PROBE", "1"),
    )
    .expect("isolated symbol allocation probe starts");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report = String::from_utf8_lossy(&output.stdout);
    assert!(
        report.contains("1 passed"),
        "isolated clone probe ran no test:\n{report}"
    );
}

#[test]
fn isolated_detached_clone_allocation_probe() {
    if std::env::var_os("WORTH_SYMBOL_CLONE_ALLOCATION_PROBE").is_none() {
        return;
    }
    let allocator = &stats_alloc::INSTRUMENTED_SYSTEM;
    let control = stats_alloc::Region::new(allocator);
    let allocation = std::hint::black_box(Box::new(7_u64));
    assert!(control.change().bytes_allocated >= std::mem::size_of::<u64>());
    drop(allocation);

    let measured = |count: usize| {
        let mut interner = StringInterner::default();
        for index in 0..count {
            interner
                .intern(&format!("key-{index:05}"))
                .expect("symbol fixture has identity capacity");
        }
        let last = format!("key-{:05}", count - 1);
        let region = stats_alloc::Region::new(allocator);
        let detached = interner.clone();
        let stats = region.change();
        assert_eq!(detached.symbol(&last), interner.symbol(&last));
        stats
    };
    let small = measured(100);
    let large = measured(10_000);
    assert_eq!(small.allocations, large.allocations);
    assert_eq!(small.bytes_allocated, large.bytes_allocated);
}
