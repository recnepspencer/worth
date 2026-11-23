use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn memory_checkpoint_preserves_lineage_artifact_and_original_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "first");
    let second = create_entity_outcome(execution, &runtime, "second");
    let first_entity = changed_entities(&first)[0];
    let second_entity = changed_entities(&second)[0];
    let first_lineage = runtime
        .lineage_access()
        .for_record(execution, first_entity)
        .unwrap()
        .unwrap()
        .lineage_id;
    let second_lineage = runtime
        .lineage_access()
        .for_record(execution, second_entity)
        .unwrap()
        .unwrap()
        .lineage_id;
    let checkpoint = runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    assert_eq!(
        checkpoint
            .lineage
            .digest_basis()
            .published_lineage_commit_count,
        runtime
            .history()
            .commit_envelopes_snapshot()
            .iter()
            .filter(|envelope| envelope.has_lineage_authority())
            .count()
    );
    assert_eq!(
        checkpoint
            .lineage
            .digest_basis()
            .published_lineage_event_count,
        runtime
            .history()
            .commit_envelopes_snapshot()
            .iter()
            .map(|envelope| envelope.lineage_digest_basis().lineage_event_count())
            .sum::<usize>()
    );
    assert_eq!(
        checkpoint
            .lineage
            .digest_basis()
            .published_lineage_decision_count,
        runtime
            .history()
            .commit_envelopes_snapshot()
            .iter()
            .map(|envelope| envelope.lineage_digest_basis().lineage_decision_count())
            .sum::<usize>()
    );
    assert_eq!(
        checkpoint.lineage.counters().node_count,
        checkpoint.lineage.nodes().len()
    );
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let graph = recovered.lineage_access().graph(
        execution,
        crate::facade::lineage::LineageGraphRequest {
            branch_id: BranchId("main".to_string()),
            traversal_basis:
                crate::facade::lineage::LineageGraphTraversalBasis::FullBranchGraphMaterialization,
        },
    ).unwrap();

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.events.len(), 2);
    assert!(graph
        .events
        .iter()
        .all(|event| event.kind == LineageEventKind::Create));
    assert_eq!(
        recovered
            .lineage_access()
            .for_record(execution, first_entity)
            .unwrap()
            .unwrap()
            .lineage_id(),
        first_lineage
    );
    assert_eq!(
        recovered
            .lineage_access()
            .for_record(execution, second_entity)
            .unwrap()
            .unwrap()
            .lineage_id(),
        second_lineage
    );
}

// Fixture adaptation (C2.4): restores e4d3b01c8d's retained envelope census
// and partition-image pointer identity on the Memory runtime (Embedded
// checkpoint plus post-checkpoint tail recovery plan).
#[test]
fn durability_contract_checkpoint_tail_recovery_preserves_post_checkpoint_commits() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let main = create_entity_outcome(execution, &runtime, "main-a");
    let _checkpoint = runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let later = create_entity_outcome(execution, &runtime, "main-b");
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let checkpoint_images = plan.checkpoint.as_ref().unwrap().partition_images.as_ptr();
    let mut recovered = runtime_with_test_schema();
    let outcome = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    assert_eq!(outcome.recovered_commits, 2);
    assert_eq!(outcome.coverage.checkpoint_commits, 1);
    assert_eq!(outcome.coverage.replayed_tail_commits, 1);
    assert_eq!(outcome.latest_commit, Some(later.commit.clone()));
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_string())),
        Some(later.commit.clone())
    );
    assert_eq!(
        recovered
            .replay()
            .canonical_commit_envelope(execution, main.commit.commit_id)
            .unwrap()
            .unwrap()
            .commit
            .commit_id,
        main.commit.commit_id
    );
    let retained = recovered.durability.latest_checkpoint().unwrap();
    assert_eq!(retained.envelopes.len(), 1);
    assert_eq!(retained.partition_images.as_ptr(), checkpoint_images);
}

// Fixture adaptation (C2.4): restores f9f15bf8c8's tail-replay snapshot-handle
// release bound beyond live capacity on the Memory runtime.
#[test]
fn durability_tail_replay_releases_each_commit_snapshot_beyond_live_capacity() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(
        crate::facade::config::RelationalRuntimeProfile::AiWorkflow,
    );
    create_entity(execution, &runtime, "replay-capacity-checkpoint");
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    for index in 0..70 {
        create_entity(
            execution,
            &runtime,
            &format!("replay-capacity-tail-{index}"),
        );
    }
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let mut recovered = runtime_with_test_schema_profile(
        crate::facade::config::RelationalRuntimeProfile::AiWorkflow,
    );

    let outcome = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    assert_eq!(outcome.recovered_commits, 71);
    assert_eq!(
        recovered
            .storage_access()
            .storage_stats(execution)
            .unwrap()
            .published_snapshot_handle_count,
        0,
        "replay closeout must not consume the live published-snapshot budget"
    );
}

// Fixture adaptation (C2.4): restores b736be6f47:cold_invariants.rs's
// `selected_branch_views_preserve_cold_probe_and_demand` over ExternalMembers
// recovery, probing `selected_branch_state(&transaction.basis)` through
// `probe_full_sources` with zero-read and targeted-demand assertions.
#[test]
fn selected_branch_views_preserve_cold_probe_and_demand() {
    use crate::durability::data::{
        RecoveryVerificationMode, RelationalCheckpointArtifactPolicy, RelationalCheckpointMember,
    };
    use crate::durability::medium::{
        RelationalDurableCheckpointSource, RelationalDurableColdSource,
        RelationalDurableMemberCustody, RelationalDurableMemberLocator,
        RelationalDurableOwnedReadRefusal, RelationalDurableReadPayload,
        RelationalDurableRecoverySource,
    };
    use crate::storage::data::RelationalStorageUnavailableCause as Cause;
    use std::collections::BTreeMap;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    #[derive(Debug)]
    struct CountedColdSource(
        BTreeMap<RelationalDurableMemberLocator, worth_execution::ExecutionImmutableBytes>,
        AtomicUsize,
    );
    impl RelationalDurableColdSource for CountedColdSource {
        fn retain_members_for_request(
            &self,
            request: &worth_execution::ExecutionRequest<'_, '_>,
            _: &[RelationalDurableMemberLocator],
            issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
        ) -> Result<RelationalDurableMemberCustody, RelationalDurableOwnedReadRefusal> {
            RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || {
                Ok(())
            })
        }
        fn checkpoint_selected_read_protection(
            &self,
            request: &worth_execution::ExecutionRequest<'_, '_>,
        ) -> Result<RelationalDurableReadPayload, RelationalDurableOwnedReadRefusal> {
            request
                .check_live()
                .map_err(|c| RelationalDurableOwnedReadRefusal::Request(c.into()))?;
            Ok(Box::new(()))
        }
        fn read_member_owned(
            &self,
            request: &worth_execution::ExecutionRequest<'_, '_>,
            _: &(dyn std::any::Any + Send + Sync),
            locator: RelationalDurableMemberLocator,
            _: u64,
        ) -> Result<worth_execution::ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal>
        {
            request
                .check_live()
                .map_err(|c| RelationalDurableOwnedReadRefusal::Request(c.into()))?;
            let bytes = self
                .0
                .get(&locator)
                .cloned()
                .ok_or(RelationalDurableOwnedReadRefusal::Storage(
                    Cause::MissingMember,
                ))?;
            if crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.as_ref()).is_err() {
                self.1.fetch_add(1, Ordering::Relaxed);
            }
            Ok(bytes)
        }
        fn retain_members(
            &self,
            _: &[RelationalDurableMemberLocator],
        ) -> Result<RelationalDurableMemberCustody, Cause> {
            Ok(RelationalDurableMemberCustody::new(()))
        }
        fn read_member(
            &self,
            locator: RelationalDurableMemberLocator,
            _: u64,
        ) -> Result<Vec<u8>, Cause> {
            self.1.fetch_add(1, Ordering::Relaxed);
            self.0
                .get(&locator)
                .map(|b| b.to_vec())
                .ok_or(Cause::MissingMember)
        }
        fn read_commit(
            &self,
            _: crate::publication::patch::data::PatchStreamPosition,
            _: u64,
        ) -> Result<Vec<u8>, Cause> {
            Err(Cause::MissingMember)
        }
    }
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let producer = runtime_with_test_schema();
    let entity = create_entity_in_partition(execution, &producer, "selected-target", PartitionId(1));
    let untouched =
        create_entity_in_partition(execution, &producer, "selected-sibling", PartitionId(2));
    let native = producer
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let members = native
        .members()
        .iter()
        .map(|member| match member {
            RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.clone()),
            RelationalCheckpointMember::Carried { .. } => unreachable!(),
        })
        .collect();
    let source = Arc::new(CountedColdSource(members, AtomicUsize::new(0)));
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            native.covered_position().unwrap(),
            native.covered_branch_transitions().unwrap(),
            native,
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    let mut runtime = runtime_with_test_schema();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime.durability_recovery().recover(execution, plan).unwrap();
    assert_eq!(source.1.load(Ordering::Relaxed), 0);
    let transaction = test_owner_begin_transaction_for_main(&runtime);
    let selected = runtime.selected_branch_state(&transaction.basis).unwrap();
    assert_eq!(
        source.1.load(Ordering::Relaxed),
        0,
        "acquiring selected_branch_state performs zero cold I/O"
    );
    assert!(matches!(
        selected.state().root().probe_full_region(entity.partition_id),
        crate::storage::residency::PartitionProbe::Cold
    ));
    let answer = crate::storage::residency::probe_full_sources(execution, [&selected], |[view]| {
        view.get_partition(entity.partition_id).is_some()
    })
    .unwrap();
    assert!(answer);
    assert!(source.1.load(Ordering::Relaxed) > 0);
    assert!(matches!(
        selected.state().root().probe_full_region(entity.partition_id),
        crate::storage::residency::PartitionProbe::Resident(_)
    ));
    assert!(
        matches!(
            selected.state().root().probe_full_region(untouched.partition_id),
            crate::storage::residency::PartitionProbe::Cold
        ),
        "selected-view probe must only demand the targeted partition"
    );
}
