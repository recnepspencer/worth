#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::wal_barrier::*;
use {std::num::NonZeroU32, std::path::Path};

use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::certification::MediaFaultDirective};
use {worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalGroupBarrierFailureCause, worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome, worth_store::physical_runtime::SealedPhysicalDurabilityGroupMembers};
use {worth_store_physical_backend::ArtifactTreeFailureKind, worth_store_physical_backend::BackendTargetProfile, worth_store_physical_backend::CertificationMediaFaultActivation, worth_store_physical_backend::FilesystemAccessPosture, worth_store_physical_backend::MediaOperationRole, worth_store_physical_backend::WalDurabilityBarrier};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_with_group_limit};

#[test]
fn exact_appended_member_crosses_the_scheduled_barrier_into_wal_durable_authority() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_initialization(&store_root);
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let appended = append(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([91; 32]),
    );
    let appended_member = appended.members()[0].mutation();
    let mutation = appended_member.mutation_identity();
    let member = appended_member.reserved().member_basis();
    let append_work = appended_member.settlement().work_identity();
    let append_operation = appended_member.settlement().backend_operation();
    let durability = serving.durability_observation();

    let durable = match submission.synchronize_appended_wal_group(appended) {
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(durable) => durable
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
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization(&store_root).close();
    let (media, activation) = media_with_synchronization_fault(
        &store_root,
        worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    );
    let policy = durability_with_group_limit(&media, std::num::NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = worth_store_testkit_store::physical_record_journeys::success(
        media.open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, policy)),
    );
    let submission = serving.certification_record_submission();
    let appended = append(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([92; 32]),
    );
    let mutation = appended.members()[0].mutation().mutation_identity();
    let member = appended.members()[0].mutation().reserved().member_basis();

    activation.arm().unwrap();
    let preserved = match submission.synchronize_appended_wal_group(appended) {
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::BarrierNotStarted {
            appended,
            cause: worth_store::physical_runtime::PhysicalWalGroupBarrierFailureCause::MediaDeniedBeforeEffect(failure),
        } => {
            assert_eq!(failure.kind(), worth_store_physical_backend::ArtifactTreeFailureKind::DeniedBeforeEffect);
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
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(durable) => durable
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
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization(&store_root).close();
    let (media, activation) = media_with_synchronization_fault(
        &store_root,
        worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
    );
    let policy = durability_with_group_limit(&media, std::num::NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = worth_store_testkit_store::physical_record_journeys::success(
        media.open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, policy)),
    );
    let submission = serving.certification_record_submission();
    let appended = append(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([93; 32]),
    );
    let basis = appended.basis();

    activation.arm().unwrap();
    let uncertain = match submission.synchronize_appended_wal_group(appended) {
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Indeterminate(uncertain) => uncertain,
        _ => panic!("post-effect uncertainty must remain inspection-only"),
    };
    assert_eq!(uncertain.basis(), basis);
    assert_eq!(uncertain.member_count(), 1);
    serving.close();
}
