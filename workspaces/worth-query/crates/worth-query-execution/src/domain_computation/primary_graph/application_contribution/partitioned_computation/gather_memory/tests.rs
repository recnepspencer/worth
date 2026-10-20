//! Declared capacities with no sum are an overflow, under a request whose
//! memory admits one of them.

use std::num::NonZeroUsize;
use std::sync::Arc;

use worth_foundational::facade::PartitionIdentity;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy;
use crate::domain_computation::primary_graph::tests::fixture::live_scope;

struct Nothing;

impl ChargedBytes for Nothing {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

type Denial = WorthQueryPartitionedComputationDenial<u32>;

#[test]
fn a_second_declared_capacity_with_no_sum_is_an_overflow_before_its_gather() {
    const CHILD: &str = "WORTH_QUERY_GATHER_OVERFLOW_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let filter = concat!(
            module_path!(),
            "::a_second_declared_capacity_with_no_sum_is_an_overflow_before_its_gather"
        )
        .split_once("::")
        .unwrap()
        .1;
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", filter, "--nocapture"])
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "isolated capacity proof failed: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
        return;
    }
    assert_eq!(std::env::var(CHILD).unwrap(), "1");
    let request = live_scope();
    let execution = QueryRequestExecution::open(
        RuntimeWorldExecutionPlacement::Serial(test_policy(NonZeroUsize::MIN, None)),
        &request,
    );
    let mut memory = GatheredMemory::new::<u32>(&execution, u64::MAX).unwrap();
    // The request's memory admits the largest declaration once.
    assert_eq!(memory.before_gather::<u32>(&execution), Ok(()));
    let partition = GatheredComputationPartition {
        identity: PartitionIdentity::new(1),
        key: Arc::new(()),
        items: Arc::from([]),
        gathered: Nothing,
    };
    assert_eq!(memory.after_gather::<_, _, u32>(&partition), Ok(()));
    // What the first partition settled plus the same declaration has no
    // value: the refusal is arithmetic, not the request's memory.
    let overflow: Denial = WorthQueryPartitionedComputationDenial::Resource(
        WorthQueryManagedComputationResourceDenial::CapacityOverflow,
    );
    assert_eq!(memory.before_gather::<u32>(&execution), Err(overflow));
}
