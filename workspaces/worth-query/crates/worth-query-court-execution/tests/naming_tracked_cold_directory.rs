//! Cold-directory (external-member) checkpoint restore of named and nameless
//! runtimes: the named runtime rebuilds its derived name index eagerly and
//! answers every read like its source; the nameless runtime reads as having
//! no names, so Bridge continuity falls back to record history.

use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalCheckpointArtifactPolicy,
    RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
};
use worth_relational::facade::lineage::{LineageRecordTarget, PersistentNameOfRecord};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::{
    BridgeHistoricalLineageTopology, BridgeHistoricalResolvedRecordIdentity,
    RelationalBridgeRecordIdentityParts,
};

use crate::naming_bridge_planning::{nameless_lifecycle_face_runtime, planned_continuity};
use crate::naming_cold_source_fixture::ExternalColdSource;
use crate::naming_execution_fixture::{
    begin_staged_tx, create_face_intent, execution_lease_for_workers, face_ref, fresh_cad_runtime,
    release_commit_snapshot,
};
use crate::naming_tracked_rebuild::answers;
use crate::naming_tracked_world::TrackedWorld;

/// Restores `source`'s external-member checkpoint into `reopened` through
/// `cold_directory::restore`.
fn cold_restore(
    source: &RelationalRuntime,
    mut reopened: RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
) -> RelationalRuntime {
    let native = source
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .expect("external-member checkpoint captures");
    let position = native.covered_position().expect("covered position");
    let transitions = native
        .covered_branch_transitions()
        .expect("covered transitions");
    let cold = ExternalColdSource::from_native_capture(&native, source, execution);
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            position,
            transitions,
            native,
        )),
        Vec::new(),
        Vec::new(),
        cold.clone(),
    );
    let plan = reopened
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .expect("cold plan builds");
    reopened
        .durability_recovery()
        .recover(execution, plan)
        .expect("cold-directory restore succeeds");
    assert_eq!(
        *cold.region_reads.lock().unwrap(),
        0,
        "record regions stay cold"
    );
    reopened
}

#[test]
fn named_cold_directory_restore_rebuilds_the_name_index_with_full_parity() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::new();
    let (seeded, _) = world.seed(&execution, "occ.seed", &["f1", "f2", "f3"]);
    let (f1, f2, f3) = (seeded[0], seeded[1], seeded[2]);
    let (pieces, _) = world.split(&execution, f1, "occ.cut", &[("a", 1), ("b", 2)]);
    let (a, b) = (pieces[0], pieces[1]);
    world.retire(&execution, f2);
    let (seam, _) = world.seam(&execution, "occ.seam", f3, b);
    let held = [f1, f2, f3, a, b];
    let original = answers(&world.runtime, &execution, &held, seam);
    assert!(!original.dependencies.is_empty());

    let reopened = cold_restore(&world.runtime, fresh_cad_runtime(), &execution);
    assert_eq!(answers(&reopened, &execution, &held, seam), original);
}

#[test]
fn nameless_cold_directory_restore_keeps_bridge_record_history_continuity() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let source = nameless_lifecycle_face_runtime();
    let tx = begin_staged_tx(&source, "nameless", vec![create_face_intent("plain")]);
    let created = source
        .commit_branch_transaction(&execution, tx)
        .expect("a nameless face commits");
    let face = created
        .created_entity(&face_ref("plain"))
        .expect("face created");
    release_commit_snapshot(&source, &created);
    assert!(
        !created.envelope().lineage_events().is_empty(),
        "the commit carries lineage"
    );

    let reopened = Arc::new(cold_restore(
        &source,
        nameless_lifecycle_face_runtime(),
        &execution,
    ));
    let head = reopened
        .admit_branch_basis(&reopened.main_branch_identity())
        .expect("head basis")
        .observation();
    let named = reopened
        .with_persistent_names_at_observation(&execution, &head, |view| {
            view.name_of_record(LineageRecordTarget::Entity(face))
        })
        .expect("a nameless restored runtime admits name reads");
    assert_eq!(named, PersistentNameOfRecord::Unnamed);

    let planned = planned_continuity(&reopened, &created, execution);
    let parts = RelationalBridgeRecordIdentityParts::entity(
        face.partition_id.0,
        face.local_slot.0,
        face.generation.0,
    );
    let (_, selection) = planned
        .iter()
        .find(|(prior, _)| prior.relational_record_identity_parts() == Some(parts))
        .expect("the nameless face has a continuity entry");
    assert_eq!(
        selection.topology(),
        BridgeHistoricalLineageTopology::SingleSuccessor
    );
    assert_eq!(
        selection.canonical_resolved_record_identities(),
        [BridgeHistoricalResolvedRecordIdentity::from_relational_record(parts)].as_slice()
    );
}
