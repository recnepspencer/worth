use std::{num::NonZeroU32, path::Path};

use worth_proof::{NonEmpty, TransitionOutcome};
use worth_store::physical_runtime::certification::MediaFaultDirective;
use worth_store::physical_runtime::{
    FilesystemMediaAdmission, PhysicalMutationIdempotencyMaterial, PhysicalRecordOpen,
    PhysicalRuntimeAdmission, PhysicalStore, PhysicalWalGroupAppendOutcome,
    PhysicalWalGroupBarrierFailureCause, PhysicalWalGroupBarrierOutcome,
    SealedPhysicalDurabilityGroupMembers,
};
use worth_store_physical_backend::{
    ArtifactTreeFailureKind, BackendTargetProfile, CertificationMediaFaultActivation,
    FilesystemAccessPosture, MediaOperationRole, WalDurabilityBarrier,
};

use super::super::{configuration, durability_with_group_limit};

#[test]
fn exact_appended_member_crosses_the_scheduled_barrier_into_wal_durable_authority() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = super::super::serving_from_initialization(&store_root);
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let appended = append(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([91; 32]),
    );
    let appended_member = appended.members()[0].mutation();
    let mutation = appended_member.mutation_identity();
    let member = appended_member.reserved().member_basis();
    let append_work = appended_member.settlement().work_identity();
    let append_operation = appended_member.settlement().backend_operation();
    let durability = serving.durability_observation();

    let durable = match submission.synchronize_appended_wal_group(appended) {
        PhysicalWalGroupBarrierOutcome::Durable(durable) => durable
            .into_members()
            .into_vec()
            .pop()
            .expect("one sealed member derives one durable member"),
        _ => panic!("the canonical scheduled barrier must mint WAL-durable authority"),
    };
    let settlement = durable.barrier_settlement();
    assert_eq!(durable.mutation_identity(), mutation);
    assert_eq!(durable.member_basis(), member);
    assert_eq!(durable.appended().reserved().member_basis(), member);
    assert_ne!(settlement.work(), append_work);
    assert_eq!(settlement.effect().work(), settlement.work());
    assert_ne!(settlement.backend_operation(), append_operation);
    assert_eq!(settlement.member_basis(), member);
    assert_eq!(settlement.policy_identity(), durability.policy_identity());
    assert_eq!(
        settlement.admission_basis_identity(),
        durability.admission_basis_identity()
    );
    assert_eq!(settlement.profile(), durability.profile());
    assert_eq!(settlement.barrier(), required_barrier(durability.profile()));
    assert_ne!(settlement.binding_digest(), [0; 32]);
    serving.close();
}

#[test]
fn denied_before_barrier_effect_returns_the_exact_append_and_retries_canonically() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    super::super::serving_from_initialization(&store_root).close();
    let (media, activation) = media_with_synchronization_fault(
        &store_root,
        MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    );
    let policy = durability_with_group_limit(&media, NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = super::super::success(
        media.open_record_store(PhysicalRecordOpen::new(format, access, policy)),
    );
    let submission = serving.certification_record_submission();
    let appended = append(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([92; 32]),
    );
    let mutation = appended.members()[0].mutation().mutation_identity();
    let member = appended.members()[0].mutation().reserved().member_basis();

    activation.arm().unwrap();
    let preserved = match submission.synchronize_appended_wal_group(appended) {
        PhysicalWalGroupBarrierOutcome::BarrierNotStarted {
            appended,
            cause: PhysicalWalGroupBarrierFailureCause::MediaDeniedBeforeEffect(failure),
        } => {
            assert_eq!(failure.kind(), ArtifactTreeFailureKind::DeniedBeforeEffect);
            appended
        }
        _ => panic!("a pre-effect synchronization denial must preserve the exact append"),
    };
    assert_eq!(
        preserved.members()[0].mutation().mutation_identity(),
        mutation
    );
    assert_eq!(
        preserved.members()[0].mutation().reserved().member_basis(),
        member
    );

    let durable = match submission.synchronize_appended_wal_group(preserved) {
        PhysicalWalGroupBarrierOutcome::Durable(durable) => durable
            .into_members()
            .into_vec()
            .pop()
            .expect("one sealed member derives one durable member"),
        _ => panic!("one-shot barrier denial must retry through the canonical route"),
    };
    assert_eq!(durable.mutation_identity(), mutation);
    assert_eq!(durable.member_basis(), member);
    serving.close();
}

#[test]
fn post_effect_uncertainty_never_mints_wal_durable_authority_or_returns_retry_authority() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    super::super::serving_from_initialization(&store_root).close();
    let (media, activation) = media_with_synchronization_fault(
        &store_root,
        MediaFaultDirective::IndeterminateAfterEffect,
    );
    let policy = durability_with_group_limit(&media, NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = super::super::success(
        media.open_record_store(PhysicalRecordOpen::new(format, access, policy)),
    );
    let submission = serving.certification_record_submission();
    let appended = append(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([93; 32]),
    );
    let basis = appended.basis();

    activation.arm().unwrap();
    let uncertain = match submission.synchronize_appended_wal_group(appended) {
        PhysicalWalGroupBarrierOutcome::Indeterminate(uncertain) => uncertain,
        _ => panic!("post-effect uncertainty must remain inspection-only"),
    };
    assert_eq!(uncertain.basis(), basis);
    assert_eq!(uncertain.member_count(), 1);
    serving.close();
}

pub(super) fn append(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: PhysicalMutationIdempotencyMaterial,
) -> SealedPhysicalDurabilityGroupMembers {
    let prepared = super::wal_append::prepared(submission, placement, material, b"barrier-redo");
    match submission.append_prepared_wal_group(NonEmpty::new(prepared, Vec::new())) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("barrier evidence requires one canonical appended WAL member"),
    }
}

fn required_barrier(profile: BackendTargetProfile) -> WalDurabilityBarrier {
    match profile {
        BackendTargetProfile::PosixFileFsyncDirSync => WalDurabilityBarrier::WalFileFsync,
        BackendTargetProfile::WindowsFlushFileBuffers => {
            WalDurabilityBarrier::WindowsFlushFileBuffers
        }
        _ => panic!("production filesystem admission must select a supported WAL barrier"),
    }
}

fn media_with_synchronization_fault(
    root: &Path,
    directive: MediaFaultDirective,
) -> (
    worth_store::physical_runtime::MediaOwnedPhysicalRuntime,
    CertificationMediaFaultActivation,
) {
    let admission =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let activation = authority.one_shot_activation();
    let schedule = authority
        .schedule(vec![authority
            .rule(MediaOperationRole::SynchronizeFileState, 1, directive)
            .for_next_identified_operation_after_activation(
                activation.clone(),
            )])
        .unwrap();
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        TransitionOutcome::Success(media) => (media, activation),
        _ => panic!("fault-scheduled media admission must succeed"),
    }
}
