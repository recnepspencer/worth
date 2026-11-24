use crate::facade::config::{MvccConfig, RetentionBackend, SnapshotReleasePolicy};
use crate::tests::support::*;

#[test]
fn observed_storage_blocks_reclaim_until_edition_is_released() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::AiWorkflow);
    let create_outcome = create_entity_outcome(execution, &runtime, "first");
    let entity_a = changed_entities(&create_outcome)[0];
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&create_outcome.snapshot)
        .is_ok());
    let delete_outcome = delete_entity(execution, &runtime, entity_a);
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&delete_outcome.snapshot)
        .is_ok());
    let edition = runtime.storage_access().current_edition();
    let denied = runtime.retention().try_run_pass(execution).unwrap_err();
    assert_eq!(
        denied.stop(),
        crate::runtime::RelationalRetentionPassStop::StorageObserved
    );
    assert!(denied.progress().slots_scanned() > 0);
    assert_eq!(denied.progress().outcome().entity_reclaimed, 0);
    drop(edition);

    let retention = runtime.retention().run_pass(execution).unwrap();
    let entity_b = create_entity(execution, &runtime, "second");

    assert!(retention.entity_reclaimed <= 1);
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats(execution)
            .unwrap()
            .reusable_entity_slots,
        0
    );
    assert_eq!(entity_a.local_slot, entity_b.local_slot);
    assert!(entity_b.generation.0 > entity_a.generation.0);
}

#[test]
fn snapshots_resolve_historical_entity_aspects_by_version() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let create_outcome = create_entity_outcome(execution, &runtime, "before");
    let entity = changed_entities(&create_outcome)[0];
    let snapshot = runtime
        .visibility_authority()
        .snapshot()
        .expect("current snapshot fixture is admitted");
    let update_outcome = update_entity(execution, &runtime, entity, "after");

    let old_read = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .expect("read request remains live")
        .unwrap();
    let current_read = runtime
        .read_truth()
        .read_snapshot(&update_outcome.snapshot, execution)
        .expect("read request remains live")
        .unwrap();
    let version_read = runtime
        .read_truth()
        .read_version(execution, create_outcome.version_id)
        .expect("read request remains live");

    assert_eq!(
        read_entity_name(old_read.get_entity(entity).unwrap()),
        Some("before".into())
    );
    assert_eq!(
        read_entity_name(current_read.get_entity(entity).unwrap()),
        Some("after".into())
    );
    assert_eq!(
        read_entity_name(version_read.get_entity(entity).unwrap()),
        Some("before".into())
    );
}

#[test]
fn historical_reads_preserve_generation_and_aspects_after_slot_reuse() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::AiWorkflow);
    let created = create_entity_outcome(execution, &runtime, "before");
    let original = changed_entities(&created)[0];
    let deleted = delete_entity(execution, &runtime, original);
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&deleted.snapshot)
        .is_ok());
    let _ = runtime.retention().run_pass(execution).unwrap();
    let replacement = create_entity(execution, &runtime, "after");

    let historical = runtime
        .read_truth()
        .read_version(execution, created.version_id)
        .expect("read request remains live");
    let record = historical.get_entity(original).unwrap();

    assert_eq!(record.entity_id, original);
    assert_eq!(read_entity_name(record), Some("before".into()));
    assert_eq!(original.local_slot, replacement.local_slot);
    assert!(replacement.generation.0 > original.generation.0);
}

mod read_record_metadata;

#[path = "snapshot_and_retention/cold_source_support.rs"]
mod cold_source_support;
#[path = "snapshot_and_retention/epoch.rs"]
mod epoch;

#[derive(Default)]
struct ColdRetentionCompanion {
    slots: std::sync::Mutex<
        std::collections::BTreeMap<
            crate::facade::history::BranchId,
            crate::facade::mvcc::CompanionBranchCellSlot<
                crate::facade::runtime::ManagedIndexBranchImage,
            >,
        >,
    >,
}
impl std::fmt::Debug for ColdRetentionCompanion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ColdRetentionCompanion")
    }
}
impl crate::facade::mvcc::RelationalPublicationCompanion for ColdRetentionCompanion {
    fn prepare_managed_source_placement(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &crate::facade::mvcc::ManagedIndexSourceCellPlacement<'_>,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, crate::facade::mvcc::CompanionPreflightStop>
    {
        execution.consult().map_err(|cause| {
            crate::facade::mvcc::CompanionPreflightStop::Execution(cause.into())
        })?;
        self.slots
            .lock()
            .unwrap()
            .insert(target.branch().clone(), target.lookup_slot());
        Ok(Box::new(()))
    }

    fn prepare_managed_source(
        &self,
        _: &worth_execution::ExecutionRequest<'_, '_>,
        source: &crate::facade::runtime::ManagedIndexSourcePreparation<'_>,
    ) -> Result<
        Vec<crate::facade::runtime::PreparedManagedIndexSourceRoot>,
        crate::facade::runtime::ManagedIndexSourcePreparationDenial,
    > {
        let mut roots = Vec::new();
        for scope in source.scopes() {
            let proposed = if scope.scope_exists()? {
                let rows = scope.proposed();
                let read = rows.acquire_all()?;
                let ids: Vec<_> = rows
                    .occurrences(&read)?
                    .iter()
                    .map(|o| o.entity())
                    .collect();
                Some(std::sync::Arc::new(ids) as std::sync::Arc<dyn std::any::Any + Send + Sync>)
            } else {
                None
            };
            roots.push(scope.seal(None, proposed, Default::default()));
        }
        Ok(roots)
    }

    fn prepare(
        &self,
        ctx: &mut crate::facade::mvcc::PublicationCompanionPreflight<'_>,
    ) -> Result<
        crate::facade::mvcc::PreparedPublicationCompanionEffect,
        crate::facade::mvcc::CompanionPreflightStop,
    > {
        let mark = ctx
            .mint_selected_branch_cell(std::sync::Arc::new(()))?
            .reserve_preflight(ctx)?;
        let slot = self.slots.lock().unwrap().get(ctx.branch_id()).cloned();
        let source = match slot.as_ref().and_then(|s| s.admitted()) {
            Some(cell) => cell.reserve_preflight(ctx)?,
            None => match slot
                .as_ref()
                .map(|s| s.pending_for_preflight(ctx))
                .transpose()?
                .flatten()
            {
                Some(cell) => cell.reserve_preflight(ctx)?,
                None => {
                    let cell = ctx.mint_managed_index_source_cell()?;
                    self.slots
                        .lock()
                        .unwrap()
                        .insert(ctx.branch_id().clone(), cell.lookup_slot());
                    cell.reserve_preflight(ctx)?
                }
            },
        };
        let image = ctx.prepare_managed_source_image(&source)?;
        ctx.seal_managed_source_replacement(mark, std::sync::Arc::new(()), source, image)
    }
}

#[test]
fn readmitted_cold_source_refill_succeeds_at_exact_observation_retention_ceiling() {
    use crate::facade::{
        branch::RelationalBranchBasisDenial as BasisDenial, identity::EntityId,
        mvcc::RelationalTransactionIntent, runtime::*, schema::RelationalSchemaRegistry,
    };
    use cold_source_support::{advance_batch, cold_source_schema, seed_batch};

    let (reg, program) = cold_source_schema();
    let install = |r: RelationalSchemaRegistry, p: ManagedIndexSourceProgram| {
        let mut rt = RelationalRuntimeApi::builder()
            .schema_registry(r)
            .managed_index_source(p)
            .build()
            .unwrap();
        let rc = rt
            .prepare_initial_schema_installation()
            .unwrap()
            .install_with_managed_sources(RelationalSchemaRegistry::new(), vec![], vec![])
            .unwrap();
        (rt, rc)
    };
    let (runtime, receipt) = install(reg.clone(), program.clone());
    let _reg = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(std::sync::Arc::new(ColdRetentionCompanion::default()))
        .unwrap();
    let host_request = crate::test_execution_policy::serial_request();
    let exec = worth_execution::ExecutionRequest::serial(&host_request);
    let (ca, cx, batch) = seed_batch();
    let born = {
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let mut tx = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        tx.push_batch(batch, exec.allocation_policy()).unwrap();
        tx.commit(&exec, &runtime).unwrap()
    };
    let (a, x) = (
        born.created_entity(&ca).unwrap(),
        born.created_entity(&cx).unwrap(),
    );
    let ckpt = runtime
        .durability_authority()
        .native_checkpoint(&exec, exec.allocation_policy())
        .unwrap();

    let (mut recovered, _) = install(reg.clone(), program.clone());
    recovered
        .durability_recovery()
        .restore_native_checkpoint(&exec, &ckpt)
        .unwrap();
    recovered
        .readmit_recovered_initial_schema_installation(receipt.clone())
        .unwrap();
    let _rec_reg = recovered
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(std::sync::Arc::new(ColdRetentionCompanion::default()))
        .unwrap();
    let main_id = recovered.main_branch_identity();
    let restored_snap = {
        let (_, current) = recovered.observe_branch(&main_id).unwrap();
        recovered
            .snapshots()
            .snapshot_for_observation(&current.observation())
            .unwrap()
    };
    let mut ceiling = 1usize;
    loop {
        recovered.set_retention_capacity_for_test(ceiling, 8);
        match recovered.observe_branch(&main_id) {
            Err(BasisDenial::RetentionCapacityExhausted) => {
                recovered.set_retention_capacity_for_test(ceiling + 1, 8);
                drop(recovered.observe_branch(&main_id).unwrap());
                recovered.set_retention_capacity_for_test(ceiling, 8);
                break;
            }
            Ok(probe) => {
                drop(probe);
                ceiling += 1;
                assert!(ceiling <= 16);
            }
            Err(other) => panic!("unexpected observe_branch error: {other:?}"),
        }
    }
    assert_eq!(
        recovered.observe_branch(&main_id).unwrap_err(),
        BasisDenial::RetentionCapacityExhausted
    );

    // Pinned cold head refills without acquiring a surplus observation slot.
    let loan = recovered
        .read_truth()
        .prepare_managed_index_rebuild(&exec, &restored_snap, &program, a)
        .expect("pinned cold head must refill without a surplus observation slot");
    let rows = loan.rows(exec);
    let read = rows.acquire_all().unwrap();
    let members: Vec<EntityId> = rows
        .occurrences(&read)
        .unwrap()
        .iter()
        .map(|o| o.entity())
        .collect();
    assert_eq!(members, vec![x]);
    let payload: std::sync::Arc<dyn std::any::Any + Send + Sync> = std::sync::Arc::new(members);
    let (refilled, retirement) = recovered
        .read_truth()
        .finish_managed_index_rebuild(&exec, &restored_snap, &loan, &payload)
        .unwrap();
    drop(retirement);
    let ready = recovered
        .read_truth()
        .observe_managed_index_source_if_ready(&exec, &restored_snap, &program, a)
        .unwrap()
        .expect("refilled cold source is ready at the ceiling");
    assert!(refilled.same_source(&ready));

    // A stale, still-unplaced cold root refuses ForeignSource through
    // branch_live_root_matches at the exact observation ceiling.
    let (mut stale, _) = install(reg, program.clone());
    stale
        .durability_recovery()
        .restore_native_checkpoint(&exec, &ckpt)
        .unwrap();
    stale
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let _stale_reg = stale
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(std::sync::Arc::new(ColdRetentionCompanion::default()))
        .unwrap();
    let stale_main = stale.main_branch_identity();
    let stale_snap = {
        let (_, cur) = stale.observe_branch(&stale_main).unwrap();
        stale
            .snapshots()
            .snapshot_for_observation(&cur.observation())
            .unwrap()
    };
    {
        let (_, basis) = stale.observe_branch(&stale_main).unwrap();
        let mut tx = stale
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        tx.push_batch(advance_batch(), exec.allocation_policy())
            .unwrap();
        tx.commit(&exec, &stale).unwrap();
    }
    stale.set_retention_capacity_for_test(ceiling, 8);
    assert_eq!(
        stale.observe_branch(&stale_main).unwrap_err(),
        BasisDenial::RetentionCapacityExhausted
    );
    assert!(matches!(
        stale
            .read_truth()
            .prepare_managed_index_rebuild(&exec, &stale_snap, &program, a),
        Err(ManagedIndexSourcePreparationDenial::ForeignSource)
    ));
    stale.snapshots().release_snapshot(&stale_snap).unwrap();
    recovered
        .snapshots()
        .release_snapshot(&restored_snap)
        .unwrap();
}
