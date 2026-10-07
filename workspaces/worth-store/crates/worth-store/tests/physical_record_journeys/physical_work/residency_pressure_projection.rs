use std::num::{NonZeroU32, NonZeroU64};
use std::time::{Duration, Instant};

use worth_proof::{NonEmpty, TransitionOutcome};
use worth_store::physical_runtime::{
    certification::CertificationDurableMutationInput, AdmittedPhysicalRecordFormat,
    AdmittedPhysicalRecordResidencyPolicy, DataDispatchedPhysicalMutation,
    FilesystemMediaAdmission, PhysicalDataDispatchFailureCause, PhysicalDataDispatchOutcome,
    PhysicalDataEffectSource, PhysicalManifestCapacityTransition,
    PhysicalMutationIdempotencyMaterial, PhysicalOperationAllocationScope,
    PhysicalRecordInitialization, PhysicalRecordOpen, PhysicalRecordResidencyPolicy,
    PhysicalResidencyDimension, PhysicalResidencyRetryPosture, PhysicalRuntimeAdmission,
    PhysicalSpeculativeWorkKind, PhysicalStore, PhysicalWorkEffectFate, RecordAppendBatch,
    RecordServingTerminalPosture,
};
use worth_store_physical_backend::{
    FilesystemAccessPosture, MediaFaultDirective, MediaOperationRole,
};

use super::{configuration, media, success};

#[path = "residency_pressure_projection/public_pre_effect_basis.rs"]
mod public_pre_effect_basis;

#[path = "residency_pressure_projection/arena_prefix_retry.rs"]
mod arena_prefix_retry;

#[test]
fn inline_writebehind_pressure_preserves_settled_prefix_before_retry() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("inline-writebehind-pressure");
    let (format, placement, access) = configuration();
    let initialized = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    assert!(!initialized.close().residency().requires_inspection());

    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                MediaOperationRole::PositionedWrite,
                4,
                MediaFaultDirective::PauseBefore(gate.clone()),
            )
            .for_identified_operation_ordinal()])
        .unwrap();
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(&root).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("writebehind pressure media must admit"),
    };
    let serving = success(open_record_store!(media, |durability| {
        PhysicalRecordOpen::new(format, access, durability)
            .with_residency_policy(two_append_writebehind_policy(format))
    },));
    let payload = vec![73_u8; format.declaration().page_size().bytes() as usize / 3];
    let residency_before = serving.residency_observation().counters();
    let (group_basis, durable_group) = serving.certification_prepare_wal_durable_group(
        placement,
        PhysicalManifestCapacityTransition::PreserveCurrent,
        NonEmpty::new(
            CertificationDurableMutationInput::new(
                PhysicalMutationIdempotencyMaterial::new([0xA1; 32]),
                RecordAppendBatch::try_from_iter([payload.as_slice(); 8]).unwrap(),
            ),
            vec![CertificationDurableMutationInput::new(
                PhysicalMutationIdempotencyMaterial::new([0xA2; 32]),
                RecordAppendBatch::try_from_iter([payload.as_slice(); 8]).unwrap(),
            )],
        ),
    );
    let mut durable_group = durable_group.into_vec().into_iter();
    let primary_durable = durable_group.next().unwrap();
    let denied_durable = durable_group.next().unwrap();
    assert!(durable_group.next().is_none());
    let submission = serving.certification_record_submission();
    let primary = std::thread::spawn(move || submission.dispatch_wal_durable_data(primary_durable));

    let deadline = Instant::now() + Duration::from_secs(5);
    while gate.reached_context().is_none() && Instant::now() < deadline {
        std::thread::yield_now();
    }
    let context = gate
        .reached_context()
        .expect("primary canonical mutation must reach candidate writeback");
    assert_eq!(context.role(), MediaOperationRole::PositionedWrite);
    assert_eq!(context.identified_operation_ordinal(), Some(4));
    let residency_at_dispatch = serving.residency_observation().counters();

    let (retry_durable, pressure, first_effect) = match serving
        .certification_record_submission()
        .dispatch_wal_durable_data(denied_durable)
    {
        PhysicalDataDispatchOutcome::Suspended(retry) => {
            assert!(!retry.completed_effects().is_empty());
            let PhysicalDataDispatchFailureCause::PhysicalPressure(pressure) = retry.cause() else {
                gate.release();
                panic!(
                    "suspended dispatch must retain exact pressure evidence: {:?}",
                    retry.cause()
                );
            };
            let pressure = *pressure;
            let first = retry.completed_effects()[0].clone();
            (retry.into_durable(), pressure, first)
        }
        PhysicalDataDispatchOutcome::NotStarted {
            durable,
            cause: PhysicalDataDispatchFailureCause::PhysicalPressure(_),
        } => {
            drop(durable);
            panic!("writebehind pressure denied before settling the first inline page")
        }
        PhysicalDataDispatchOutcome::NotStarted { cause, .. } => {
            panic!("canonical dispatch omitted pressure evidence: {cause:?}")
        }
        PhysicalDataDispatchOutcome::Dispatched(_) => {
            panic!("competing canonical dispatch bypassed pressure")
        }
        PhysicalDataDispatchOutcome::Indeterminate(indeterminate) => panic!(
            "competing canonical dispatch became indeterminate: {:?}",
            indeterminate.cause()
        ),
    };
    assert_eq!(
        pressure.dimension(),
        PhysicalResidencyDimension::SpeculativeFrames(PhysicalSpeculativeWorkKind::WriteBehind)
    );
    assert_eq!(pressure.basis().store_identity(), serving.store_identity());
    assert!(pressure.basis().frame_coordinate().is_some());
    assert_eq!(pressure.requested(), 1);
    assert_eq!(pressure.admitted(), 1);
    assert_eq!(pressure.limit(), 1);
    assert_eq!(
        pressure.retry_posture(),
        PhysicalResidencyRetryPosture::AfterWritebackSettlement
    );
    assert!(!pressure.effect_may_have_started());
    // Media deletion counters also include successful work-obligation cleanup.
    // The retained segment itself must remain present for suffix writeback.
    let segment_path = root
        .join("families/records/segments")
        .join(first_effect.coordinate().artifact().file_name());
    assert!(std::fs::metadata(segment_path).unwrap().len() > 0);
    assert!(serving.publication_residue().is_empty());
    assert!(!serving.observed_non_authoritative_residue());
    let residency_after_denial = serving.residency_observation().counters();

    gate.release();
    let primary_dispatched = match primary.join().unwrap() {
        PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        PhysicalDataDispatchOutcome::Suspended(_) => {
            panic!("primary canonical dispatch unexpectedly suspended")
        }
        PhysicalDataDispatchOutcome::NotStarted { cause, .. } => {
            panic!("primary canonical dispatch did not start: {cause:?}")
        }
        PhysicalDataDispatchOutcome::Indeterminate(indeterminate) => panic!(
            "primary canonical dispatch became indeterminate: {:?}",
            indeterminate.cause()
        ),
    };
    let primary_writebacks = assert_inline_candidate_trace(&primary_dispatched);
    let residency_after_primary = serving.residency_observation().counters();
    let retry_dispatched = match serving
        .certification_record_submission()
        .dispatch_wal_durable_data(retry_durable)
    {
        PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        PhysicalDataDispatchOutcome::Suspended(_) => {
            panic!("canonical retry suspended after writeback capacity was released")
        }
        PhysicalDataDispatchOutcome::NotStarted { cause, .. } => {
            panic!("canonical retry did not start: {cause:?}")
        }
        PhysicalDataDispatchOutcome::Indeterminate(indeterminate) => panic!(
            "canonical retry became indeterminate: {:?}",
            indeterminate.cause()
        ),
    };
    let retry_writebacks = assert_inline_candidate_trace(&retry_dispatched);
    assert_eq!(retry_dispatched.effects()[0], first_effect);
    let counters = serving.residency_observation().counters();
    let completed = serving.certification_complete_dispatched_group(
        group_basis,
        NonEmpty::new(primary_dispatched, vec![retry_dispatched]),
    );
    assert_eq!(completed.settled_members().len(), 2);
    assert!(completed
        .settled_members()
        .iter()
        .all(|member| member.persisted_records().len() == 8));
    let successful_writebacks = primary_writebacks + retry_writebacks;
    assert_eq!(counters.writebacks(), successful_writebacks);
    let primary_candidate_publications = residency_at_dispatch
        .candidate_publications()
        .saturating_sub(residency_before.candidate_publications())
        .saturating_add(
            residency_after_primary
                .candidate_publications()
                .saturating_sub(residency_after_denial.candidate_publications()),
        );
    let denied_candidate_publications = residency_after_denial
        .candidate_publications()
        .saturating_sub(residency_at_dispatch.candidate_publications());
    let retry_candidate_publications = counters
        .candidate_publications()
        .saturating_sub(residency_after_primary.candidate_publications());
    assert_eq!(denied_candidate_publications, 1);
    assert!(primary_candidate_publications > primary_writebacks);
    assert_eq!(retry_candidate_publications, retry_writebacks);
    assert_eq!(
        counters.candidate_publications(),
        primary_candidate_publications
            + retry_candidate_publications
            + denied_candidate_publications
    );
    let close = serving.close();
    assert_eq!(
        close.records().posture(),
        RecordServingTerminalPosture::NoInspectionRequired
    );
    assert!(!close.residency().requires_inspection());
}

fn assert_inline_candidate_trace(dispatched: &DataDispatchedPhysicalMutation) -> u64 {
    let fates = dispatched
        .effects()
        .iter()
        .map(|effect| (effect.source(), effect.effect_fate()))
        .collect::<Vec<_>>();
    assert_eq!(
        fates
            .iter()
            .filter(|(source, fate)| {
                *source == PhysicalDataEffectSource::NewArtifact
                    && *fate == PhysicalWorkEffectFate::PublicationCompleted
            })
            .count(),
        1,
        "an inline batch creates exactly one segment candidate artifact"
    );
    let writebacks = fates
        .iter()
        .filter(|(source, fate)| {
            *source == PhysicalDataEffectSource::ExistingArtifactWriteback
                && *fate == PhysicalWorkEffectFate::WriteCompleted
        })
        .count() as u64;
    assert!(
        writebacks > 0,
        "the multi-page inline segment must require writeback"
    );
    assert_eq!(fates.len() as u64, writebacks + 1);
    writebacks
}

fn two_append_writebehind_policy(
    format: AdmittedPhysicalRecordFormat,
) -> AdmittedPhysicalRecordResidencyPolicy {
    use PhysicalOperationAllocationScope as Scope;
    use PhysicalSpeculativeWorkKind as Kind;

    let page = u64::from(format.declaration().page_size().bytes());
    let operation = page * 512;
    let metadata = page * 2;
    let resident = page * 4;
    let mut builder = PhysicalRecordResidencyPolicy::builder()
        .total_bytes(nonzero(operation + metadata + resident))
        .resident_bytes(nonzero(resident))
        .metadata_bytes(nonzero(metadata))
        .frame_entries(nonzero_count(12))
        .pinned_frames(nonzero_count(4))
        .pin_leases(nonzero_count(6))
        .dirty_frames(nonzero_count(2))
        .dirty_replacement_bytes(nonzero(resident))
        .operation_bytes(nonzero(operation));
    for scope in [
        Scope::ForegroundRead,
        Scope::ForegroundWrite,
        Scope::Recovery,
        Scope::Scrub,
        Scope::Maintenance,
        Scope::Verification,
        Scope::Blob,
    ] {
        builder = builder.scope_bytes(scope, nonzero(operation));
    }
    for (kind, frames) in [
        (Kind::ReadAhead, 2),
        (Kind::Prefetch, 2),
        (Kind::WriteBehind, 1),
    ] {
        builder = builder.speculative_frames(kind, nonzero_count(frames));
    }
    builder.admit(format).into_result().unwrap()
}

fn nonzero(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).unwrap()
}

fn nonzero_count(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap()
}
