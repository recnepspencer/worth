#[allow(unused_imports)] use crate::memory_reservation::ordinary_context::*;
#[allow(unused_imports)]
pub(crate) use {crate::test_serialization};

pub(crate) use {std::collections::BTreeMap, std::num::NonZeroUsize, std::sync::Mutex, std::sync::OnceLock};

pub(crate) use {worth_execution::CancellationToken, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionMap, worth_execution::ExecutionResourceLease, worth_execution::ExecutionWorkCeiling, worth_execution::KeylessPartition, worth_execution::LeaseDenial, worth_execution::LeaseRequest, worth_execution::MapKernelContext, worth_execution::MapKernelFailure, worth_execution::MapOutcome, worth_execution::MapPartition, worth_execution::MapStop, worth_execution::MemoryLimitDenial, worth_execution::MemoryLimitLevel, worth_execution::SerialMemoryBudget, worth_execution::SerialRequest};
pub(crate) use {worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};
#[allow(unused_imports)]
pub(crate) use {crate::memory_reservation::isolated_context::payload_custody};

#[allow(unused_imports)]
pub(crate) use {};

#[test]
fn a_serial_budget_holds_the_policy_bytes_and_names_what_it_admits() {
    let budget = worth_execution::SerialMemoryBudget::from_policy(&policy(100));
    let mut first = budget.reserve(60).unwrap();
    assert_eq!(
        budget.reserve(41).unwrap_err(),
        worth_execution::MemoryLimitDenial::Capacity {
            requested: 41,
            admitted: 40,
            level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 },
        }
    );
    let second = budget.reserve(40).unwrap();
    assert_eq!(
        first.resize(61).unwrap_err(),
        worth_execution::MemoryLimitDenial::Capacity {
            requested: 61,
            admitted: 60,
            level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 },
        }
    );
    assert_eq!(first.bytes(), 60, "a refused resize keeps what was held");
    first.resize(10).unwrap();
    drop(second);
    assert_eq!(budget.reserve(90).unwrap().bytes(), 90);
}

#[test]
fn a_serial_run_with_a_policy_refuses_at_its_own_memory_boundary() {
    assert_eq!(serial_run(0), None, "the scope holds its framework bytes");
    let (mut refused, mut admitted) = (0_u64, 1 << 20);
    assert_eq!(serial_run(admitted), Some(true));
    while admitted - refused > 1 {
        let middle = refused + (admitted - refused) / 2;
        if serial_run(middle) == Some(true) {
            admitted = middle;
        } else {
            refused = middle;
        }
    }
    assert_eq!(
        serial_run(admitted - 1),
        Some(false),
        "one byte short refuses"
    );
    assert_eq!(serial_run(admitted), Some(true));

    let funded = worth_execution::ExecutionWorkCeiling::new(Some(u64::MAX)).run_serial(
        &serial_request(worth_execution::SerialMemoryBudget::from_policy(&policy(1 << 20))),
        || one_partition_map().run(None, |value, _| Ok::<_, MapKernelFailure<()>>(*value)),
    );
    assert!(matches!(funded, Ok((worth_execution::MapOutcome::Complete { .. }, _))));
}

/// The same on a serial budget: the scope's framework bytes and the hold fit
/// together, and the map then fits in the hold's place.
#[test]
fn a_serial_run_takes_over_the_hold_on_its_inputs() {
    let admitted = least_admitted(|budget| serial_run(budget) == Some(true));
    let hold = 1;
    assert_eq!(serial_run_holding(admitted, Some(hold)), Some(true));
    assert_eq!(serial_run_holding(admitted - 1, Some(hold)), Some(false));
}
