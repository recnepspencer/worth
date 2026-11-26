//! Value-level consistency checks mint no root or publication authority.
use super::*;
use contracts::identity::data::primitives::PartitionId;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use worth_relational_storage::internal_port::storage::overlay::partition::PartitionState;
use worth_relational_storage::internal_port::storage::residency::kind_commitment::RelationalRegionKindCommitment;
use worth_relational_storage::internal_port::storage::residency::record_directory::portable::RegionRecordDirectoryFixturePort as _;
use worth_relational_storage::internal_port::storage::residency::retention::RelationalRegionRetention;
use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::{
    EntityArena, RelationArena,
};

#[test]
fn every_leaf_disagreement_refuses_with_the_selected_region_and_capture_posture() {
    let (selected, matching) = leaf_values();
    validate_value(
        &selected,
        &matching,
        worth_relational_branch::internal_port::branch::root::capture::with_reconstruction_fixture_for_test(|lane| {
            selected.directory.portable_oracle_for_test(lane)
        })
        .unwrap()
        .as_ref()
            == matching.directory.as_ref(),
    )
    .unwrap();
    let disagreements: [fn(&mut RelationalCheckpointRegion); 9] = [
        |leaf| leaf.partition_id = PartitionId(11),
        |leaf| leaf.content_digest[0] ^= 1,
        |leaf| leaf.live_counts.entities += 1,
        |leaf| leaf.live_counts.relations += 1,
        |leaf| leaf.entity_slots += 1,
        |leaf| leaf.relation_slots += 1,
        |leaf| leaf.unavailable_records += 1,
        |leaf| disagree_digest(&mut leaf.kind_authority.head),
        |leaf| disagree_digest(&mut leaf.kind_authority.history),
    ];
    for (dimension, disagree) in disagreements.into_iter().enumerate() {
        let mut captured = matching.clone();
        disagree(&mut captured);
        let error = validate_value(
            &selected,
            &captured,
            worth_relational_branch::internal_port::branch::root::capture::with_reconstruction_fixture_for_test(|lane| {
                selected.directory.portable_oracle_for_test(lane)
            })
            .unwrap()
            .as_ref()
                == captured.directory.as_ref(),
        )
        .unwrap_err();
        assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
        assert_eq!(
            error.storage_unavailable,
            Some(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(PartitionId(7)),
                cause: RelationalStorageUnavailableCause::IntegrityMismatch,
            }),
            "leaf dimension {dimension} must name the selected subject"
        );
        assert_eq!(error.context.operation, ErrorOperation::WriteDurableStore);
        assert_eq!(
            error.context.suggested_fix,
            Some(SuggestedFix::InspectDiagnostics)
        );
        assert!(
            error.context.affected_records.is_empty(),
            "a region mismatch does not identify a guilty record"
        );
    }
}

fn leaf_values() -> (RelationalRootRegionCheckpoint, RelationalCheckpointRegion) {
    let partition_id = PartitionId(7);
    let partition = PartitionState {
        partition_id,
        adjacency_policy: contracts::config::data::policies::AdjacencyPolicy {
            backend:
                contracts::config::data::policies::AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        },
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(0),
        relation_arena: RelationArena::with_capacity(0),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    };
    let retention = RelationalRegionRetention::resident(partition);
    let selected = RelationalRootRegionCheckpoint {
        partition_id,
        content_digest: [1; 32],
        kind_commitment: RelationalRegionKindCommitment {
            head: worth_foundational::CanonicalDigestId::new([2; 32]),
            history: worth_foundational::CanonicalDigestId::new([3; 32]),
        },
        directory: std::sync::Arc::new(worth_relational_storage::internal_port::storage::residency::record_directory::RegionRecordDirectory::empty(
            partition_id,
        )),
        live_counts: Default::default(),
        entity_slots: 0,
        relation_slots: 0,
        unavailable_records: 0,
        cell: retention.cell().clone(),
        kind_authority: None,
    };
    let locator = RelationalDurableMemberLocator {
        sha256: [4; 32],
        length: 0,
    };
    let captured = RelationalCheckpointRegion {
        partition_id,
        content_digest: selected.content_digest,
        kind_authority: selected.kind_commitment,
        directory: worth_relational_branch::internal_port::branch::root::capture::with_reconstruction_fixture_for_test(|lane| {
            selected.directory.portable_oracle_for_test(lane)
        })
        .unwrap(),
        live_counts: Default::default(),
        entity_slots: 0,
        relation_slots: 0,
        unavailable_records: 0,
        head: locator,
        history: locator,
        chunks: std::sync::Arc::new(worth_relational_storage::internal_port::storage::residency::chunked::region_placement::RegionChunkPlacement::empty()),
    };
    (selected, captured)
}

fn disagree_digest(digest: &mut worth_foundational::CanonicalDigestId) {
    let mut bytes = *digest.bytes();
    bytes[0] ^= 1;
    *digest = worth_foundational::CanonicalDigestId::new(bytes);
}
