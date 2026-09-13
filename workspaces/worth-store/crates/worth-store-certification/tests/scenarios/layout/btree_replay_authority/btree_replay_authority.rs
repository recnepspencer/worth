use worth_store_budgets::PreExecutionBudgetEnvelope;
use worth_store_layout_indexes::{
    layout_btree_recovery, BTreeReplayDenied, BTreeReplayLocation, BTreeReplayPhysicalSource,
    BTreeReplayRequest, BaselineBTreeExecutionDenial,
};
use worth_store_physical_format::{
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId, PhysicalRecordSlot,
    PhysicalReferenceAuthority, PhysicalSegmentId, PhysicalStoreIdentity,
};
use worth_store_security::{
    admitted_store_managed_root_security_scope_for_layout_partition_test,
    admitted_tenant_page_security_scope_for_layout_partition_test,
};
use worth_store_test_support::harness::recovery::deterministic_checkpoint_plus_tail_source;
use worth_store_test_support::{
    admitted_layout_bootstrap_catalog, deterministic_btree_replay_world,
};

#[test]
fn ordinary_recovery_facade_reopens_and_validates_btree_state() {
    let catalog = admitted_layout_bootstrap_catalog();
    let security = admitted_tenant_page_security_scope_for_layout_partition_test();
    let world = deterministic_btree_replay_world();

    let recovered = layout_btree_recovery()
        .replay(BTreeReplayRequest::new(
            &catalog,
            security.witnesses(),
            location(),
            PreExecutionBudgetEnvelope::maintenance_default(),
            physical_source(&world, world.root_reference()),
        ))
        .into_result()
        .unwrap();

    assert!(recovered.replay_generation_monotonic());
    assert!(recovered.manifest_advanced());
    assert!(recovered.rebuild_source_authoritative());
    assert_eq!(recovered.rebuild_authority_records(), 4);
    assert_eq!(recovered.rebuild_output_records(), 4);
    let counters = recovered.exact_counters();
    assert_eq!(counters.wal_replays(), 1);
    assert_eq!(counters.maintenance_reads(), 3);
    assert_eq!(counters.page_touches(), 3);
    assert_eq!(counters.index_probes(), 6);
    assert_eq!(counters.key_comparisons(), 6);
    assert_eq!(counters.manifest_reads(), 1);
    assert_eq!(counters.bytes_read(), 12_288);
    assert_eq!(counters.read_amplification(), 3);
    assert_eq!(recovered.recovery_source_digest().len(), 64);
}

#[test]
fn copied_or_stale_root_reference_is_rejected_by_recovery_source_admission() {
    let catalog = admitted_layout_bootstrap_catalog();
    let security = admitted_tenant_page_security_scope_for_layout_partition_test();
    let world = deterministic_btree_replay_world();
    let stale_cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .slot_cell(
            PhysicalSegmentId::from_raw(7).unwrap(),
            PhysicalPageId::from_raw(9).unwrap(),
            PhysicalRecordSlot::from_raw(1).unwrap(),
        )
        .with_slot_generation(PhysicalGeneration::from_raw(999).unwrap());
    let stale = PhysicalReferenceAuthority::for_canonical_physical_format()
        .admit_page_slot(stale_cell)
        .reference();

    let outcome = layout_btree_recovery()
        .replay(BTreeReplayRequest::new(
            &catalog,
            security.witnesses(),
            location(),
            PreExecutionBudgetEnvelope::maintenance_default(),
            physical_source(&world, stale),
        ))
        .into_result();

    assert!(matches!(outcome, Err(BTreeReplayDenied::Execution(_))));
}

#[test]
fn store_internal_security_scope_cannot_select_tenant_btree_replay() {
    let catalog = admitted_layout_bootstrap_catalog();
    let security = admitted_store_managed_root_security_scope_for_layout_partition_test();
    let world = deterministic_btree_replay_world();

    let outcome = layout_btree_recovery()
        .replay(BTreeReplayRequest::new(
            &catalog,
            security.witnesses(),
            location(),
            PreExecutionBudgetEnvelope::maintenance_default(),
            physical_source(&world, world.root_reference()),
        ))
        .into_result();

    assert_eq!(outcome, Err(BTreeReplayDenied::SecurityScope));
}

#[test]
fn replay_artifact_from_another_store_instance_is_rejected() {
    let catalog = admitted_layout_bootstrap_catalog();
    let security = admitted_tenant_page_security_scope_for_layout_partition_test();
    let world = deterministic_btree_replay_world();
    let key = worth_foundational::aspects()
        .vocabulary()
        .key("store.physical.foreign_instance")
        .unwrap();
    let foreign = PhysicalStoreIdentity::from_aspect_identity(
        worth_store_aspect_native::StoreAspectIdentity::from_aspect_key(key),
    );

    let outcome = layout_btree_recovery()
        .replay(BTreeReplayRequest::new(
            &catalog,
            security.witnesses(),
            location(),
            PreExecutionBudgetEnvelope::maintenance_default(),
            BTreeReplayPhysicalSource::new(
                world.readiness().clone(),
                world.root_reference(),
                world.replay_artifact().clone(),
                foreign,
                deterministic_checkpoint_plus_tail_source(),
            ),
        ))
        .into_result();

    assert!(matches!(outcome, Err(BTreeReplayDenied::Execution(_))));
}

#[test]
fn checkpoint_for_copied_root_generation_cannot_authorize_current_root() {
    let catalog = admitted_layout_bootstrap_catalog();
    let security = admitted_tenant_page_security_scope_for_layout_partition_test();
    let world = deterministic_btree_replay_world();
    let copied_generation = PhysicalGenerationAuthority::for_canonical_physical_format()
        .slot_cell(
            PhysicalSegmentId::from_raw(7).unwrap(),
            PhysicalPageId::from_raw(9).unwrap(),
            PhysicalRecordSlot::from_raw(1).unwrap(),
        )
        .with_slot_generation(PhysicalGeneration::from_raw(999).unwrap());
    let copied_root = PhysicalReferenceAuthority::for_canonical_physical_format()
        .admit_page_slot(copied_generation)
        .reference();

    let outcome = layout_btree_recovery()
        .replay(BTreeReplayRequest::new(
            &catalog,
            security.witnesses(),
            location(),
            PreExecutionBudgetEnvelope::maintenance_default(),
            BTreeReplayPhysicalSource::new(
                world.readiness().clone(),
                copied_root,
                world.replay_artifact().clone(),
                world.replay_artifact().store_identity().clone(),
                deterministic_checkpoint_plus_tail_source(),
            ),
        ))
        .into_result();

    assert!(matches!(
        outcome,
        Err(BTreeReplayDenied::Execution(denial))
            if matches!(
                denial.as_ref(),
                BaselineBTreeExecutionDenial::Recovery(recovery)
                    if matches!(
                        recovery.as_ref(),
                        worth_store_layout_indexes::BTreeReplaySourceDenial::PhysicalOpen(_)
                    )
            )
    ));
}

fn location() -> BTreeReplayLocation {
    BTreeReplayLocation::new(
        PhysicalSegmentId::from_raw(7).unwrap(),
        PhysicalPageId::from_raw(9).unwrap(),
    )
}

fn physical_source(
    world: &worth_store_test_support::DeterministicBTreeReplayWorld,
    root: worth_store_physical_format::PhysicalReference,
) -> BTreeReplayPhysicalSource {
    BTreeReplayPhysicalSource::new(
        world.readiness().clone(),
        root,
        world.replay_artifact().clone(),
        world.replay_artifact().store_identity().clone(),
        deterministic_checkpoint_plus_tail_source(),
    )
}
