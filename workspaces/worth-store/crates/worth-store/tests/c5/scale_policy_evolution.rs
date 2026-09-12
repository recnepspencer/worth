use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    ExternalPhysicalRecordLocator, PhysicalManifestCapacityTransition,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationPreparationDenial,
    PhysicalRecordInitialization, PhysicalRecordOpen, RecordAppendBatch, RecordAppendDenial,
};
use worth_store_physical_backend::MediaOperationRole;

use crate::durable_publication::{
    prepare_single, publish_single, publish_single_with_manifest_capacity_transition,
};

use super::scale_support::{access, assert_canonical_parity, complete_scan, format, placement};
use super::{media, success};

pub(super) fn prove() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("policy");
    let format = format();
    let initial = placement(format, 4, 2, 50);
    let admitted_access = access(format, 11);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, initial, admitted_access, durability)
    }));
    let first = publish_single(
        &serving,
        initial,
        PhysicalMutationIdempotencyMaterial::new([171; 32]),
        RecordAppendBatch::try_from_iter((0..25).map(|value| vec![value; 600])).unwrap(),
    );
    let stable = first.settled_members()[0].record_id(0).unwrap();
    let locator = ExternalPhysicalRecordLocator::new(serving.store_identity(), stable);

    let wider = placement(format, 8, 3, 75);
    let writes_before = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    assert!(matches!(
        prepare_single(
            &serving.record_submission(),
            wider,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            PhysicalMutationIdempotencyMaterial::new([172; 32]),
            RecordAppendBatch::try_from_iter([b"cost-visible migration".as_slice()]).unwrap(),
        )
        .into_raw(),
        TransitionOutcome::Denied(PhysicalMutationPreparationDenial::RecordAppend(
            RecordAppendDenial::ManifestCapacityMigrationRequired
        ))
    ));
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        writes_before
    );
    publish_single_with_manifest_capacity_transition(
        &serving,
        wider,
        PhysicalManifestCapacityTransition::ReconstructToRequested,
        PhysicalMutationIdempotencyMaterial::new([173; 32]),
        RecordAppendBatch::try_from_iter((0..5).map(|value| vec![0x80 + value; 900])).unwrap(),
    );
    let wide = worth_store_offline_verifier::walk_current_durable_record_manifest(
        &root,
        format.declaration(),
    )
    .unwrap();
    assert_eq!(wide.node_capacity(), 8);

    let narrower = placement(format, 2, 1, 60);
    publish_single_with_manifest_capacity_transition(
        &serving,
        narrower,
        PhysicalManifestCapacityTransition::ReconstructToRequested,
        PhysicalMutationIdempotencyMaterial::new([174; 32]),
        RecordAppendBatch::try_from_iter((0..5).map(|value| vec![0xc0 + value; 700])).unwrap(),
    );
    assert_eq!(
        serving
            .records()
            .expect("read protection admission")
            .readmit_locator(locator)
            .into_result()
            .unwrap(),
        stable
    );
    let narrow = worth_store_offline_verifier::walk_current_durable_record_manifest(
        &root,
        format.declaration(),
    )
    .unwrap();
    assert_eq!(narrow.node_capacity(), 2);
    assert_eq!(narrow.placements().len(), 35);
    assert_canonical_parity(&serving, &narrow);
    let scan = complete_scan(&serving, 11, 131_072);
    assert_eq!(scan.records(), 35);
    serving.close();

    let changed_access = access(format, 3);
    let reopened = success(open_record_store!(media(&root), |durability| {
        PhysicalRecordOpen::new(format, changed_access, durability)
    }));
    assert_eq!(
        reopened
            .records()
            .expect("read protection admission")
            .readmit_locator(locator)
            .into_result()
            .unwrap(),
        stable
    );
    reopened.close();
}
