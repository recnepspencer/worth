#[allow(unused_imports)] use crate::physical_record_journeys::physical_work::residency_pressure_projection::*;
use {std::num::NonZeroU32, std::num::NonZeroU64};
use {std::time::Duration, std::time::Instant};

use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::certification::CertificationDurableMutationInput, worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy, worth_store::physical_runtime::DataDispatchedPhysicalMutation, worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::PhysicalDataDispatchFailureCause, worth_store::physical_runtime::PhysicalDataDispatchOutcome, worth_store::physical_runtime::PhysicalDataEffectSource, worth_store::physical_runtime::PhysicalManifestCapacityTransition, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRecordResidencyPolicy, worth_store::physical_runtime::PhysicalResidencyDimension, worth_store::physical_runtime::PhysicalResidencyRetryPosture, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalSpeculativeWorkKind, worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWorkEffectFate, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordServingTerminalPosture};
use {worth_store_physical_backend::FilesystemAccessPosture, worth_store_physical_backend::MediaFaultDirective, worth_store_physical_backend::MediaOperationRole};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::success};





#[test]
fn inline_writebehind_pressure_preserves_settled_prefix_before_retry() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("inline-writebehind-pressure");
    let (format, placement, access) = configuration();
    let initialized = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    assert!(!initialized.close().residency().requires_inspection());

    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::production(worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                worth_store_physical_backend::MediaOperationRole::PositionedWrite,
                4,
                worth_store_physical_backend::MediaFaultDirective::PauseBefore(gate.clone()),
            )
            .for_identified_operation_ordinal()])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(worth_store::physical_runtime::PhysicalRuntimeAdmission::new(&root).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("writebehind pressure media must admit"),
    };
    let serving = success(open_record_store!(media, |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
            .with_residency_policy(two_append_writebehind_policy(format))
    },));
    let payload = vec![73_u8; format.declaration().page_size().bytes() as usize / 3];
    let residency_before = serving.residency_observation().counters();
    let (group_basis, durable_group) = serving.certification_prepare_wal_durable_group(
        placement,
        worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
        worth_proof::NonEmpty::new(
            worth_store::physical_runtime::certification::CertificationDurableMutationInput::new(
                worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0xA1; 32]),
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload.as_slice(); 8]).unwrap(),
            ),
            vec![worth_store::physical_runtime::certification::CertificationDurableMutationInput::new(
                worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0xA2; 32]),
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload.as_slice(); 8]).unwrap(),
            )],
        ),
    );
    let mut durable_group = durable_group.into_vec().into_iter();
    let primary_durable = durable_group.next().unwrap();
    let denied_durable = durable_group.next().unwrap();
    assert!(durable_group.next().is_none());
    let submission = serving.certification_record_submission();
    let primary = std::thread::spawn(move || submission.dispatch_wal_durable_data(primary_durable));

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while gate.reached_context().is_none() && std::time::Instant::now() < deadline {
        std::thread::yield_now();
    }
    let context = gate
        .reached_context()
        .expect("primary canonical mutation must reach candidate writeback");
    assert_eq!(context.role(), worth_store_physical_backend::MediaOperationRole::PositionedWrite);
    assert_eq!(context.identified_operation_ordinal(), Some(4));
    let residency_at_dispatch = serving.residency_observation().counters();

    let (retry_durable, pressure, first_effect) = match serving
        .certification_record_submission()
        .dispatch_wal_durable_data(denied_durable)
    {
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Suspended(retry) => {
            assert!(!retry.completed_effects().is_empty());
            let worth_store::physical_runtime::PhysicalDataDispatchFailureCause::PhysicalPressure(pressure) = retry.cause() else {
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
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::NotStarted {
            durable,
            cause: worth_store::physical_runtime::PhysicalDataDispatchFailureCause::PhysicalPressure(_),
        } => {
            drop(durable);
            panic!("writebehind pressure denied before settling the first inline page")
        }
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::NotStarted { cause, .. } => {
            panic!("canonical dispatch omitted pressure evidence: {cause:?}")
        }
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Dispatched(_) => {
            panic!("competing canonical dispatch bypassed pressure")
        }
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Indeterminate(indeterminate) => panic!(
            "competing canonical dispatch became indeterminate: {:?}",
            indeterminate.cause()
        ),
    };
    assert_eq!(
        pressure.dimension(),
        worth_store::physical_runtime::PhysicalResidencyDimension::SpeculativeFrames(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind)
    );
    assert_eq!(pressure.basis().store_identity(), serving.store_identity());
    assert!(pressure.basis().frame_coordinate().is_some());
    assert_eq!(pressure.requested(), 1);
    assert_eq!(pressure.admitted(), 1);
    assert_eq!(pressure.limit(), 1);
    assert_eq!(
        pressure.retry_posture(),
        worth_store::physical_runtime::PhysicalResidencyRetryPosture::AfterWritebackSettlement
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
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Suspended(_) => {
            panic!("primary canonical dispatch unexpectedly suspended")
        }
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::NotStarted { cause, .. } => {
            panic!("primary canonical dispatch did not start: {cause:?}")
        }
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Indeterminate(indeterminate) => panic!(
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
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Suspended(_) => {
            panic!("canonical retry suspended after writeback capacity was released")
        }
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::NotStarted { cause, .. } => {
            panic!("canonical retry did not start: {cause:?}")
        }
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Indeterminate(indeterminate) => panic!(
            "canonical retry became indeterminate: {:?}",
            indeterminate.cause()
        ),
    };
    let retry_writebacks = assert_inline_candidate_trace(&retry_dispatched);
    assert_eq!(retry_dispatched.effects()[0], first_effect);
    let counters = serving.residency_observation().counters();
    let completed = serving.certification_complete_dispatched_group(
        group_basis,
        worth_proof::NonEmpty::new(primary_dispatched, vec![retry_dispatched]),
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
        worth_store::physical_runtime::RecordServingTerminalPosture::NoInspectionRequired
    );
    assert!(!close.residency().requires_inspection());
}
#[path = "arena_prefix_retry/mod.rs"]
pub(crate) mod arena_prefix_retry;
#[path = "public_pre_effect_basis/mod.rs"]
pub(crate) mod public_pre_effect_basis;
