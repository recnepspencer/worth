use worth_proof::NonEmpty;
use worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission;
use worth_store::physical_runtime::{
    IndeterminatePhysicalWalGroupAppend, PhysicalWalAppendFailureCause,
    PhysicalWalGroupAppendContinuation, PhysicalWalGroupAppendFailureCause,
    PhysicalWalGroupAppendOutcome, PreparedPhysicalMutation, RejectedPhysicalDurabilityGroup,
    SealedPhysicalDurabilityGroupMembers, WalAppendedPhysicalMutation,
};
enum WalGroupAppendDecision {
    Appended(SealedPhysicalDurabilityGroupMembers),
    NotAdmitted {
        members: NonEmpty<PreparedPhysicalMutation>,
        cause: PhysicalWalGroupAppendFailureCause,
    },
    AdmissionRejected(RejectedPhysicalDurabilityGroup),
    Continue(PhysicalWalGroupAppendContinuation),
    Inspect(IndeterminatePhysicalWalGroupAppend),
}

fn append_wal_group(
    submission: &CertificationPhysicalRecordSubmission,
    members: NonEmpty<PreparedPhysicalMutation>,
) -> WalGroupAppendDecision {
    match submission.append_prepared_wal_group(members) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => {
            WalGroupAppendDecision::Appended(appended)
        }
        PhysicalWalGroupAppendOutcome::NotAdmitted { members, cause } => {
            WalGroupAppendDecision::NotAdmitted { members, cause }
        }
        PhysicalWalGroupAppendOutcome::AdmissionRejected(rejected) => {
            WalGroupAppendDecision::AdmissionRejected(rejected)
        }
        PhysicalWalGroupAppendOutcome::NotStarted(continuation)
        | PhysicalWalGroupAppendOutcome::PartiallyAppended(continuation) => {
            WalGroupAppendDecision::Continue(continuation)
        }
        PhysicalWalGroupAppendOutcome::Indeterminate(indeterminate) => {
            WalGroupAppendDecision::Inspect(indeterminate)
        }
    }
}

use worth_store::physical_runtime::{
    IndeterminatePhysicalWalGroupBarrier, PhysicalWalGroupBarrierFailureCause,
    PhysicalWalGroupBarrierOutcome, WalDurablePhysicalMutationMembers,
};

enum WalGroupBarrierDecision {
    Durable(WalDurablePhysicalMutationMembers),
    Retry {
        appended: SealedPhysicalDurabilityGroupMembers,
        cause: PhysicalWalGroupBarrierFailureCause,
    },
    Inspect(IndeterminatePhysicalWalGroupBarrier),
}

fn synchronize_wal_group(
    submission: &CertificationPhysicalRecordSubmission,
    appended: SealedPhysicalDurabilityGroupMembers,
) -> WalGroupBarrierDecision {
    match submission.synchronize_appended_wal_group(appended) {
        PhysicalWalGroupBarrierOutcome::Durable(durable) => {
            WalGroupBarrierDecision::Durable(durable)
        }
        PhysicalWalGroupBarrierOutcome::BarrierNotStarted { appended, cause } => {
            WalGroupBarrierDecision::Retry { appended, cause }
        }
        PhysicalWalGroupBarrierOutcome::Indeterminate(indeterminate) => {
            WalGroupBarrierDecision::Inspect(indeterminate)
        }
    }
}

fn blocked_dependency_keeps_its_signal_classification(
    cause: &PhysicalWalAppendFailureCause,
) -> bool {
    match cause {
        PhysicalWalAppendFailureCause::DependencyBlocked { class, condition } => {
            let _exact_signal_cause = (*class, *condition);
            true
        }
        _ => false,
    }
}

use worth_store::physical_runtime::PhysicalRecordSubmission;

fn inspect_wal_append(
    submission: &PhysicalRecordSubmission,
    appended: &WalAppendedPhysicalMutation,
) -> Result<(), &'static str> {
    let declaration = appended.reserved().declaration();
    let settlement = appended.settlement();

    assert_eq!(settlement.range(), declaration.artifact_range());
    assert_eq!(
        settlement.payload_digest(),
        declaration.payload_digest(),
    );

    let wal = submission
        .wal_observation()
        .ok_or("the serving Store has released its publication authority")?;

    assert!(wal.appended_frames() >= 1);
    assert_eq!(
        wal.last_lsn_end(),
        Some(declaration.lsn_range().end_exclusive().get()),
    );
    assert!(!wal.sealed_for_inspection());
    Ok(())
}

fn main() {
    let _ = (
        append_wal_group,
        synchronize_wal_group,
        blocked_dependency_keeps_its_signal_classification,
        inspect_wal_append,
    );
}
