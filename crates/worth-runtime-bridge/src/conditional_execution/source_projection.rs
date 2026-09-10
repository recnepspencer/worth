use std::sync::Arc;

use sha2::{Digest, Sha256};

pub(super) fn conditional_source_projection(
    identity: &crate::snapshot::TruthSnapshotIdentity,
) -> Arc<str> {
    let digest = Sha256::digest(identity.as_str().as_bytes());
    Arc::from(format!("bridge-conditional-source:sha256:{digest:x}"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn projection_width_is_independent_of_snapshot_label_width() {
        let short = crate::truth_identity_fixtures::truth_snapshot_fixture("a");
        let long = crate::truth_identity_fixtures::truth_snapshot_fixture("a-much-longer-snapshot");
        let short = super::conditional_source_projection(&short);
        let long = super::conditional_source_projection(&long);

        assert_eq!(short.len(), long.len());
        assert_ne!(short, long);
    }
}
