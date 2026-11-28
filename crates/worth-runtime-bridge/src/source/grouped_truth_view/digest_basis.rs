use sha2::{Digest, Sha256};

use crate::source::row_set::BridgeMaterializedRowSetArtifact;

use super::{BridgeGroupedMemberRow, BridgeGroupedTruthViewDigest};
use crate::source::grouped_contract::GroupedProjectionContract;

pub(super) fn grouped_truth_view_digest_from_materialized_members(
    row_set: &BridgeMaterializedRowSetArtifact,
    contract: &GroupedProjectionContract,
    members: &[BridgeGroupedMemberRow],
) -> BridgeGroupedTruthViewDigest {
    BridgeGroupedTruthViewDigest::from_canonical_basis(grouped_truth_view_canonical_basis(
        row_set, contract, members,
    ))
}

fn grouped_truth_view_canonical_basis(
    row_set: &BridgeMaterializedRowSetArtifact,
    contract: &GroupedProjectionContract,
    members: &[BridgeGroupedMemberRow],
) -> String {
    let mut canonical_basis = format!(
        "bridge-grouped-truth-view|truth-view={}|snapshot={}|grouping={}|identity-binding={}|grouping-binding={}",
        row_set.truth_view_digest(),
        row_set.basis_snapshot_identity().canonical_label(),
        contract.native_grouping_aspect_key().as_str(),
        contract.identity_binding().aspect_key(),
        contract.grouping_binding().aspect_key()
    );
    for member in members {
        canonical_basis.push_str("|member=");
        canonical_basis.push_str(member.row_identity().as_str());
        canonical_basis.push_str("|identity-value=");
        canonical_basis.push_str(member.identity_value_canonical_basis());
        canonical_basis.push_str("|lane-value=");
        canonical_basis.push_str(member.lane().validated_value_canonical_basis());
    }
    canonical_basis
}

impl BridgeGroupedTruthViewDigest {
    fn from_canonical_basis(canonical_basis: String) -> Self {
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self(format!("bridge-grouped-truth-view:sha256:{digest:x}").into())
    }
}
