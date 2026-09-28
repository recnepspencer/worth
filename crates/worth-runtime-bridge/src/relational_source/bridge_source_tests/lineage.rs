use std::sync::{Arc, Mutex};

use crate::facade::{
    BridgeContinuityAuthorityBasis, BridgeHistoricalLineageAuthority,
    BridgeHistoricalLineagePacket, BridgeHistoricalResolvedRecordIdentity, BridgeLineageContext,
    BridgeMappingContext, BridgeRouteRequest, CommittedPatchSource,
    RelationalCommittedPatchRequest, RuntimeBridge, TruthBranchIdentity, TruthCommitIdentity,
    TruthSnapshotIdentity,
};
use crate::relational_source::relational_test_support::{
    changed_entities, create_entity_outcome, replace_entity_on_branch, replacement_successor,
};
use worth_relational::facade::history::BranchId;

use super::super::RuntimeBridgeRelationalSource;
use super::support::{runtime_bridge_for_envelope, runtime_with_test_schema};

#[test]
fn runtime_bridge_lineage_source_resolves_real_relational_history() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "source");
    let entity = changed_entities(&created)[0];

    let replaced =
        replace_entity_on_branch(&runtime, entity, "replacement", BranchId("main".to_owned()));
    let latest_bundle = runtime
        .publication()
        .latest_bundle()
        .expect("runtime publication bundle")
        .clone();
    let branch_identity = runtime
        .branch_identity(&latest_bundle.commit.branch_id)
        .expect("lineage branch identity");
    let expected_successor_record_identities = vec![
        BridgeHistoricalResolvedRecordIdentity::from_relational_record(
            super::super::identities::record_ref_identity(
                &worth_relational::facade::transactions::RecordRef::Entity(replacement_successor(
                    &replaced, entity,
                )),
            ),
        ),
    ];

    let runtime = Arc::new(runtime);
    let source = RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(&runtime), "model")
        .expect("test graph role");
    let (_, basis) = source
        .observe_branch_basis(&branch_identity)
        .expect("owner-admitted lineage basis");
    let lease = source
        .retain_branch_basis_for_bridge(&basis)
        .expect("retained lineage observation");
    let expected_snapshot_identity = lease.snapshot_identity().clone();
    let latest_commit_identity = RelationalCommittedPatchRequest::new(
        TruthCommitIdentity::from_relational_commit_id(latest_bundle.commit.commit_id.0),
    );
    let envelope = source
        .load_committed_patch(latest_commit_identity.clone())
        .expect("runtime bridge committed patch");
    let bridge = runtime_bridge_for_envelope(source.clone(), &envelope);
    let lineage_context = BridgeMappingContext::default().with_lineage_context(
        BridgeLineageContext::new(BridgeContinuityAuthorityBasis::new(
            TruthBranchIdentity::from_relational_branch_id("main"),
            expected_snapshot_identity.clone(),
        )),
    );
    let planned_route = bridge
        .plan_committed_patch_with_mapping_context(
            BridgeRouteRequest::for_commit(latest_commit_identity.commit_identity().clone()),
            lineage_context,
        )
        .expect("lineage-context route should plan");
    let continuity_requests = bridge
        .plan_continuity_requests_from_planned_route(&planned_route)
        .expect("continuity requests should derive native prior slices");
    let lineage_packet = bridge
        .plan_historical_lineage_packet(&continuity_requests)
        .expect("historical lineage packet should resolve");
    let authority = lineage_packet
        .entries()
        .iter()
        .map(|entry| entry.lineage_authority())
        .find(|authority| !authority.traversed_event_ids().is_empty())
        .expect("lineage packet entry with traversed replacement history");

    assert_eq!(
        authority.branch_identity().relational_branch_id(),
        Some("main")
    );
    assert_eq!(authority.snapshot_identity(), &expected_snapshot_identity);
    assert_eq!(authority.canonical_resolved_lineage_identities().len(), 1);
    assert_eq!(
        authority.canonical_resolved_record_identities(),
        expected_successor_record_identities.as_slice()
    );
    assert_eq!(authority.traversed_event_ids().len(), 1);
}

#[test]
fn retained_lineage_observation_excludes_later_same_branch_replacement() {
    let fixture = retained_lineage_fixture();
    let before = lineage_authority_with_events(
        &plan_lineage_packet(
            &fixture.bridge,
            fixture.commit_identity.clone(),
            TruthBranchIdentity::from_relational_branch_id("main"),
            fixture.snapshot.clone(),
        )
        .expect("retained lineage basis should resolve before branch movement"),
    );

    replace_entity_on_branch(
        &fixture.runtime.lock().unwrap(),
        fixture.successor,
        "later-replacement",
        BranchId("main".to_owned()),
    );

    let after = lineage_authority_with_events(
        &plan_lineage_packet(
            &fixture.bridge,
            fixture.commit_identity,
            TruthBranchIdentity::from_relational_branch_id("main"),
            fixture.snapshot,
        )
        .expect("retained lineage basis should remain readable after branch movement"),
    );
    assert_eq!(after, before);
}

#[test]
fn continuity_lineage_denies_mixed_branch_and_snapshot_axes() {
    let fixture = retained_lineage_fixture();
    let denial = plan_lineage_packet(
        &fixture.bridge,
        fixture.commit_identity,
        TruthBranchIdentity::from_relational_branch_id("feature"),
        fixture.snapshot,
    )
    .expect_err("a branch identity from outside the retained observation must deny");
    assert!(
        denial.contains("branch"),
        "mixed-axis denial lost its branch mismatch reason: {denial}"
    );
}

struct RetainedLineageFixture {
    runtime: Arc<Mutex<worth_relational::facade::runtime::RelationalRuntime>>,
    bridge: RuntimeBridge,
    commit_identity: TruthCommitIdentity,
    snapshot: TruthSnapshotIdentity,
    successor: worth_relational::facade::identity::EntityId,
    _lease: super::super::RelationalBridgeObservationLease,
}

fn retained_lineage_fixture() -> RetainedLineageFixture {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "source");
    let entity = changed_entities(&created)[0];
    let replaced =
        replace_entity_on_branch(&runtime, entity, "replacement", BranchId("main".to_owned()));
    let successor = replacement_successor(&replaced, entity);
    let bundle = runtime
        .publication()
        .latest_bundle()
        .expect("replacement publication bundle")
        .clone();
    let branch_identity = runtime
        .branch_identity(&bundle.commit.branch_id)
        .expect("retained lineage branch identity");
    let runtime = Arc::new(Mutex::new(runtime));
    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), "model")
            .expect("shared lineage graph role");
    let (_, basis) = source
        .observe_branch_basis(&branch_identity)
        .expect("owner-admitted retained lineage basis");
    let lease = source
        .retain_branch_basis_for_bridge(&basis)
        .expect("retained lineage observation");
    let snapshot = lease.snapshot_identity().clone();
    let commit_identity = TruthCommitIdentity::from_relational_commit_id(bundle.commit.commit_id.0);
    let envelope = source
        .load_committed_patch(RelationalCommittedPatchRequest::new(
            commit_identity.clone(),
        ))
        .expect("retained lineage committed patch");
    let bridge = runtime_bridge_for_envelope(source, &envelope);
    RetainedLineageFixture {
        runtime,
        bridge,
        commit_identity,
        snapshot,
        successor,
        _lease: lease,
    }
}

fn plan_lineage_packet(
    bridge: &RuntimeBridge,
    commit_identity: TruthCommitIdentity,
    branch: TruthBranchIdentity,
    snapshot: TruthSnapshotIdentity,
) -> Result<BridgeHistoricalLineagePacket, String> {
    let lineage_context = BridgeMappingContext::default().with_lineage_context(
        BridgeLineageContext::new(BridgeContinuityAuthorityBasis::new(branch, snapshot)),
    );
    let planned_route = bridge
        .plan_committed_patch_with_mapping_context(
            BridgeRouteRequest::for_commit(commit_identity),
            lineage_context,
        )
        .map_err(|error| error.to_string())?;
    let continuity_requests = bridge
        .plan_continuity_requests_from_planned_route(&planned_route)
        .map_err(|error| error.to_string())?;
    bridge
        .plan_historical_lineage_packet(&continuity_requests)
        .map_err(|error| error.to_string())
}

fn lineage_authority_with_events(
    packet: &BridgeHistoricalLineagePacket,
) -> BridgeHistoricalLineageAuthority {
    packet
        .entries()
        .iter()
        .map(|entry| entry.lineage_authority())
        .find(|authority| !authority.traversed_event_ids().is_empty())
        .expect("lineage packet entry with traversed replacement history")
        .clone()
}
