use sha2::{Digest, Sha256};

use super::BridgeMaterializedRowArtifact;

pub(super) fn row_set_digest_from_materialized_rows(
    planned_truth_view_digest: &str,
    snapshot_identity: &crate::snapshot::TruthSnapshotIdentity,
    rows: &[BridgeMaterializedRowArtifact],
) -> super::BridgeMaterializedRowSetDigest {
    super::BridgeMaterializedRowSetDigest::from_canonical_basis(row_set_canonical_basis(
        planned_truth_view_digest,
        snapshot_identity,
        rows,
    ))
}

fn row_set_canonical_basis(
    planned_truth_view_digest: &str,
    snapshot_identity: &crate::snapshot::TruthSnapshotIdentity,
    rows: &[BridgeMaterializedRowArtifact],
) -> String {
    let mut canonical_basis = format!(
        "bridge-row-set|planned={planned_truth_view_digest}|snapshot={}",
        snapshot_identity.canonical_label()
    );
    for row in rows {
        canonical_basis.push_str("|row=");
        canonical_basis.push_str(row.row_identity().as_str());
        for (field, value) in row.fields() {
            canonical_basis.push_str("|field=");
            canonical_basis.push_str(field.as_str());
            canonical_basis.push_str("|projection=");
            canonical_basis.push_str(value.projection().canonical_basis());
            canonical_basis.push_str("|value=");
            canonical_basis.push_str(value.validated_value_canonical_basis());
        }
    }
    canonical_basis
}

impl super::BridgeMaterializedRowSetDigest {
    fn from_canonical_basis(canonical_basis: String) -> Self {
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self(format!("bridge-row-set:sha256:{digest:x}").into())
    }
}
