use super::*;
use crate::{
    indexes::data::DerivedIndexExecutionDenialKind as IndexKind,
    transactions::data::CommitExecutionDenialKind as CommitKind,
};

pub(super) fn assert_both(
    stop: impl Fn() -> PacketExecutionStop,
    expected: Cause,
    boundary: Option<u64>,
) {
    let commit: crate::transactions::data::CommitExecutionDenial = stop().into();
    let index: crate::indexes::data::DerivedIndexExecutionDenial = stop().into();
    assert_eq!(commit.kind, CommitKind::Cause(expected));
    assert_eq!(index.kind, IndexKind::Cause(expected));
    assert_eq!(commit.partition_identity, boundary);
    assert_eq!(index.partition_identity, boundary);
}

#[test]
fn every_lease_cause_survives_both_domain_doors() {
    let cases = [
        (
            LeaseDenial::WorkerLimitExceedsParent,
            Cause::WorkerLimitExceedsParent,
        ),
        (
            LeaseDenial::MemoryLimitExceedsParent,
            Cause::MemoryLimitExceedsParent,
        ),
        (
            LeaseDenial::WorkLimitExceedsParent,
            Cause::WorkLimitExceedsParent,
        ),
        (
            LeaseDenial::ChargedBytesOverflow,
            Cause::ChargedBytesOverflow,
        ),
        (
            LeaseDenial::UnrelatedNestedLease,
            Cause::UnrelatedNestedLease,
        ),
        (
            LeaseDenial::NoActiveExecutionScope,
            Cause::NoActiveExecutionScope,
        ),
        (
            LeaseDenial::EquivalenceContractUnavailable,
            Cause::EquivalenceContractUnavailable,
        ),
        (
            LeaseDenial::MemoryExhausted(MemoryLimitDenial {
                requested: 31,
                admitted: 17,
                level: MemoryLimitLevel::Policy { ancestor: 2 },
            }),
            Cause::PolicyMemoryExhausted {
                requested: 31,
                admitted: 17,
                ancestor: 2,
            },
        ),
        (
            LeaseDenial::MemoryExhausted(MemoryLimitDenial {
                requested: 31,
                admitted: 17,
                level: MemoryLimitLevel::Process,
            }),
            Cause::ProcessMemoryExhausted {
                requested: 31,
                admitted: 17,
            },
        ),
        (
            LeaseDenial::MemoryExhausted(MemoryLimitDenial {
                requested: 31,
                admitted: 17,
                level: MemoryLimitLevel::Declared,
            }),
            Cause::DeclaredMemoryExhausted {
                requested: 31,
                admitted: 17,
            },
        ),
    ];
    for (denial, expected) in cases {
        assert_both(
            || PacketExecutionStop::Execution {
                boundary: None,
                reason: MapStop::Admission(denial),
            },
            expected,
            None,
        );
    }
}

#[test]
fn every_kernel_cause_survives_both_domain_doors() {
    let cases = [
        (MapKernelStop::Cancelled, Cause::Cancelled),
        (MapKernelStop::DeadlineElapsed, Cause::DeadlineElapsed),
        (MapKernelStop::WorkCeiling, Cause::WorkExhausted),
        (
            MapKernelStop::WorkCounterOverflow,
            Cause::WorkCounterOverflow,
        ),
        (MapKernelStop::NestedStopped, Cause::NestedStopped),
    ];
    for (stop, expected) in cases {
        assert_both(
            || PacketExecutionStop::Execution {
                boundary: Some(PartitionIdentity::new(7)),
                reason: MapStop::Failure {
                    identity: PartitionIdentity::new(7),
                    cause: MapKernelFailure::Stop(stop),
                },
            },
            expected,
            Some(7),
        );
    }
    assert_both(
        || PacketExecutionStop::Execution {
            boundary: Some(PartitionIdentity::new(7)),
            reason: MapStop::WorkExhausted {
                identity: PartitionIdentity::new(7),
            },
        },
        Cause::WorkExhausted,
        Some(7),
    );
}

#[test]
fn every_packet_failure_survives_both_domain_doors() {
    type PacketFailureFactory = fn() -> MapKernelFailure<PacketBudgetDenial>;
    let cases: [(PacketFailureFactory, Cause); 4] = [
        (|| MapKernelFailure::Panic, Cause::WorkerFailed),
        (
            || MapKernelFailure::ResultCapacityExceeded,
            Cause::ResultCapacityExceeded,
        ),
        (
            || MapKernelFailure::Domain(PacketBudgetDenial::ScratchCapacityExceeded),
            Cause::ScratchCapacityExceeded,
        ),
        (
            || MapKernelFailure::Domain(PacketBudgetDenial::UncheckedCustomKernel),
            Cause::UncheckedCustomKernel,
        ),
    ];
    for (failure, expected) in cases {
        assert_both(
            || PacketExecutionStop::Execution {
                boundary: Some(PartitionIdentity::new(7)),
                reason: MapStop::Failure {
                    identity: PartitionIdentity::new(7),
                    cause: failure(),
                },
            },
            expected,
            Some(7),
        );
    }
}

#[test]
fn keyless_admission_keeps_identity_and_memory_refusals_typed() {
    for (denial, cause) in [
        (
            PacketAdmissionDenial::ExpectedIdentitiesNotCanonical,
            Cause::ExpectedIdentitiesNotCanonical,
        ),
        (PacketAdmissionDenial::MemoryOverflow, Cause::MemoryOverflow),
    ] {
        assert_both(|| PacketExecutionStop::Admission(denial), cause, None);
    }
}
