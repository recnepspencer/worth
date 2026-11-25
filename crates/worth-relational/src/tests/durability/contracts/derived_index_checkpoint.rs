//! Checkpoints retain only the live derived-index payload across repeated edits.
//! Fixture adaptation (C2.3): the original checkpointed a runtime bound to the
//! retired in-crate test medium through `checkpoint_durably`, which also filed
//! a native image in that medium. The asserted sections are the owner's own
//! `DurableCheckpoint` and resident generations, so the claim has no
//! physical-medium content and runs on the Memory runtime.
use super::*;
use crate::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use worth_relational_durability::internal_port::durability::data::checkpoint_images::DurableCheckpointFieldsFixturePort as _;
use worth_relational_durability::internal_port::durability::data::checkpoint_images::DurableCheckpointOwnerPort as _;

fn name_index(runtime: &crate::runtime::RelationalRuntime) -> DerivedIndexDefinition {
    runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "checkpoint-index-tail".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: false,
        })
        .expect("index fixture has identity capacity")
}
fn build(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    index: DerivedIndexId,
    outcome: &crate::transactions::data::CommitResult,
) -> crate::indexes::data::DerivedIndexGenerationId {
    let built = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: outcome.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index],
        },
    );
    assert!(built.failed_indexes.is_empty());
    built.generations[0].generation_id
}
#[test]
fn repeated_edits_checkpoint_only_the_live_index_payload() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "checkpoint-size-0");
    let index = name_index(&runtime);
    build(execution, &runtime, index.index_id, &first);
    release_test_commit_snapshot(&runtime, &first);
    for ordinal in 1..24 {
        let outcome =
            create_entity_outcome(execution, &runtime, &format!("checkpoint-size-{ordinal}"));
        build(execution, &runtime, index.index_id, &outcome);
        release_test_commit_snapshot(&runtime, &outcome);
    }
    let legacy_all = crate::indexes::data::DerivedIndexArtifacts::new(
        runtime.index_access().generations_snapshot(),
    );
    let legacy_bytes = rmp_serde::to_vec_named(&legacy_all).unwrap().len();
    let checkpoint = runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let retained_bytes = rmp_serde::to_vec_named(checkpoint.derived_index_checkpoint().unwrap())
        .unwrap()
        .len();
    assert!(retained_bytes * 4 < legacy_bytes);
    assert!(runtime.index_access().generations_snapshot().len() < 24);
}
#[test]
fn checkpoint_index_artifact_survives_tail_replay_without_envelope_caches() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "checkpoint-index-first");
    let index = name_index(&runtime);
    let first_generation = build(execution, &runtime, index.index_id, &first);
    let checkpoint = runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    assert!(checkpoint.derived_index_artifacts.is_empty());
    assert_eq!(
        checkpoint.derived_index_checkpoint_format_for_test(),
        crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::FORMAT_VERSION
    );
    let second = create_entity_outcome(execution, &runtime, "checkpoint-index-tail");
    let unjournaled_cache = build(execution, &runtime, index.index_id, &second);
    let plan = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert!(recovered.indexes.generation(first_generation).is_some());
    assert!(recovered.indexes.generation(unjournaled_cache).is_none());
    let rebuilt = build(execution, &recovered, index.index_id, &second);
    assert!(recovered.indexes.generation(rebuilt).is_some());
}

fn reopen_cold_index_members(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    producer: &crate::runtime::RelationalRuntime,
) -> (
    crate::runtime::RelationalRuntime,
    std::sync::Arc<IndexColdSource>,
) {
    use crate::durability::data::{RelationalCheckpointArtifactPolicy, RelationalCheckpointMember};
    use crate::durability::published_medium::{
        RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
    };

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
        .map(|m| match m {
            RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.clone()),
            RelationalCheckpointMember::Carried { .. } => unreachable!(),
        })
        .collect();
    let source = std::sync::Arc::new(IndexColdSource(members, Default::default()));
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
    let mut runtime =
        runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    (runtime, source)
}

// Fixture adaptation (C2.4): restores the mutable index snapshot demand/install
// branch of 8283728aff:cold_index_selections.rs over ExternalMembers recovery.
#[test]
fn cold_checkpoint_mutable_index_snapshot_demand_and_install_preserves_carried_selection_and_postings(
) {
    use crate::indexes::data::BoundedIndexParityMode;
    use crate::runtime::cold_residency_census::RelationalRegionProbeStatus as Probe;
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let (producer, entity, changed, _stable, current, carried) =
        scoped_affinity::carried_index_producer(execution);
    let (runtime, source) = reopen_cold_index_members(execution, &producer);
    let branch = BranchId("main".into());
    assert_eq!(
        runtime
            .observe_branch_region_residency(&branch, entity.partition_id)
            .unwrap()
            .full,
        Probe::Cold
    );
    assert!(
        runtime.observe_carried_index_checkpoint_image().is_some(),
        "carried index image must be pending before mutable snapshot demand"
    );
    let reads_before = source.1.load(std::sync::atomic::Ordering::Relaxed);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let (mut snapshot, original) = runtime.indexes.snapshot_with_original();
    original
        .expect("pending image belongs to the selected index state")
        .readmit_into_snapshot(execution, &mut snapshot)
        .unwrap();
    let reads_after_demand = source.1.load(std::sync::atomic::Ordering::Relaxed);
    assert!(
        reads_after_demand > reads_before,
        "mutable snapshot demand must read the carried index image"
    );
    assert!(
        runtime.observe_carried_index_checkpoint_image().is_some(),
        "carried image stays pending until the demanded snapshot is installed"
    );
    runtime.indexes.install(snapshot);
    assert!(
        runtime.observe_carried_index_checkpoint_image().is_none(),
        "installing the demanded snapshot must clear the pending image before any ordinary read"
    );
    let access = runtime.index_access();
    for expected in [&current, &carried] {
        assert_eq!(
            access
                .read_published_generation_for_observation(
                    execution,
                    expected.index_id,
                    &basis.observation()
                )
                .unwrap()
                .as_deref(),
            Some(expected.as_ref())
        );
    }
    assert_eq!(
        source.1.load(std::sync::atomic::Ordering::Relaxed),
        reads_after_demand,
        "ordinary generation reads must reuse the installed snapshot without fallback reads"
    );
    let view = runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap();
    let lookup = access
        .prepare_entity_field_lookup(
            &view,
            changed,
            KindId(1),
            &aspect_field_locator(aspect_key("name"), field_key("name")),
            execution,
        )
        .unwrap();
    let outcome = lookup
        .execute_prepared_entity_field_lookup(
            &string_aspect_value("complete-index-updated"),
            2,
            BoundedIndexParityMode::Production,
            execution,
            || Ok::<_, ()>(()),
        )
        .unwrap();
    assert_eq!(outcome.candidate_entity_ids(), &[entity]);
    assert_eq!(
        runtime
            .observe_branch_region_residency(&branch, entity.partition_id)
            .unwrap()
            .full,
        Probe::Cold
    );
}

#[derive(Debug)]
struct IndexColdSource(
    std::collections::BTreeMap<
        crate::durability::medium::RelationalDurableMemberLocator,
        worth_execution::ExecutionImmutableBytes,
    >,
    std::sync::atomic::AtomicUsize,
);
impl crate::durability::medium::RelationalDurableColdSource for IndexColdSource {
    fn retain_members_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        _: &[crate::durability::medium::RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn crate::durability::medium::RelationalDurableColdSource>,
    ) -> Result<
        crate::durability::medium::RelationalDurableMemberCustody,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        crate::durability::medium::RelationalDurableMemberCustody::try_new_from_source_for_request(
            issuer,
            request,
            || Ok(()),
        )
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::durability::medium::RelationalDurableReadPayload,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        use crate::durability::medium::RelationalDurableOwnedReadRefusal as R;
        request.check_live().map_err(|c| R::Request(c.into()))?;
        Ok(Box::new(()))
    }
    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        _: &(dyn std::any::Any + Send + Sync),
        locator: crate::durability::medium::RelationalDurableMemberLocator,
        _: u64,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        use crate::durability::medium::RelationalDurableOwnedReadRefusal as R;
        use crate::storage::data::RelationalStorageUnavailableCause as Cause;
        request.check_live().map_err(|c| R::Request(c.into()))?;
        let bytes = self
            .0
            .get(&locator)
            .cloned()
            .ok_or(R::Storage(Cause::MissingMember))?;
        if crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.as_ref()).is_err() {
            self.1.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(bytes)
    }
    fn retain_members(
        &self,
        _: &[crate::durability::medium::RelationalDurableMemberLocator],
    ) -> Result<
        crate::durability::medium::RelationalDurableMemberCustody,
        crate::storage::data::RelationalStorageUnavailableCause,
    > {
        Ok(crate::durability::medium::RelationalDurableMemberCustody::new(()))
    }
    fn read_member(
        &self,
        locator: crate::durability::medium::RelationalDurableMemberLocator,
        _: u64,
    ) -> Result<Vec<u8>, crate::storage::data::RelationalStorageUnavailableCause> {
        self.1.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.0
            .get(&locator)
            .map(|b| b.to_vec())
            .ok_or(crate::storage::data::RelationalStorageUnavailableCause::MissingMember)
    }
    fn read_commit(
        &self,
        _: crate::publication::patch::data::PatchStreamPosition,
        _: u64,
    ) -> Result<Vec<u8>, crate::storage::data::RelationalStorageUnavailableCause> {
        Err(crate::storage::data::RelationalStorageUnavailableCause::MissingMember)
    }
}

// Fixture adaptation (C2.4): restores 8283728aff:cold_index_frontier.rs over an
// external-members capture recovered into a Memory runtime (no medium adapter).
#[path = "derived_index_checkpoint/artifact_affinity.rs"]
mod artifact_affinity;
#[path = "derived_index_checkpoint/deleted_branch_reader.rs"]
mod deleted_branch_reader;
#[path = "derived_index_checkpoint/historical_reader.rs"]
mod historical_reader;
#[path = "derived_index_checkpoint/scoped_affinity.rs"]
mod scoped_affinity;
