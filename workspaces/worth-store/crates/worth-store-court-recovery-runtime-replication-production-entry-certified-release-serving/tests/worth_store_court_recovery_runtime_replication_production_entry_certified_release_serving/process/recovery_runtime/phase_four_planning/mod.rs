#[allow(unused_imports)]
use crate::phase_four_planning::*;

use {
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery,
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery_with_limits,
    worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked,
    worth_store_testkit_recovery_runtime::phase_three_support::initialize_store,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_checkpoint,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_covered_wal,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis,
};

#[test]
fn selected_checkpoint_becomes_one_effect_free_immutable_plan() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_secured_synthetic_checkpoint(&root, store);
    publish_synthetic_covered_wal(&root);

    let selected = admitted_recovery(&root)
        .discover()
        .unwrap()
        .select()
        .unwrap();
    let planned = selected.plan().unwrap();

    assert_eq!(planned.store_identity(), store);
    assert_eq!(
        planned.freshness_sample().selected_checkpoint_generation(),
        1
    );
    assert_eq!(planned.freshness_sample().policy_identity(), [7; 32]);
    assert!(planned.operation_fates().operations().is_empty());
    assert!(planned.redo_plan().decisions().is_empty());
    assert_eq!(planned.redo_plan().counters().targets(), 0);

    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Refused(cancelled) =
        planned.cancel_before_execution()
    else {
        panic!("an explicit pre-execution cancellation is a refusal")
    };
    assert_eq!(cancelled.recovery_effects(), 0);
}

#[test]
fn an_unsecured_checkpoint_cannot_cross_the_planning_boundary() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_synthetic_checkpoint(&root, store);
    publish_synthetic_covered_wal(&root);

    let selected = admitted_recovery(&root)
        .discover()
        .unwrap()
        .select()
        .unwrap();
    let blocked = match selected.plan() {
        Ok(_) => panic!("an unsecured checkpoint cannot form a Phase 4 plan"),
        Err(outcome) => expect_blocked(outcome),
    };
    assert_eq!(
        blocked.cause().damage(),
        Some(worth_store_recovery_runtime::PhysicalRecoveryBlockKind::BindingFreshness)
    );
    assert_eq!(
        blocked.evidence().planning_denial,
        Some(worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::BindingFreshness(
            worth_store::physical_runtime::StoreRecoveryBindingSampleDenial::MissingCheckpointSecurityBinding,
        ))
    );
    assert_eq!(blocked.recovery_effects(), 0);
}

#[test]
fn ordinary_store_mutation_reopens_as_a_nonempty_effect_free_plan() {
    let retained_root = prepare_ordinary_recovery_root("c8-phase4-ordinary");
    let recovery_limits = ordinary_recovery_limits(4_096);
    let selected = admitted_recovery_with_limits(retained_root.path(), recovery_limits)
        .discover()
        .unwrap()
        .select()
        .unwrap();
    let planned = selected.plan().unwrap();
    assert_eq!(planned.discovery_counters().manifest_entries, 3);
    assert_eq!(planned.freshness_sample().operations().len(), 3);
    assert_eq!(planned.freshness_sample().wal_members().len(), 2);
    let fates = planned.operation_fates().operations();
    assert_eq!(fates.len(), 3);
    assert_eq!(
        fates[0].fate(),
        worth_store_recovery_physics::RecoveryOperationFate::AcknowledgedDurable
    );
    assert_eq!(
        fates[1].fate(),
        worth_store_recovery_physics::RecoveryOperationFate::DurableUnacknowledged
    );
    assert_eq!(
        fates[2].fate(),
        worth_store_recovery_physics::RecoveryOperationFate::Indeterminate
    );
    let decisions = planned.redo_plan().resolved_decisions().collect::<Vec<_>>();
    assert_eq!(decisions.len(), 2);
    assert_eq!(
        decisions[0].kind(),
        worth_store_recovery_physics::PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn
    );
    assert_eq!(decisions[0].record().lsn().get(), 2);
    assert_eq!(
        decisions[1].kind(),
        worth_store_recovery_physics::PhysicalRedoDecisionKind::Apply
    );
    assert_eq!(decisions[1].record().lsn().get(), 3);
    assert_eq!(
        decisions[1].target().identity(),
        worth_store_recovery_physics::PhysicalRedoTargetIdentity::InlinePage {
            segment: 1,
            page: 1,
            generation: 3,
        }
    );
    let cost = planned.plan_cost();
    assert_eq!(cost.redo_targets(), 2);
    assert_eq!(cost.redo_bytes(), 34_315);
    assert_eq!(cost.distinct_targets(), 2);
    assert_eq!(cost.operation_bindings(), 3);
    assert_eq!(cost.observation_reads(), 7);
    // Includes the 368-byte source-root manifest the checkpoint binds.
    assert_eq!(cost.observation_bytes(), 73_993);
    assert_eq!(cost.staging_bytes(), 3_276_800);
    assert_eq!(cost.dirty_frames(), 1);
    let counters = planned.planning_counters();
    assert_eq!(counters.page_extent_reads(), 7);
    assert_eq!(counters.page_extent_bytes(), 17_328);
    assert_eq!(counters.redo_records(), 2);
    assert_eq!(counters.redo_targets(), 2);
    assert_eq!(counters.redo_apply(), 1);
    assert_eq!(counters.redo_skip_page_lsn(), 1);
    assert_eq!(counters.redo_skip_operation(), 0);
    assert_eq!(counters.fate_counts(), [1, 1, 0, 1]);
    assert_eq!(counters.page_extent_integrity_attempts(), 1);
    assert_eq!(counters.page_extent_integrity_admissions(), 1);
    assert_eq!(counters.page_extent_integrity_rejections(), 0);
    assert_eq!(counters.page_extent_owner_projections(), 1);
    assert_eq!(counters.page_extent_owner_decoders(), 1);
    assert_ne!(planned.publication_plan().plan_identity(), [0; 32]);
    let protocol = planned.publication_plan().root_protocol();
    let publication = protocol.publication();
    assert_eq!(
        protocol.catalog_candidate(),
        worth_store_physical_format::RecordArtifactFile::CatalogCandidate { publication }
    );
    assert!(matches!(
        protocol.previous_candidate(),
        worth_store_physical_format::RecordArtifactFile::RootSelectorCandidate {
            publication: candidate,
            ..
        } if candidate == publication
    ));
    assert!(matches!(
        protocol.current_candidate(),
        worth_store_physical_format::RecordArtifactFile::RootSelectorCandidate {
            publication: candidate,
            ..
        } if candidate == publication
    ));
    let staging = planned.staging_layout();
    assert_eq!(staging.base_image().selected_root().generation(), 3);
    let base = staging.base_image();
    assert_eq!(base.root_states().len(), 1);
    let root_state = &base.root_states()[0];
    assert_eq!(root_state.root_publication_allocation_bytes(), 3_260_416);
    assert_eq!(root_state.manifest_capacity_transition(), 1);
    assert_eq!(root_state.inline_allocations().len(), 1);
    let inline = root_state.inline_allocations()[0];
    assert_eq!(inline.segment().segment_id().get(), 1);
    assert_eq!(inline.segment().generation().get(), 3);
    assert_eq!(inline.page_capacity(), 407);
    assert_eq!(inline.used_pages(), 1);
    assert_eq!(root_state.last_inline_segment(), Some(inline.segment()));
    assert!(base
        .actions()
        .iter()
        .any(|action| { Some(action.placement().record()) == root_state.last_inline_record() }));
    assert_eq!(base.actions().len(), 3);
    assert!(base.actions().iter().all(|action| action.is_projected()));
    let mut payloads = base
        .actions()
        .iter()
        .map(|action| action.placement().payload_bytes())
        .collect::<Vec<_>>();
    payloads.sort_unstable();
    assert_eq!(payloads, [14, 16, 23]);
    assert_eq!(base.segment_updates().len(), 1);
    let routing = base.segment_updates()[0].update();
    assert_eq!(routing.page_generation(), 3);
    assert_eq!(routing.data_generation(), 3);
    assert_eq!(routing.frame_index(), 0);
    assert_eq!(staging.actions().len(), 1);
    assert_eq!(staging.actions()[0].steps().len(), 1);
    assert_eq!(staging.actions()[0].steps()[0].record_lsn(), 3);
    assert_eq!(staging.actions()[0].source(), decisions[1].target());
    assert!(staging
        .commands()
        .iter()
        .any(|command| command.byte_count() == 16_384));
    assert_eq!(staging.allocated_bytes(), 3_276_800);
    assert_eq!(staging.write_bytes(), 16_384);
    assert_eq!(staging.dirty_frames(), 1);
    crate::publication_assertions::assert_exact_publication_plan(&planned);
    assert_eq!(planned.quiescence_plan().staging_commands(), 2);
    assert_eq!(
        planned.quiescence_plan().publication_commands(),
        planned.publication_plan().actions().len() as u64
    );
    assert_eq!(
        planned
            .quiescence_plan()
            .expected_live_commands_after_close(),
        0
    );

    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Refused(cancelled) =
        planned.cancel_before_execution()
    else {
        panic!("an ordinary plan remains effect-free before execution")
    };
    assert_eq!(cancelled.recovery_effects(), 0);
}
#[path = "page_extent_recovery/mod.rs"]
pub(crate) mod page_extent_recovery;
#[path = "phase_six_publication/mod.rs"]
pub(crate) mod phase_six_publication;
#[path = "phase_six_publication_counters/mod.rs"]
pub(crate) mod phase_six_publication_counters;
#[path = "root_protocol_terminal_evidence/mod.rs"]
pub(crate) mod root_protocol_terminal_evidence;
