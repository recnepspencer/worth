#[path = "phase_four_planning/page_extent_recovery.rs"]
mod page_extent_recovery;
#[path = "phase_four_planning/phase_four_denial_counters.rs"]
mod phase_four_denial_counters;
#[path = "phase_four_planning/phase_six_publication.rs"]
mod phase_six_publication;
#[cfg(feature = "certification-test-authority")]
#[path = "phase_four_planning/phase_six_publication_counters.rs"]
mod phase_six_publication_counters;
#[cfg(feature = "certification-test-authority")]
#[path = "phase_four_planning/phase_six_reopen_faults.rs"]
mod phase_six_reopen_faults;
#[path = "phase_four_planning/phase_six_reopen_media.rs"]
mod phase_six_reopen_media;
#[path = "phase_four_planning/phase_six_terminal_isolation.rs"]
mod phase_six_terminal_isolation;
#[allow(dead_code)]
mod phase_three_support;
#[path = "phase_four_planning/publication_assertions.rs"]
mod publication_assertions;
#[path = "phase_four_planning/root_protocol_terminal_evidence.rs"]
mod root_protocol_terminal_evidence;

use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

use phase_three_support::*;
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, StoreRecoveryBindingSampleDenial,
};
use worth_store_physical_format::{
    encode_data_frame_page_lsn, CheckpointBindingCompactionHeader, CheckpointRootBasis,
    CheckpointStreamEncoder, CheckpointWalSourceRange, DurableFrameKind,
    PhysicalCheckpointIdentity, PhysicalCheckpointSource, PhysicalPageLsn, RecordArtifactFile,
};
use worth_store_recovery_physics::{
    PhysicalRedoDecisionKind, PhysicalRedoPlanningDenial, PhysicalRedoTargetIdentity,
    RecoveryOperationFate, RecoveryPlanCostDenial,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension, PhysicalRecoveryOutcome,
    PhysicalRecoveryPlanningDenial,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_durable_wal_attempt_without_execution, canonical_physical_mutation_acknowledgment,
    canonical_rooted_mutation_without_acknowledgment, PhysicalResidencyStoreWorld,
};

fn publish_secured_synthetic_checkpoint(
    root: &Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
) -> PhysicalCheckpointIdentity {
    let checkpoint = PhysicalCheckpointIdentity::new(store, NonZeroU64::new(1).unwrap());
    let source = PhysicalCheckpointSource::secured_concurrent(
        checkpoint,
        CheckpointWalSourceRange::new(1, 2).unwrap(),
        CheckpointRootBasis::new(1, 7),
        1,
        [7; 32],
        8,
    )
    .unwrap();
    let (encoder, header) = CheckpointStreamEncoder::begin(source);
    let cutover = CheckpointBindingCompactionHeader::new(1, 2).unwrap();
    let (compaction, cutover_record) = encoder.begin_binding_compaction(cutover);
    let (_, footer) = compaction.finish();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(&cutover_record);
    bytes.extend_from_slice(&footer);
    std::fs::write(root.join("families").join("checkpoint.current"), bytes).unwrap();
    checkpoint
}

fn prepare_ordinary_recovery_root(name: &str) -> worth_store_test_support::TemporaryDirectory {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery(name).unwrap();
    let retained_root = world.retained_root();
    let acknowledgment =
        canonical_physical_mutation_acknowledgment(&world, [0x41; 32], b"ordinary-c8-redo");
    assert_ne!(acknowledgment.request_fingerprint().bytes(), [0; 32]);
    let checkpoint_request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x42; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = world
        .serving()
        .checkpoints()
        .start(checkpoint_request)
        .into_raw()
    else {
        panic!("ordinary checkpoint admission must succeed")
    };
    let checkpoint = match handle.wait() {
        PhysicalCheckpointOutcome::Completed(checkpoint) => checkpoint,
        other => panic!("ordinary checkpoint publication must complete: {other:?}"),
    };
    assert!(checkpoint.retained_wal_tail().segment_count().get() > 0);
    canonical_rooted_mutation_without_acknowledgment(&world, [0x43; 32], b"rooted-c8-redo");
    canonical_durable_wal_attempt_without_execution(&world, [0x44; 32], b"post-checkpoint-c8-redo");
    drop(world);
    retained_root
}

fn ordinary_recovery_declaration(manifest_entries: u64) -> PhysicalRecoveryLimitDeclaration {
    let mut declaration = limit_declaration(2, 8, 2 * 1024 * 1024);
    declaration.manifest_entries = manifest_entries;
    declaration.wal_bytes = 2 * 1024 * 1024;
    declaration.redo_targets = 4_096;
    declaration.redo_bytes = 4 * 1024 * 1024;
    declaration.distinct_pages_and_extents = 4_096;
    declaration.operation_bindings = 4_096;
    declaration.staging_bytes = 32 * 1024 * 1024;
    declaration.recovery_memory_bytes = 32 * 1024 * 1024;
    declaration.dirty_frames = 4_096;
    declaration.publication_effects = 64;
    declaration.observation_bytes = 32 * 1024 * 1024;
    declaration
}

fn ordinary_recovery_limits(
    manifest_entries: u64,
) -> worth_store_recovery_runtime::PhysicalRecoveryLimits {
    worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(ordinary_recovery_declaration(
        manifest_entries,
    ))
    .unwrap()
}

fn selected_ordinary_recovery(
    root: &Path,
) -> worth_store_recovery_runtime::SelectedPhysicalRecovery {
    admitted_recovery_with_limits(root, ordinary_recovery_limits(4_096))
        .discover()
        .unwrap()
        .select()
        .unwrap()
}

fn only_artifact(directory: PathBuf, predicate: impl Fn(&str) -> bool) -> PathBuf {
    let matches = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(&predicate)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "fixture must identify one governed artifact: {matches:?}"
    );
    matches.into_iter().next().unwrap()
}

#[test]
fn selected_checkpoint_becomes_one_effect_free_immutable_plan() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_secured_synthetic_checkpoint(&root, store);

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

    let PhysicalRecoveryOutcome::Refused(cancelled) = planned.cancel_before_execution() else {
        panic!("an explicit pre-execution cancellation is a refusal")
    };
    assert_eq!(cancelled.recovery_effects(), 0);
}

#[test]
fn an_unsecured_checkpoint_cannot_cross_the_phase_four_boundary() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_synthetic_checkpoint(&root, store);

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
        blocked.kind,
        worth_store_recovery_runtime::PhysicalRecoveryBlockKind::BindingFreshness
    );
    assert_eq!(
        blocked.evidence().planning_denial,
        Some(PhysicalRecoveryPlanningDenial::BindingFreshness(
            StoreRecoveryBindingSampleDenial::MissingCheckpointSecurityBinding,
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
    assert_eq!(fates[0].fate(), RecoveryOperationFate::AcknowledgedDurable);
    assert_eq!(
        fates[1].fate(),
        RecoveryOperationFate::DurableUnacknowledged
    );
    assert_eq!(fates[2].fate(), RecoveryOperationFate::Indeterminate);
    let decisions = planned.redo_plan().resolved_decisions().collect::<Vec<_>>();
    assert_eq!(decisions.len(), 2);
    assert_eq!(
        decisions[0].kind(),
        PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn
    );
    assert_eq!(decisions[0].record().lsn().get(), 2);
    assert_eq!(decisions[1].kind(), PhysicalRedoDecisionKind::Apply);
    assert_eq!(decisions[1].record().lsn().get(), 3);
    assert_eq!(
        decisions[1].target().identity(),
        PhysicalRedoTargetIdentity::InlinePage {
            segment: 1,
            page: 1,
            generation: 3,
        }
    );
    let cost = planned.plan_cost();
    assert_eq!(cost.redo_targets(), 2);
    assert_eq!(cost.redo_bytes(), 34_258);
    assert_eq!(cost.distinct_targets(), 2);
    assert_eq!(cost.operation_bindings(), 3);
    assert_eq!(cost.observation_reads(), 7);
    assert_eq!(cost.observation_bytes(), 73_238);
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
        RecordArtifactFile::CatalogCandidate { publication }
    );
    assert!(matches!(
        protocol.previous_candidate(),
        RecordArtifactFile::RootSelectorCandidate {
            publication: candidate,
            ..
        } if candidate == publication
    ));
    assert!(matches!(
        protocol.current_candidate(),
        RecordArtifactFile::RootSelectorCandidate {
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
    publication_assertions::assert_exact_publication_plan(&planned);
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

    let PhysicalRecoveryOutcome::Refused(cancelled) = planned.cancel_before_execution() else {
        panic!("an ordinary plan remains effect-free before execution")
    };
    assert_eq!(cancelled.recovery_effects(), 0);
}
