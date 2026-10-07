use super::*;
use crate::orchestration::recovery_budget::recovery_limit_for_test;

/// Recovery declared 1,000 bytes of memory, and one of every other count.
fn limits() -> PhysicalRecoveryLimitDeclaration {
    let mut values = [1; 19];
    values[12] = 1_000;
    PhysicalRecoveryLimitDeclaration::from_values_for_test(values)
}

fn memory(observed: u64) -> PhysicalRecoveryBlockCause {
    PhysicalRecoveryBlockCause::Limit {
        phase: PhysicalRecoveryBlockKind::SourceAllocation,
        limit: recovery_limit_for_test(
            PhysicalRecoveryLimitDimension::RecoveryMemoryBytes,
            observed,
            1_000,
        )
        .into(),
    }
}

fn reserve_error() -> std::collections::TryReserveError {
    Vec::<u8>::new().try_reserve(usize::MAX).unwrap_err()
}

#[test]
fn refused_memory_is_recovery_memorys_limit_with_both_counts() {
    // Handed 600 of recovery's 1,000 bytes, the owner needed 700: recovery
    // held 400 beside it, so it needed 1,100 in all.
    let resident = Resident::BudgetExceeded {
        required: 700,
        admitted: 600,
    };
    assert_eq!(stopped(&limits(), &resident).cause, memory(1_100));
    let admission = Admission::RecoveryMemoryBytes {
        observed: 1_001,
        admitted: 1_000,
    };
    assert_eq!(
        stopped(&limits(), &Read::Residency(resident)).cause,
        memory(1_100)
    );
    let failure = source_allocation(&limits(), Wal::Ownership(admission), |cause| {
        PhysicalRecoverySourceDenial::WalAdmissionAllocation { cause }
    });
    assert_eq!(failure.cause, memory(1_001));
    assert_eq!(
        failure.source_denials,
        [PhysicalRecoverySourceDenial::WalAdmissionAllocation {
            cause: Wal::Ownership(admission)
        }]
    );
}

fn cause(refused: &impl MemoryRefused) -> PhysicalRecoveryBlockCause {
    stopped(&limits(), refused).cause
}

/// However deep the owner that refused its charge sits, the read states
/// recovery memory's limit with that owner's counts.
#[test]
fn every_read_that_refused_memory_states_recovery_memorys_limit() {
    use worth_store::physical_runtime::{
        ArtifactTreeListingAllocationBoundary as Listing,
        ArtifactTreePathAllocationBoundary as Path,
    };
    // Handed 600 of recovery's 1,000 bytes, the owner needed 700: 1,100 in all.
    let refused = || Resident::BudgetExceeded {
        required: 700,
        admitted: 600,
    };
    let admission = Admission::RecoveryMemoryBytes {
        observed: 1_001,
        admitted: 1_000,
    };
    let wal = Wal::Backing {
        requested: 700,
        cause: refused(),
    };
    assert_eq!(cause(&wal), memory(1_100));
    let listing = Observation::ListingResidency {
        boundary: Listing::EntryName,
        cause: refused(),
    };
    let path = Observation::PathResidency {
        boundary: Path::FileOpen,
        cause: refused(),
    };
    assert_eq!(cause(&Observation::Residency(refused())), memory(1_100));
    assert_eq!(cause(&listing), memory(1_100));
    assert_eq!(cause(&path), memory(1_100));
    assert_eq!(cause(&Read::Observation(path)), memory(1_100));
    let binding = Binding::Backing {
        requested: 700,
        cause: refused(),
    };
    assert_eq!(cause(&binding), memory(1_100));
    assert_eq!(cause(&Read::BindingDecode(binding.clone())), memory(1_100));
    assert_eq!(cause(&Read::BindingBasis(binding)), memory(1_100));
    assert_eq!(cause(&Read::Admission(admission)), memory(1_001));
    let sample = Sample::Backing {
        requested: 700,
        cause: refused(),
    };
    assert_eq!(cause(&sample), memory(1_100));
    assert_eq!(cause(&Sample::Ownership(admission)), memory(1_001));
    let local = Sample::LocalLimit {
        required: 700,
        admitted: 600,
    };
    assert_eq!(cause(&local), memory(1_100));
}

#[test]
fn an_allocation_that_refused_no_memory_names_no_limit() {
    let damage = PhysicalRecoveryBlockCause::Damage(PhysicalRecoveryBlockKind::SourceAllocation);
    for read in [
        Read::Residency(Resident::Allocation {
            requested: 8,
            cause: reserve_error(),
        }),
        Read::Residency(Resident::AllocatorExceededReservation {
            requested: 8,
            actual: 16,
        }),
        Read::AllocatorExceededReservation {
            requested: 8,
            actual: 16,
        },
        Read::ReadBufferLengthMismatch {
            requested: 8,
            observed: 7,
        },
        Read::Admission(Admission::SizeOverflow),
    ] {
        assert_eq!(stopped(&limits(), &read).cause, damage, "{read:?}");
    }
    // A memory count no part of recovery's names no limit either.
    let wider = Resident::BudgetExceeded {
        required: 2_001,
        admitted: 2_000,
    };
    assert_eq!(stopped(&limits(), &wider).cause, damage);
}
