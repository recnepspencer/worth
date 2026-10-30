pub(crate) use std::fs;
pub(crate) use std::num::NonZeroU32;

pub(crate) use worth_store::physical_runtime::certification::MediaFaultDirective;
pub(crate) use worth_store_physical_backend::MediaOperationRole;
pub(crate) use {
    worth_store::physical_runtime::PhysicalDataDispatchFailureCause,
    worth_store::physical_runtime::PhysicalDataDispatchOutcome,
    worth_store::physical_runtime::PhysicalDataEffectSource,
    worth_store::physical_runtime::PhysicalDataSettlementOutcome,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::RecordAppendBatch,
};

pub(crate) use crate::oracle::support::data_durability::artifact_path;
pub(crate) use {
    crate::durability_admission::data_durability::mutation_world::append,
    crate::durability_admission::data_durability::mutation_world::certification_media,
    crate::durability_admission::data_durability::mutation_world::fault_scheduled_media,
    crate::durability_admission::data_durability::mutation_world::fault_scheduled_media_at_identified_ordinal,
    crate::durability_admission::data_durability::mutation_world::synchronize,
};
pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::durability_with_group_limit,
    worth_store_testkit_store::physical_record_journeys::success,
};

pub(crate) struct ArenaWriteCalibration {
    pub(crate) before_dispatch: u64,
    pub(crate) target: u64,
}

pub(crate) fn calibrated_second_arena_write() -> ArenaWriteCalibration {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("control");
    let media = certification_media(&store_root);
    let policy = durability_with_group_limit(&media, std::num::NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = success(media.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        ),
    ));
    let submission = serving.certification_record_submission();
    let payload = vec![0x6d; format.declaration().page_size().bytes() as usize * 3];
    let appended = append(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([114; 32]),
        worth_store::physical_runtime::RecordAppendBatch::builder()
            .push_owned(payload)
            .build()
            .unwrap(),
    );
    let durable = synchronize(&submission, appended);
    let before = serving.media_counters().identified_operation_attempts_for(
        worth_store_physical_backend::MediaOperationRole::PositionedWrite,
    );
    let dispatched = match submission.dispatch_wal_durable_data(durable) {
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Dispatched(dispatched) => {
            dispatched
        }
        _ => panic!("control dispatch must complete"),
    };
    let after = serving.media_counters().identified_operation_attempts_for(
        worth_store_physical_backend::MediaOperationRole::PositionedWrite,
    );
    let arena_effects = dispatched
        .effects()
        .iter()
        .filter(|effect| {
            effect.source() == worth_store::physical_runtime::PhysicalDataEffectSource::NewArtifact
        })
        .count() as u64;
    assert_eq!(
        after - before,
        arena_effects,
        "each arena frame has one canonical ranged write"
    );
    assert!(arena_effects > 1);
    assert!(matches!(
        dispatched.settle_exact_effects(),
        worth_store::physical_runtime::PhysicalDataSettlementOutcome::Settled(_)
    ));
    serving.close();
    ArenaWriteCalibration {
        before_dispatch: before,
        target: before + 2,
    }
}
