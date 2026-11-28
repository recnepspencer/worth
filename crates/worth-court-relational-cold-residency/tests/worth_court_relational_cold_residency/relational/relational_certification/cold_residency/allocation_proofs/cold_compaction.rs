//! Cold-reopened retention-floor compaction `#[global_allocator]` drop proof
//! (`[P1-10]` + `[P1-6]` + `[P1-8]`): proves real heap release on a cold-
//! reopened runtime during `advance_naming_retention_floor` AND verifies that
//! capturing + acknowledging compacted `C2` shrinks cold naming pages on disk
//! below `C1` and releases pre-ACK overlays on consolidation.
use std::cell::Cell;
use std::sync::Arc;

use tracking_allocator::AllocationGroupToken;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    aggregate_identity::AggregateMemberRole,
    durability::{
        RecoveryVerificationMode, RelationalCheckpointArtifactPolicy, RelationalCheckpointMember,
        RelationalDurableColdSource,
    },
    history::CommitId,
    lineage::{PersistentNameKey, PersistentNameSelection},
};
use worth_testkit_relational::durable_medium::TestDurableMedium;

use super::{
    ensure_tracker_installed, fixture, reset_counters, ALLOC_BYTES, DEALLOC_BYTES, LIVE_BYTES,
};

pub(super) fn churn_rounds(
    request: &ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    branch: &worth_relational::facade::history::BranchId,
    anchor_e: worth_relational::facade::identity::EntityId,
    anchor_n: worth_relational::facade::lineage::PersistentNameIdentity,
    prefix: &str,
    rounds: usize,
) -> (
    worth_relational::facade::lineage::PersistentNameIdentity,
    CommitId,
) {
    let mut keep_name = anchor_n;
    let mut floor = CommitId(1);
    let keep_from = rounds.saturating_sub(5);
    for step in 0..rounds {
        let keys: Vec<String> = (0..6).map(|j| format!("{prefix}-{step:03}-{j}")).collect();
        let (entities, names) = fixture::create_named_batch(request, runtime, branch, &keys);
        let mut members = vec![(anchor_n, AggregateMemberRole::Anchor)];
        for &n in &names {
            members.push((n, AggregateMemberRole::Interior));
        }
        fixture::publish_group(request, runtime, branch, anchor_e, vec![members]);
        if step < keep_from {
            fixture::delete_entities(request, runtime, branch, &entities);
        } else {
            keep_name = names[0];
        }
        if step == keep_from {
            floor = runtime
                .history()
                .historical_latest_commit()
                .unwrap()
                .commit_id;
        }
    }
    (keep_name, floor)
}

#[test]
fn cold_reopened_retention_floor_compaction_drops_global_allocator_live_bytes() {
    ensure_tracker_installed();
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);

    // 1. Build and checkpoint churn-heavy `C1` outside the tracked group, then drop producer.
    let medium = TestDurableMedium::new();
    let producer = fixture::durable_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();
    let (anchor_e, anchor_n) =
        fixture::create_named_batch(request, &producer, &main, &["cold-anchor".into()]);
    let _ = churn_rounds(
        request,
        &producer,
        &main,
        anchor_e[0],
        anchor_n[0],
        "c1",
        60,
    );
    let medium_source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    let c1 = producer
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: Some(medium_source.clone()),
            },
        )
        .unwrap();
    medium.file_checkpoint(c1.clone());
    drop(producer);

    // 2. Reopen cold from `C1` and track live heap across post-reopen churn + compaction.
    let mut token = AllocationGroupToken::register().expect("cold compaction group");
    let id = token.id().as_usize().get();
    let guard = token.enter();
    let _reset = reset_counters(id);

    let mut reopened = fixture::durable_runtime(&medium, u64::MAX / 4);
    let plan = reopened
        .durability()
        .recovery_plan_from_source(
            request,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    reopened
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    let obs_at_reopen = reopened
        .cold_naming_observation()
        .expect("cold-reopened runtime must be backed by ColdNamingStore");
    assert_eq!(obs_at_reopen.resident_pages, 0);

    let (keep_name, floor) = churn_rounds(
        request,
        &reopened,
        &main,
        anchor_e[0],
        anchor_n[0],
        "c2",
        70,
    );

    // Measure uncompacted CHK24 naming footprint at this exact commit outside tracking.
    drop(guard);
    let c1_members: Vec<_> = c1
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect();
    let naming_bytes =
        |ck: &worth_relational::facade::durability::RelationalNativeCheckpoint| -> u64 {
            let dir = super::super::selected_directory::read_native(ck);
            dir.naming
                .branches
                .iter()
                .flat_map(|b| &b.pages)
                .chain(&dir.naming.group_log)
                .map(|p| p.member.length)
                .sum()
        };
    let c2_uncompacted = reopened
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &c1_members,
                source: Some(medium_source.clone()),
            },
        )
        .unwrap();
    let uncompacted_naming_bytes = naming_bytes(&c2_uncompacted);
    drop(c2_uncompacted);

    // Re-enter tracking to measure the global allocator live-byte drop from compaction.
    let guard = token.enter();
    let live_before = LIVE_BYTES.with(Cell::get);
    let alloc_before = ALLOC_BYTES.with(Cell::get);
    let dealloc_before = DEALLOC_BYTES.with(Cell::get);
    reopened
        .advance_naming_retention_floor(&main, floor)
        .unwrap();
    let live_after = LIVE_BYTES.with(Cell::get);
    let bytes_freed = live_before - live_after;
    eprintln!(
        "[M4.3f-ALLOC] cold-reopened compaction: live_before={} B, live_after={} B, freed={} B (dealloc={} B, alloc={} B)",
        live_before,
        live_after,
        bytes_freed,
        DEALLOC_BYTES.with(Cell::get).saturating_sub(dealloc_before),
        ALLOC_BYTES.with(Cell::get).saturating_sub(alloc_before),
    );
    assert!(
        bytes_freed > 16 * 1024,
        "cold-reopened retention-floor compaction must drop global allocator live bytes, freed only {bytes_freed} B"
    );

    // 3. Capture & ACK compacted `C2` and verify cold naming pages shrink below
    //    the uncompacted snapshot at the same commit while ACK consolidation frees overlays.
    drop(guard);
    let c2 = reopened
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &c1_members,
                source: Some(medium_source),
            },
        )
        .unwrap();
    let c2_naming_bytes = naming_bytes(&c2);
    medium.file_checkpoint(c2.clone());

    let guard = token.enter();
    let dealloc_before_ack = DEALLOC_BYTES.with(Cell::get);
    request
        .run_native_with_checkpoint(|context| c2.acknowledge_selected(request, context))
        .unwrap()
        .0
        .unwrap();
    let ack_dealloc = DEALLOC_BYTES
        .with(Cell::get)
        .saturating_sub(dealloc_before_ack);
    eprintln!(
        "[M4.3f-ALLOC] cold-reopened C2 ACK: uncompacted_naming={} B -> compacted_naming={} B, ack_dealloc={} B",
        uncompacted_naming_bytes, c2_naming_bytes, ack_dealloc
    );
    assert!(
        c2_naming_bytes < uncompacted_naming_bytes,
        "compacted C2 naming pages ({c2_naming_bytes} B) must shrink below uncompacted ({uncompacted_naming_bytes} B)"
    );
    assert!(
        ack_dealloc > 100 * 1024,
        "C2 ACK consolidation must deallocate pre-ACK deltas and C1 cache pages, got {ack_dealloc} B"
    );

    let head = fixture::observe_branch(&reopened, &main);
    reopened
        .with_persistent_names_at_observation(request, &head.observation(), |view| {
            assert!(view
                .group_of_member(fixture::GROUP_SCHEME, &keep_name)
                .is_some());
            assert!(view
                .resolve(
                    PersistentNameKey::Entity(anchor_n[0]),
                    &PersistentNameSelection::Exact,
                )
                .result
                .is_ok());
        })
        .unwrap();

    drop((c2, reopened, _reset, guard));
}
