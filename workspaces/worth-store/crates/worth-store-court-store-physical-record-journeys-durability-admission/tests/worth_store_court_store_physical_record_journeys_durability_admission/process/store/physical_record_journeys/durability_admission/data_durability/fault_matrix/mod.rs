#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::data_durability::fault_matrix::*;
use {std::fs};
use {std::num::NonZeroU32};

use {worth_store::physical_runtime::certification::MediaFaultDirective};
use {worth_store::physical_runtime::PhysicalDataDispatchFailureCause, worth_store::physical_runtime::PhysicalDataDispatchOutcome, worth_store::physical_runtime::PhysicalDataEffectSource, worth_store::physical_runtime::PhysicalDataSettlementOutcome, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::RecordAppendBatch};
use {worth_store_physical_backend::MediaOperationRole};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_with_group_limit, worth_store_testkit_store::physical_record_journeys::success};
use {crate::durability_admission::data_durability::mutation_world::append, crate::durability_admission::data_durability::mutation_world::certification_media, crate::durability_admission::data_durability::mutation_world::fault_scheduled_media, crate::durability_admission::data_durability::mutation_world::fault_scheduled_media_at_identified_ordinal, crate::durability_admission::data_durability::mutation_world::synchronize};
use {crate::physical_record_journeys::durability_admission::data_durability::oracle::artifact_path};

#[test]
fn denied_byte_write_after_artifact_creation_is_inspection_only() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let (media, activation) = fault_scheduled_media(
        &store_root,
        1,
        worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    );
    let policy = durability_with_group_limit(&media, std::num::NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        )),
    );
    let submission = serving.certification_record_submission();
    let appended = append(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([111; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"pre-effect-retry".as_slice()]).unwrap(),
    );
    let target =
        appended.members()[0].mutation().reserved().redo().records()[0].targets()[0].target();
    let durable = synchronize(&submission, appended);
    activation.arm().unwrap();
    let uncertain = match submission.dispatch_wal_durable_data(durable) {
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Indeterminate(uncertain) => uncertain,
        _ => panic!("artifact creation before a denied byte write requires inspection"),
    };
    assert_eq!(uncertain.completed_frames(), 0);
    assert!(matches!(
        uncertain.cause(),
        worth_store::physical_runtime::PhysicalDataDispatchFailureCause::Canonical(_)
    ));
    assert!(artifact_path(&store_root, target.coordinate().artifact()).exists());
    serving.close();
}

#[test]
fn uncertainty_after_first_data_effect_returns_inspection_only_authority() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let (media, activation) = fault_scheduled_media(
        &store_root,
        1,
        worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
    );
    let policy = durability_with_group_limit(&media, std::num::NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        )),
    );
    let submission = serving.certification_record_submission();
    let appended = append(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([112; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"post-effect-uncertainty".as_slice()]).unwrap(),
    );
    let appended_member = appended.members()[0].mutation();
    let target = appended_member.reserved().redo().records()[0].targets()[0].target();
    let mutation = appended_member.mutation_identity();
    let durable = synchronize(&submission, appended);
    activation.arm().unwrap();
    let uncertain = match submission.dispatch_wal_durable_data(durable) {
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Indeterminate(uncertain) => uncertain,
        _ => panic!("post-effect uncertainty must never return retry authority"),
    };
    assert_eq!(uncertain.mutation_identity(), mutation);
    assert!(matches!(
        uncertain.cause(),
        worth_store::physical_runtime::PhysicalDataDispatchFailureCause::Canonical(_)
    ));
    assert!(artifact_path(&store_root, target.coordinate().artifact()).exists());
    serving.close();
}

#[test]
fn second_chunk_pre_effect_denial_preserves_and_resumes_the_exact_settled_prefix() {
    let calibration = calibrated_second_arena_write();
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let media = fault_scheduled_media_at_identified_ordinal(
        &store_root,
        calibration.target,
        worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    );
    let policy = durability_with_group_limit(&media, std::num::NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        )),
    );
    let submission = serving.certification_record_submission();
    let payload = vec![0x7c; format.declaration().page_size().bytes() as usize * 3];
    let appended = append(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([113; 32]),
        worth_store::physical_runtime::RecordAppendBatch::builder()
            .push_owned(payload)
            .build()
            .unwrap(),
    );
    let durable = synchronize(&submission, appended);
    assert_eq!(
        serving
            .media_counters()
            .identified_operation_attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        calibration.before_dispatch
    );
    let suspended = match submission.dispatch_wal_durable_data(durable) {
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Suspended(suspended) => suspended,
        _ => panic!("a denied second chunk follows a completed first data effect"),
    };
    assert_eq!(suspended.completed_effects().len(), 1);
    assert_eq!(
        suspended.completed_effects()[0].source(),
        worth_store::physical_runtime::PhysicalDataEffectSource::NewArtifact
    );
    assert!(matches!(
        suspended.cause(),
        worth_store::physical_runtime::PhysicalDataDispatchFailureCause::Canonical(_)
    ));
    let first = suspended.completed_effects()[0].clone();
    let artifact = artifact_path(&store_root, first.coordinate().artifact());
    assert!(std::fs::metadata(artifact).unwrap().len() > 0);
    let before_resume = serving
        .media_counters()
        .identified_operation_attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite);
    let dispatched = match submission.dispatch_wal_durable_data(suspended.into_durable()) {
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::NotStarted { cause, .. } => panic!("resume denied: {cause:?}"),
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Indeterminate(failure) => {
            panic!("resume uncertain: {:?}", failure.cause())
        }
        _ => panic!("one-shot fault must allow the suffix to complete"),
    };
    assert_eq!(
        dispatched.effects()[0],
        first,
        "original effect evidence is retained, not recreated"
    );
    assert_eq!(
        serving
            .media_counters()
            .identified_operation_attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before_resume,
        dispatched.effects().len() as u64 - 1,
        "only the unfinished suffix is written"
    );
    assert!(matches!(
        dispatched.settle_exact_effects(),
        worth_store::physical_runtime::PhysicalDataSettlementOutcome::Settled(_)
    ));
    serving.close();
}
