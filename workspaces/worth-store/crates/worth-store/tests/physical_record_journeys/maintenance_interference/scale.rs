use super::{append_copies, checkpoint, initialize, placement};

#[test]
fn persisted_payload_reaches_eight_resident_budgets() {
    persist(8, false);
}

#[test]
fn persisted_payload_reaches_thirty_two_resident_budgets() {
    persist(32, false);
}

// The 128x world takes close to an hour in a debug build, so it runs in the
// scheduled lane: `cargo test --release ... -- --ignored scale::`.
#[test]
#[ignore = "scheduled release-scale Phase 6 lane"]
fn persisted_payload_reaches_128x_resident_budget() {
    persist(128, true);
}

fn persist(multiple: u64, require_reclaim: bool) {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = initialize(&root);
    // The canonical policy's 8 MiB hard ceiling, with progress headroom withheld
    // inside it; the product default also covers the declared WAL tail.
    let ceiling = 8 * 1024 * 1024;
    serving.certification_limit_candidate_growth_bytes(ceiling - 64 * 1024);
    let policy = placement();
    let resident = 64 * 1024;
    let target = multiple * resident;
    let record = [0x22; 4096];
    let copies = 4_usize;
    let mut payload = 0_u64;
    let mut ordinal = 1_u64;
    let mut reclaims = 0_u64;
    while payload < target {
        if append_copies(&serving, policy, ordinal, &record, copies).is_none() {
            let charged = serving.certification_charged_growth_bytes();
            assert!(
                charged <= ceiling,
                "denial charged {charged}, above the 8 MiB excess ceiling"
            );
            checkpoint(&serving, ordinal);
            ordinal += 1;
            reclaims += 1;
            assert!(
                append_copies(&serving, policy, ordinal, &record, copies).is_some(),
                "batch {ordinal} stayed denied after reclaim charged={}",
                serving.certification_charged_growth_bytes()
            );
        }
        payload += copies as u64 * record.len() as u64;
        ordinal += 1;
        assert!(
            ordinal < target / 1024 + 80,
            "did not reach {multiple} resident budgets"
        );
    }
    assert!(payload >= target);
    let charged = serving.certification_charged_growth_bytes();
    assert!(
        charged <= ceiling,
        "charged {charged} exceeded the 8 MiB excess ceiling"
    );
    if require_reclaim {
        assert!(
            reclaims >= 1,
            "128x reached {payload} bytes without reclaim under the 8 MiB ceiling"
        );
    }
    let peak = serving
        .certification_physical_residency()
        .counters()
        .peak_resident_bytes();
    assert!(peak <= resident, "resident frames peaked at {peak}");
    serving.close();
}
