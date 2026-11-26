//! Closeout courts for `G7` paged commit index:
//! - Real `C1 -> acknowledge C2 -> C3` with post-ack working-set eviction and
//!   late cold fault rebinding before `C1` retirement / close.
//! - Typed `StorageUnavailable` through runtime doors on damaged commit pages.
//! - Non-carrying cross-source capture reading prior canonical members from the
//!   prior source with digest verification.
//! - Tail replay whose parent is a non-head checkpoint commit.
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::config::{RelationalRuntimeProfile, ResidencyBudget};
use worth_relational::facade::durability::{
    RecoveryFailureClass, RecoveryVerificationMode, RelationalAssociatedEffectRefusal,
    RelationalCheckpointArtifactPolicy, RelationalCheckpointMember,
    RelationalDurableCheckpointSource, RelationalDurableColdSource,
    RelationalDurableRecoverySource, RelationalNativeCheckpoint, RelationalRecoveryStop,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::publication::PatchStreamPosition;
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
use worth_testkit_relational::aspect_storage::publications::{
    changed_entities, release_test_commit_snapshot,
};
use worth_testkit_relational::cold_recovery::{
    capture_external_checkpoint, recover, source_with_history, update_entity,
};
use worth_testkit_relational::durable_medium::immutable_members::{
    ColdRead, ImmutableCheckpointMemberSource,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::{
    create_entity_outcome, durable_runtime, schema,
};

fn reopen_with_budget(
    request: &ExecutionRequest<'_, '_>,
    checkpoint: RelationalNativeCheckpoint,
    source: Arc<ImmutableCheckpointMemberSource>,
    medium: &Arc<TestDurableMedium>,
    historical: u64,
) -> RelationalRuntime {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(schema())
        .residency_budget(ResidencyBudget {
            head_working_set_bytes: u64::MAX / 4,
            historical_working_set_bytes: historical,
            max_in_flight_fill_bytes: u64::MAX / 4,
            streaming_window_bytes: 64 * 1024,
        })
        .build_durable(medium.attachment())
        .unwrap();
    recover(request, checkpoint, source, runtime)
}

fn acknowledge(
    request: &ExecutionRequest<'_, '_>,
    medium: &Arc<TestDurableMedium>,
    checkpoint: &RelationalNativeCheckpoint,
) {
    medium.file_checkpoint(checkpoint.clone());
    request
        .run_native_with_checkpoint(|context| checkpoint.acknowledge_selected(request, context))
        .unwrap()
        .0
        .unwrap();
}

#[test]
fn acknowledged_c2_evicts_under_budget_rebinds_late_faults_and_carries_into_c3() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let historical = 128 * 1024;
    let fixture = source_with_history(request, 300, false);
    let c1 = fixture.input.checkpoint().unwrap().checkpoint().clone();
    let medium = fixture.medium.clone();
    medium.file_checkpoint(c1.clone());

    let mut runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(schema())
        .residency_budget(ResidencyBudget {
            head_working_set_bytes: u64::MAX / 4,
            historical_working_set_bytes: historical,
            max_in_flight_fill_bytes: u64::MAX / 4,
            streaming_window_bytes: 64 * 1024,
        })
        .build_durable(medium.attachment())
        .unwrap();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            request,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    for update in 0..20 {
        update_entity(
            request,
            &runtime,
            fixture.entity,
            &format!("ack-tail-{update}"),
        );
    }

    // Capture C2 carrying C1's members on `medium`.
    let current_members: Vec<_> = c1
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect();
    let medium_source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    let c2 = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &current_members,
                source: Some(medium_source.clone()),
            },
        )
        .unwrap();

    // Fault a cold commit BETWEEN C2 capture and C2 acknowledgement so it
    // enters ColdCommitCatalog::live backed by C1, then acknowledge C2.
    let mid_receipt = runtime
        .history()
        .immutable_commit_receipt_at_patch_stream_position(PatchStreamPosition(150))
        .unwrap();
    let _ = runtime
        .durability_authority()
        .discover_associated_effect(request, &mid_receipt)
        .unwrap();
    acknowledge(request, &medium, &c2);

    // Post-ack C2 commit index must remain bound to the WorkingSetAccountant
    // and evict pages across a full history traversal.
    let c2_dir = super::selected_directory::read_native(&c2);
    assert_eq!(c2_dir.commit_index.count, 321);
    for position in 1..=321 {
        runtime
            .history()
            .immutable_commit_receipt_at_patch_stream_position(PatchStreamPosition(position))
            .unwrap();
    }
    let pressure = runtime.residency_pressure().unwrap();
    assert!(
        pressure.historical_resident_bytes <= historical,
        "post-ack C2 commit index stays within historical budget: {pressure:?}"
    );
    assert!(
        pressure.evictions_completed > 0,
        "post-ack C2 commit index evicts pages under tight budget: {pressure:?}"
    );

    // C3 captured after acknowledging C2 carries every C2 commit-index page.
    let c2_members: Vec<_> = c2
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect();
    let c3 = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &c2_members,
                source: Some(medium_source),
            },
        )
        .unwrap();
    let c2_pages: Vec<_> = c2_dir.commit_index.pages.iter().map(|p| p.member).collect();
    let c3_pages: Vec<_> = super::selected_directory::read_native(&c3)
        .commit_index
        .pages
        .iter()
        .map(|p| p.member)
        .collect();
    assert_eq!(c3_pages, c2_pages);
    assert!(c3_pages.iter().all(|locator| {
        c3.members().iter().any(|member| {
            matches!(member, RelationalCheckpointMember::Carried { locator: carried } if carried == locator)
        })
    }));
}

#[test]
fn damaged_commit_index_page_surfaces_typed_storage_unavailable() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let fixture = source_with_history(request, 300, false);
    let c1 = fixture.input.checkpoint().unwrap().checkpoint().clone();
    let source = fixture.source.clone();
    let middle_page = super::selected_directory::read_native(&c1)
        .commit_index
        .pages[1]
        .member;
    let runtime = reopen_with_budget(request, c1, source.clone(), &fixture.medium, 128 * 1024);
    source
        .failures
        .lock()
        .unwrap()
        .insert(ColdRead::Member(middle_page), Cause::Damaged);

    let err = runtime
        .durability_authority()
        .discover_all_associated_effects(request)
        .unwrap_err();
    assert!(
        matches!(
            err,
            RelationalAssociatedEffectRefusal::Recovery(RelationalRecoveryStop::Durability(ref d))
                if d.class == RecoveryFailureClass::ColdStorageUnavailable
        ),
        "damaged commit-index page must return typed ColdStorageUnavailable, got {err:?}"
    );
}

#[test]
fn cross_source_non_carrying_capture_reads_prior_canonical_members_from_prior_source() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let fixture = source_with_history(request, 20, false);
    let c1 = fixture.input.checkpoint().unwrap().checkpoint().clone();
    let runtime = recover(
        request,
        c1.clone(),
        fixture.source.clone(),
        durable_runtime(&fixture.medium),
    );
    update_entity(request, &runtime, fixture.entity, "cross-source-tail");
    // Destination holds metadata members so metadata carries, while excluding
    // canonical commit members so `checkpoint_history/capture.rs` must copy
    // prior canonical members from `prior_source` (fixture.source) rather than
    // `empty_dest`.
    let canonical_set: std::collections::BTreeSet<_> =
        fixture.source.canonical_members.keys().copied().collect();
    let non_canonical: Vec<_> = c1
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .filter(|locator| !canonical_set.contains(locator))
        .collect();
    let empty_dest: Arc<dyn RelationalDurableColdSource> = TestDurableMedium::new();
    let captured = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &non_canonical,
                source: Some(empty_dest),
            },
        )
        .expect("cross-source capture copies prior canonical members from prior source");
    assert_eq!(
        super::selected_directory::read_native(&captured)
            .commit_index
            .count,
        22
    );
}

#[test]
fn tail_replay_with_non_head_checkpoint_parent_recovers_cleanly() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_runtime(&medium);
    let created = create_entity_outcome(request, &producer, "base-entity");
    let entity = changed_entities(&created)[0];
    release_test_commit_snapshot(&producer, &created);
    // Retain a historical component basis at commit 1 before advancing main so
    // C1 carries commit 1's root image while commit 1 is NOT a branch head.
    let services = producer.owner_component_services();
    let historical_basis = services
        .basis_port()
        .admit_branch_basis(&producer.main_branch_identity())
        .unwrap();
    let held = producer.retain_component_basis(&historical_basis).unwrap();
    for i in 0..5 {
        update_entity(request, &producer, entity, &format!("main-before-c1-{i}"));
    }
    let (c1, c1_source) = capture_external_checkpoint(request, &producer);
    let covered_pos = c1.covered_position().unwrap();
    let covered_trans = c1.covered_branch_transitions().unwrap();

    // Now fork from the non-head historical commit 1 and publish a tail commit
    // whose parent is commit 1 (a non-head checkpoint commit).
    let sibling = BranchId("from-non-head".into());
    let retained_source = services
        .fork_port()
        .admit_retained_fork_source(&historical_basis)
        .unwrap();
    services
        .fork_port()
        .fork_retained_branch(request, sibling.clone(), retained_source)
        .unwrap();
    producer.release_component_basis(held).unwrap();
    drop((historical_basis, services));
    let _ = super::merge_publications::create(
        request,
        &producer,
        "tail-on-non-head",
        PartitionId::main(),
        &sibling,
    );
    let tail_commit_id = producer
        .history()
        .historical_latest_commit()
        .unwrap()
        .commit_id;

    let tail_commits: Vec<_> = medium
        .committed_artifacts()
        .unwrap()
        .into_iter()
        .filter(|artifact| artifact.position() > covered_pos)
        .collect();
    let tail_transitions: Vec<_> = medium
        .offered_transitions()
        .into_iter()
        .filter(|transition| transition.sequence() > covered_trans)
        .collect();
    assert_eq!(tail_commits.len(), 1);
    assert_eq!(tail_transitions.len(), 1);

    let mut reopened = durable_runtime(&medium);
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            covered_pos,
            covered_trans,
            c1,
        )),
        tail_commits,
        tail_transitions,
        c1_source,
    );
    let plan = reopened
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    reopened
        .durability_recovery()
        .recover(request, plan)
        .expect("tail replay with non-head checkpoint parent must recover cleanly");
    assert_eq!(
        reopened
            .history()
            .immutable_commit_receipt(tail_commit_id)
            .map(|r| r.commit_id),
        Some(tail_commit_id)
    );
}
