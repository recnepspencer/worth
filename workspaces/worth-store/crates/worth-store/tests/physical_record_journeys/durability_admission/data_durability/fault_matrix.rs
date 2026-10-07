use std::fs;
use std::num::NonZeroU32;

use worth_store::physical_runtime::certification::MediaFaultDirective;
use worth_store::physical_runtime::{
    PhysicalDataDispatchFailureCause, PhysicalDataDispatchOutcome, PhysicalDataEffectSource,
    PhysicalDataSettlementOutcome, PhysicalMutationIdempotencyMaterial,
    PhysicalRecordInitialization, RecordAppendBatch,
};
use worth_store_physical_backend::MediaOperationRole;

use super::super::super::{configuration, durability_with_group_limit, success};
use super::mutation_world::{
    append, certification_media, fault_scheduled_media,
    fault_scheduled_media_at_identified_ordinal, synchronize,
};
use super::oracle::artifact_path;

#[test]
fn denied_byte_write_after_artifact_creation_is_inspection_only() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let (media, activation) = fault_scheduled_media(
        &store_root,
        1,
        MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    );
    let policy = durability_with_group_limit(&media, NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, policy,
        )),
    );
    let submission = serving.certification_record_submission();
    let appended = append(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([111; 32]),
        RecordAppendBatch::try_from_iter([b"pre-effect-retry".as_slice()]).unwrap(),
    );
    let target =
        appended.members()[0].mutation().reserved().redo().records()[0].targets()[0].target();
    let durable = synchronize(&submission, appended);
    activation.arm().unwrap();
    let uncertain = match submission.dispatch_wal_durable_data(durable) {
        PhysicalDataDispatchOutcome::Indeterminate(uncertain) => uncertain,
        _ => panic!("artifact creation before a denied byte write requires inspection"),
    };
    assert_eq!(uncertain.completed_frames(), 0);
    assert!(matches!(
        uncertain.cause(),
        PhysicalDataDispatchFailureCause::Canonical(_)
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
        MediaFaultDirective::IndeterminateAfterEffect,
    );
    let policy = durability_with_group_limit(&media, NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, policy,
        )),
    );
    let submission = serving.certification_record_submission();
    let appended = append(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([112; 32]),
        RecordAppendBatch::try_from_iter([b"post-effect-uncertainty".as_slice()]).unwrap(),
    );
    let appended_member = appended.members()[0].mutation();
    let target = appended_member.reserved().redo().records()[0].targets()[0].target();
    let mutation = appended_member.mutation_identity();
    let durable = synchronize(&submission, appended);
    activation.arm().unwrap();
    let uncertain = match submission.dispatch_wal_durable_data(durable) {
        PhysicalDataDispatchOutcome::Indeterminate(uncertain) => uncertain,
        _ => panic!("post-effect uncertainty must never return retry authority"),
    };
    assert_eq!(uncertain.mutation_identity(), mutation);
    assert!(matches!(
        uncertain.cause(),
        PhysicalDataDispatchFailureCause::Canonical(_)
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
        MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    );
    let policy = durability_with_group_limit(&media, NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, policy,
        )),
    );
    let submission = serving.certification_record_submission();
    let payload = vec![0x7c; format.declaration().page_size().bytes() as usize * 3];
    let appended = append(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([113; 32]),
        RecordAppendBatch::builder()
            .push_owned(payload)
            .build()
            .unwrap(),
    );
    let durable = synchronize(&submission, appended);
    assert_eq!(
        serving
            .media_counters()
            .identified_operation_attempts_for(MediaOperationRole::PositionedWrite),
        calibration.before_dispatch
    );
    let suspended = match submission.dispatch_wal_durable_data(durable) {
        PhysicalDataDispatchOutcome::Suspended(suspended) => suspended,
        _ => panic!("a denied second chunk follows a completed first data effect"),
    };
    assert_eq!(suspended.completed_effects().len(), 1);
    assert_eq!(
        suspended.completed_effects()[0].source(),
        PhysicalDataEffectSource::NewArtifact
    );
    assert!(matches!(
        suspended.cause(),
        PhysicalDataDispatchFailureCause::Canonical(_)
    ));
    let first = suspended.completed_effects()[0].clone();
    let artifact = artifact_path(&store_root, first.coordinate().artifact());
    assert!(fs::metadata(artifact).unwrap().len() > 0);
    let before_resume = serving
        .media_counters()
        .identified_operation_attempts_for(MediaOperationRole::PositionedWrite);
    let dispatched = match submission.dispatch_wal_durable_data(suspended.into_durable()) {
        PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        PhysicalDataDispatchOutcome::NotStarted { cause, .. } => panic!("resume denied: {cause:?}"),
        PhysicalDataDispatchOutcome::Indeterminate(failure) => {
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
            .identified_operation_attempts_for(MediaOperationRole::PositionedWrite)
            - before_resume,
        dispatched.effects().len() as u64 - 1,
        "only the unfinished suffix is written"
    );
    assert!(matches!(
        dispatched.settle_exact_effects(),
        PhysicalDataSettlementOutcome::Settled(_)
    ));
    serving.close();
}

struct ArenaWriteCalibration {
    before_dispatch: u64,
    target: u64,
}

fn calibrated_second_arena_write() -> ArenaWriteCalibration {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("control");
    let media = certification_media(&store_root);
    let policy = durability_with_group_limit(&media, NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, policy,
        )),
    );
    let submission = serving.certification_record_submission();
    let payload = vec![0x6d; format.declaration().page_size().bytes() as usize * 3];
    let appended = append(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([114; 32]),
        RecordAppendBatch::builder()
            .push_owned(payload)
            .build()
            .unwrap(),
    );
    let durable = synchronize(&submission, appended);
    let before = serving
        .media_counters()
        .identified_operation_attempts_for(MediaOperationRole::PositionedWrite);
    let dispatched = match submission.dispatch_wal_durable_data(durable) {
        PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        _ => panic!("control dispatch must complete"),
    };
    let after = serving
        .media_counters()
        .identified_operation_attempts_for(MediaOperationRole::PositionedWrite);
    let arena_effects = dispatched
        .effects()
        .iter()
        .filter(|effect| effect.source() == PhysicalDataEffectSource::NewArtifact)
        .count() as u64;
    assert_eq!(
        after - before,
        arena_effects,
        "each arena frame has one canonical ranged write"
    );
    assert!(arena_effects > 1);
    assert!(matches!(
        dispatched.settle_exact_effects(),
        PhysicalDataSettlementOutcome::Settled(_)
    ));
    serving.close();
    ArenaWriteCalibration {
        before_dispatch: before,
        target: before + 2,
    }
}
