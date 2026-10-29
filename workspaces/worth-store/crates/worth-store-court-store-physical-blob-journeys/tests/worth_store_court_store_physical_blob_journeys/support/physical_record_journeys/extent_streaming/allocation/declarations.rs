#[derive(Debug, Clone, Copy)]
pub(crate) struct ExtentAllocationObservation {
    pub(crate) append: usize,
    pub(crate) read: usize,
}

pub(crate) fn prove() {
    let short = measured_extent_append(17 * (16_384 - 104) as u64 + 7);
    let long = measured_extent_append(65 * (16_384 - 104) as u64 + 7);
    for (operation, short, long) in [
        ("append", short.append, long.append),
        ("read", short.read, long.read),
    ] {
        assert!(
            short >= 16_384,
            "C5_PREDICATE:transfer-allocation-slope the bounded {operation} frame allocation must be visible"
        );
        assert_eq!(
            long, short,
            "C5_PREDICATE:transfer-allocation-slope {operation}: {short} -> {long}"
        );
    }
}

pub(crate) fn measured_extent_append(logical_bytes: u64) -> ExtentAllocationObservation {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let output = worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
        "allocation_writer",
        &root,
        Some(&logical_bytes.to_string()),
    );
    let row = output
        .lines()
        .find_map(|line| line.strip_prefix("C5_ALLOC "))
        .expect("allocation child must report its operation evidence");
    let mut fields = row.split_whitespace();
    let append = fields.next().unwrap().parse().unwrap();
    let scratch: u64 = fields.next().unwrap().parse().unwrap();
    let locator = fields.next().unwrap();
    assert_eq!(
        scratch, 16_384,
        "append scratch remains a bounded mutable frame"
    );
    let output = worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child("allocation_reader", &root, Some(locator));
    let row = output
        .lines()
        .find_map(|line| line.strip_prefix("C5_READ_ALLOC "))
        .expect("allocation child must report its read evidence");
    let mut fields = row.split_whitespace();
    let read = fields.next().unwrap().parse().unwrap();
    let scratch: u64 = fields.next().unwrap().parse().unwrap();
    assert_eq!(
        scratch, 0,
        "resident leases are not operation scratch allocations"
    );
    ExtentAllocationObservation { append, read }
}