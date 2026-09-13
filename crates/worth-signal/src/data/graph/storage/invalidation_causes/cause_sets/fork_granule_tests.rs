use std::env;
use std::process::Command;

use stats_alloc::{Region, INSTRUMENTED_SYSTEM};

use super::CanonicalCauseSetStore;
use crate::data::aspect::Aspect;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::{
    DependencyRevision, OutputCommitOrdinal, ResolvedDependencyCause,
};
use crate::data::proof::PartitionScopeSet;

const TEST_NAME: &str = "data::graph::storage::invalidation_causes::cause_sets::fork_granule_tests::repeated_fork_insert_has_no_inherited_cause_payload_slope";

fn cause(producer_index: u32) -> ResolvedDependencyCause {
    ResolvedDependencyCause::new(
        1,
        NodeId::new(1, 0),
        DependencyRevision(1),
        NodeId::new(producer_index, 0),
        Aspect::new(1),
        None,
        0,
        OutputCommitOrdinal(1),
        1,
        PartitionScopeSet::default(),
    )
}

fn causes(start: u32, len: usize) -> Vec<ResolvedDependencyCause> {
    (0..len)
        .map(|offset| cause(start + offset as u32))
        .collect()
}

#[test]
fn repeated_fork_insert_has_no_inherited_cause_payload_slope() {
    const CHILD_PROCESS: &str = "WORTH_SIGNAL_CAUSE_SET_GRANULE_CHILD";
    // One changed vector page has 32 outer slots. These ceilings conservatively
    // allow one allocation and 512 bytes of bounded bookkeeping per slot.
    const MAX_CHANGED_PAGE_ALLOCATION_CALLS: usize = 32;
    const MAX_CHANGED_PAGE_ALLOCATED_BYTES: usize = 32 * 512;
    if env::var_os(CHILD_PROCESS).is_none() {
        let output = Command::new(env::current_exe().expect("test executable resolves"))
            .arg("--exact")
            .arg(TEST_NAME)
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env(CHILD_PROCESS, "1")
            .output()
            .expect("isolated cause-set granule probe starts");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "isolated cause-set granule probe failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
        assert!(
            stdout.contains(&format!("test {TEST_NAME} ... ok")),
            "isolated cause-set granule probe selected no exact test\nstdout:\n{stdout}"
        );
        print!("{stdout}");
        eprint!("{stderr}");
        return;
    }

    let mut allocation_samples = Vec::new();
    for inherited_len in [64, 4_096, 65_536] {
        let base_expected = vec![cause(0)];
        let inherited_expected = causes(10, inherited_len);
        let appended_expected = vec![cause(u32::MAX)];
        let appended_input = cause(u32::MAX);
        let mut original = CanonicalCauseSetStore::default();
        let base = original.insert(base_expected.iter().cloned());
        let mut first_fork = original.fork_persistent();
        let inherited = first_fork.insert(inherited_expected.iter().cloned());
        let mut repeated_fork = first_fork.fork_persistent();
        assert!(first_fork.shares_storage_with(&repeated_fork));

        let region = Region::new(&INSTRUMENTED_SYSTEM);
        let appended = repeated_fork.insert([appended_input]);
        let allocation = region.change();
        allocation_samples.push((
            inherited_len,
            allocation.allocations,
            allocation.bytes_allocated,
        ));

        assert_eq!(
            original.get(base).expect("base remains live"),
            base_expected.as_slice()
        );
        assert_eq!(
            first_fork.get(inherited).expect("tail remains live"),
            inherited_expected.as_slice()
        );
        assert_eq!(
            repeated_fork.get(inherited).expect("tail is inherited"),
            inherited_expected.as_slice()
        );
        assert_eq!(
            repeated_fork.get(appended).expect("singleton installs"),
            appended_expected.as_slice()
        );

        repeated_fork
            .release(appended)
            .expect("singleton slot releases");
        assert!(repeated_fork.get(appended).is_err());
        let reused_expected = vec![cause(u32::MAX - 1)];
        let reused = repeated_fork.insert(reused_expected.iter().cloned());
        assert_ne!(reused, appended);
        assert!(repeated_fork.get(appended).is_err());
        assert_eq!(
            repeated_fork.get(reused).expect("slot is reused"),
            reused_expected.as_slice()
        );
        assert_eq!(first_fork.occupied_slot_count(), 2);
        assert_eq!(repeated_fork.occupied_slot_count(), 3);
    }

    eprintln!("repeated-fork cause-set allocation samples: {allocation_samples:?}");
    let (_, expected_calls, expected_bytes) = allocation_samples[0];
    for (inherited_len, calls, bytes) in allocation_samples {
        assert_eq!(
            (calls, bytes),
            (expected_calls, expected_bytes),
            "singleton cause insertion cost changed with inherited payload length {inherited_len}"
        );
    }
    assert!(
        expected_calls <= MAX_CHANGED_PAGE_ALLOCATION_CALLS,
        "shared-page write used {expected_calls} allocations"
    );
    assert!(
        expected_bytes <= MAX_CHANGED_PAGE_ALLOCATED_BYTES,
        "shared-page write allocated {expected_bytes} bytes"
    );
}
