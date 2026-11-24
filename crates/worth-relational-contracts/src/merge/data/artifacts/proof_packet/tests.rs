use super::*;
use crate::history::data::commit_receipt::RelationalCommitReceipt;
use crate::history::data::coordinates::{BranchId, CommitId};
use crate::history::data::merge_branch_basis::{MergeBaseSelectionRule, ResolvedMergeBase};
use crate::identity::data::{EntityId, PartitionId, VersionId};
use crate::merge::data::requests::normalized::{
    RelationalMergeCorrespondencePosture, RelationalMergeSchemaReconciliationPosture,
    RelationalMergeTopologyIntent,
};
use crate::merge::data::requests::raw::MergeIntent;

#[test]
fn streamed_proof_packet_preserves_prior_concatenated_bytes_and_surface_codec() {
    let request = NormalizedRelationalMergeRequest::admit_full_branch(
        BranchId("target".into()),
        BranchId("source-λ".into()),
        MergeIntent::ReconcileIntoTarget,
        RelationalMergeCorrespondencePosture::Advisory,
        RelationalMergeSchemaReconciliationPosture::Participate,
        RelationalMergeTopologyIntent::PreserveTopologySemantics,
    )
    .unwrap();
    let receipt = RelationalCommitReceipt {
        commit_id: CommitId(3),
        version_id: VersionId(4),
        branch_id: BranchId("source-λ".into()),
        parents: vec![CommitId(1)],
    };
    let basis = RelationalMergeBranchBasis {
        source_head: receipt.clone(),
        target_head: receipt.clone(),
        merge_base: ResolvedMergeBase {
            rule: MergeBaseSelectionRule::MaxCommitIdCommonAncestor,
            commit: receipt,
            supporting_left_ancestors: Arc::<[CommitId]>::from([CommitId(1)]).into(),
            supporting_right_ancestors: Arc::<[CommitId]>::from([]).into(),
        },
    };
    let surface = Arc::from([RelationalMergeAdmittedSurfaceRow::new(
        RecordRef::Entity(EntityId::new(PartitionId(1), 2, 1)),
        None,
    )]);
    let packet = RelationalMergeProofPacket::retained_execution_admitted(
        request,
        basis,
        surface,
        "a".repeat(64),
        "b".repeat(64),
        "c".repeat(64),
        "d".repeat(64),
        "e".repeat(64),
        "f".repeat(64),
    );
    let mut prior_surface = b"worth.relational.merge.admitted_surface.v1".to_vec();
    for row in packet.admitted_merge_surface() {
        prior_surface.extend(rmp_serde::to_vec_named(row).unwrap());
    }
    assert_eq!(
        packet.admitted_merge_surface_digest(),
        format!("{:x}", Sha256::digest(prior_surface))
    );
    let mut prior = b"worth.relational.merge.proof_packet.v1".to_vec();
    prior.extend(packet.request().request_digest().as_bytes());
    prior.extend(packet.branch_basis().basis_digest().as_bytes());
    prior.extend(packet.admitted_merge_surface().len().to_string().as_bytes());
    for text in [
        packet.correspondence_witness_digest(),
        packet.schema_reconciliation_witness_digest(),
        packet.strategy_witness_digest(),
        packet.foundational_request_lowering_digest(),
        packet.admitted_merge_surface_digest(),
        packet.planning_digest(),
        packet.execution_digest(),
    ] {
        prior.extend(text.as_bytes());
    }
    prior.push(1);
    assert_eq!(
        packet.packet_digest(),
        format!("{:x}", Sha256::digest(prior))
    );
    let wire = rmp_serde::to_vec_named(&packet).unwrap();
    assert_eq!(
        rmp_serde::from_slice::<RelationalMergeProofPacket>(&wire).unwrap(),
        packet
    );
}
