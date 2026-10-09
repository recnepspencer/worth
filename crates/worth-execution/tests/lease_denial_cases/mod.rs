//! Authority-owned admission cases shared by downstream conversion tests.
use std::{num::NonZeroUsize, sync::OnceLock};
use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionResourceLease,
    ExecutionScan, ExecutionWorkCeiling, LeaseDenial, LeaseRequest, MapKernelFailure, MapStop,
    ScanOutcome,
};
use worth_foundational::{
    DeterminismContract, EquivalenceContractId, ExecutionBudget, ExecutionPosture,
    ExecutionRequestPolicy, PartitionIdentity,
};

fn authority() -> &'static ExecutionAuthority {
    static OWNER: OnceLock<ExecutionAuthority> = OnceLock::new();
    OWNER.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(64 << 20),
        })
        .unwrap()
    })
}
fn request(workers: usize, memory: u64, work: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(workers).unwrap(), memory, work),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    }
}
fn lease() -> ExecutionResourceLease<'static> {
    authority()
        .request_lease(request(1, 2 << 20, 1_000))
        .unwrap()
}
fn scan() -> ExecutionScan<()> {
    let key = PartitionIdentity::new(1);
    ExecutionScan::try_from_ordered(vec![key], vec![(key, ())]).unwrap()
}
fn refused(lease: Option<&ExecutionResourceLease<'_>>, state: u64, scratch: u64) -> LeaseDenial {
    let result = scan().run(lease, (), state, 0, 0, scratch, |_, _, _| {
        Ok::<_, MapKernelFailure<()>>(((), ()))
    });
    match result {
        ScanOutcome::Stopped {
            reason: MapStop::Admission(denial),
            ..
        } => denial,
        _ => panic!("operation must be refused at execution admission"),
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Cause {
    Workers,
    MemoryLimit,
    Work,
    Memory,
    Overflow,
    Nested,
    Equivalence,
    MissingScope,
}
pub const CASES: [Cause; 8] = [
    Cause::Workers,
    Cause::MemoryLimit,
    Cause::Work,
    Cause::Memory,
    Cause::Overflow,
    Cause::Nested,
    Cause::Equivalence,
    Cause::MissingScope,
];

pub fn denial(cause: Cause) -> LeaseDenial {
    let parent = lease();
    match cause {
        Cause::Workers => parent.child(request(2, 2 << 20, 1000)).unwrap_err(),
        Cause::MemoryLimit => parent.child(request(1, 4 << 20, 1000)).unwrap_err(),
        Cause::Work => parent.child(request(1, 2 << 20, 1001)).unwrap_err(),
        Cause::Memory => refused(Some(&parent), 0, 4 << 20),
        Cause::Overflow => refused(Some(&parent), u64::MAX, 0),
        Cause::Nested => {
            ExecutionWorkCeiling::new(1000)
                .run(&parent, || refused(None, 0, 0))
                .unwrap()
                .0
        }
        Cause::MissingScope => {
            worth_execution::ExecutionMemoryReservation::reserve_in_scope(None, 1).unwrap_err()
        }
        Cause::Equivalence => {
            let mut child = request(1, 2 << 20, 1000);
            child.policy = ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::ContractEquivalent(EquivalenceContractId::new(91)),
                child.policy.budget(),
            );
            parent.child(child).unwrap_err()
        }
    }
}
