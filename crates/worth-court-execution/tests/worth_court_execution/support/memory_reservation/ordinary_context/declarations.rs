#[allow(unused_imports)]
pub(crate) use {crate::test_serialization};

pub(crate) use {std::collections::BTreeMap, std::num::NonZeroUsize, std::sync::Mutex, std::sync::OnceLock};

pub(crate) use {worth_execution::CancellationToken, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionMap, worth_execution::ExecutionResourceLease, worth_execution::ExecutionWorkCeiling, worth_execution::KeylessPartition, worth_execution::LeaseDenial, worth_execution::LeaseRequest, worth_execution::MapKernelContext, worth_execution::MapKernelFailure, worth_execution::MapOutcome, worth_execution::MapPartition, worth_execution::MapStop, worth_execution::MemoryLimitDenial, worth_execution::MemoryLimitLevel, worth_execution::SerialMemoryBudget, worth_execution::SerialRequest};
pub(crate) use {worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};

pub(crate) const PROCESS_BYTES: u64 = 1 << 30;

pub(crate) static AUTHORITY: OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
pub(crate) static SERIAL: Mutex<()> = std::sync::Mutex::new(());
#[allow(unused_imports)]
pub(crate) use {crate::memory_reservation::isolated_context::payload_custody};

#[allow(unused_imports)]
pub(crate) use {};

pub(crate) fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: std::num::NonZeroUsize::new(2).unwrap(),
            charged_memory_bytes: Some(PROCESS_BYTES),
        })
        .unwrap()
    })
}

pub(crate) fn policy(bytes: u64) -> ExecutionRequestPolicy {
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::new(1).unwrap(), Some(bytes), Some(u64::MAX)),
    )
}

pub(crate) fn request(bytes: u64) -> LeaseRequest {
    LeaseRequest {
        policy: policy(bytes),
        deadline: None,
        cancellation: worth_execution::CancellationToken::new(),
    }
}

pub(crate) fn lease(bytes: u64) -> ExecutionResourceLease<'static> {
    authority().request_lease(request(bytes)).unwrap()
}

pub(crate) fn one_partition_map() -> ExecutionMap<u64, u64> {
    worth_execution::ExecutionMap::try_from_declared_partitions(
        vec![worth_foundational::PartitionIdentity::new(1)],
        vec![MapPartition {
            identity: worth_foundational::PartitionIdentity::new(1),
            value: 5,
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 256,
            max_result_bytes: 0,
        }],
    )
    .unwrap()
}

pub(crate) fn serial_request(memory: SerialMemoryBudget) -> SerialRequest {
    worth_execution::SerialRequest::from_memory(memory, worth_execution::CancellationToken::new(), None)
}

/// `Some(true)` completed, `Some(false)` the map refused at its admission,
/// `None` the scope refused before the computation.
pub(crate) fn serial_run(bytes: u64) -> Option<bool> {
    serial_run_holding(bytes, None)
}

/// [`serial_run`], with `hold` bytes held on the budget before the run and
/// taken over by the map.
pub(crate) fn serial_run_holding(bytes: u64, hold: Option<u64>) -> Option<bool> {
    let map = one_partition_map();
    let budget = worth_execution::SerialMemoryBudget::from_policy(&policy(bytes));
    let held = hold.map(|hold| budget.reserve(hold).unwrap());
    let request = serial_request(budget);
    let ran = worth_execution::ExecutionWorkCeiling::new(Some(u64::MAX)).run_serial(&request, || {
        let kernel =
            |value: &u64, _: &mut MapKernelContext<'_, '_>| Ok::<_, MapKernelFailure<()>>(*value);
        match held {
            Some(held) => map.run_taking(None, held, kernel),
            None => map.run(None, kernel),
        }
    });
    match ran {
        Ok((worth_execution::MapOutcome::Complete { values, .. }, _)) => {
            assert_eq!(values, vec![5]);
            Some(true)
        }
        Ok((
            worth_execution::MapOutcome::Stopped {
                reason:
                    worth_execution::MapStop::Admission(worth_execution::LeaseDenial::MemoryExhausted(worth_execution::MemoryLimitDenial::Capacity {
                        requested,
                        admitted,
                        level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 },
                    })),
                completed_prefix,
                ..
            },
            _,
        )) => {
            assert!(completed_prefix.is_empty(), "refused before the kernel ran");
            assert!(requested > admitted, "the budget names its own refusal");
            Some(false)
        }
        Ok(_) => panic!("unexpected serial map outcome"),
        Err(_) => None,
    }
}

/// The least budget `admits` admits, between 0 and 1 MiB.
pub(crate) fn least_admitted(admits: impl Fn(u64) -> bool) -> u64 {
    let (mut refused, mut admitted) = (0_u64, 1 << 20);
    assert!(admits(admitted));
    while admitted - refused > 1 {
        let middle = refused + (admitted - refused) / 2;
        if admits(middle) {
            admitted = middle;
        } else {
            refused = middle;
        }
    }
    admitted
}

/// The one-partition map on a child of a `budget`-byte lease while `hold`
/// bytes are held on the parent, as a request holds what it gathered: taken
/// over by the run, or kept beside it. `Err` is the run's memory refusal.
pub(crate) fn leased_run(budget: u64, hold: u64, take: bool) -> Result<(), MemoryLimitDenial> {
    let parent = lease(budget);
    let child = parent.child(request(budget)).unwrap();
    let held = parent.reserve_memory(hold).unwrap();
    let kernel =
        |value: &u64, _: &mut MapKernelContext<'_, '_>| Ok::<_, MapKernelFailure<()>>(*value);
    let outcome = if take {
        one_partition_map().run_taking(Some(&child), held, kernel)
    } else {
        let outcome = one_partition_map().run(Some(&child), kernel);
        drop(held);
        outcome
    };
    let result = match outcome {
        worth_execution::MapOutcome::Complete { values, .. } => {
            assert_eq!(values, vec![5]);
            Ok(())
        }
        worth_execution::MapOutcome::Stopped {
            reason: worth_execution::MapStop::Admission(worth_execution::LeaseDenial::MemoryExhausted(denial)),
            ..
        } => Err(denial),
        worth_execution::MapOutcome::Stopped { .. } => panic!("only memory refuses this run"),
    };
    assert_eq!(
        parent.reserve_memory(budget).map(|held| held.bytes()),
        Ok(budget),
        "nothing stays held once the run settles"
    );
    result
}