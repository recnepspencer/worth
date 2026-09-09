use super::*;

#[test]
fn prepared_bound_covers_shared_page_clone_and_override_growth() {
    let mut values: PersistentVector<String, 8> = (0..16).map(|_| "base".to_owned()).collect();
    values.prepare_retained_charge(&mut Work::new(100)).unwrap();
    let base = values.fork_persistent();
    for index in 0..16 {
        // Sharing a partially populated page forces its Arc vectors to clone
        // before insertion. In particular, a three-entry clone may grow to six.
        let sibling = values.clone();
        let before = values.prepared_retained_charge().unwrap();
        let input = "replacement".repeat(200);
        let pointer = input.as_ptr();
        let prepared = values
            .prepare_retained_replacement(index, input, &mut Work::new(100_000))
            .unwrap();
        let required = prepared.required_charge();
        let short = required
            .checked_sub(Charge::capacity::<u8>(1).unwrap())
            .unwrap();
        let (input, denial) = prepared.publish(short).unwrap_err();
        assert_eq!(input.as_ptr(), pointer);
        assert_eq!(
            denial,
            RetainedVectorCapacityDenial::CapacityExhausted {
                maximum: short,
                required,
            }
        );
        assert!(values.shares_storage_with(&sibling));
        assert_eq!(values.prepared_retained_charge().unwrap(), before);
        let prepared = values
            .prepare_retained_replacement(index, input, &mut Work::new(100_000))
            .unwrap();
        assert_eq!(prepared.required_charge(), required);
        let actual = prepared.publish(required).unwrap();
        assert!(actual <= required);
        // Independent full representation measurement, including hidden base.
        assert_eq!(
            actual,
            values
                .retained_heap_charge(&mut Work::new(100_000))
                .unwrap()
        );
        assert_eq!(sibling[index], "base");
        assert_eq!(base[index], "base");
    }
}

#[test]
fn preparation_and_capacity_denial_allocate_no_replacement_storage() {
    const CHILD: &str = "WORTH_SIGNAL_REPLACEMENT_ADMISSION_ALLOCATION_CHILD";
    const TEST: &str = "data::persistent_vector::replacement_charge::prospective_tests::preparation_and_capacity_denial_allocate_no_replacement_storage";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(TEST) && stdout.contains("1 passed; 0 failed"));
        return;
    }
    let mut values: PersistentVector<String> = ["base".into()].into_iter().collect();
    values.prepare_retained_charge(&mut Work::new(100)).unwrap();
    let _retained = values.fork_persistent();
    let input = "replacement".repeat(200);
    let mut work = Work::new(100_000);
    let region = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
    let prepared = values
        .prepare_retained_replacement(0, input, &mut work)
        .unwrap();
    let required = prepared.required_charge();
    let short = required
        .checked_sub(Charge::capacity::<u8>(1).unwrap())
        .unwrap();
    let (input, _) = prepared.publish(short).unwrap_err();
    let denied = region.change();
    assert_eq!(denied.allocations, 0);
    assert_eq!(denied.reallocations, 0);
    let prepared = values
        .prepare_retained_replacement(0, input, &mut work)
        .unwrap();
    let region = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
    prepared.publish(required).unwrap();
    assert!(
        region.change().allocations > 0,
        "publication is a positive allocation control"
    );
}
