//! Transport descriptions enter fresh native occurrence lookup.
use super::super::protocol::{BankHttpCorrectionCommit, BankHttpCorrectionSource};
use worth_query_host::facade::primary_graph::WorthQueryCorrectionSourceCoordinates;

impl From<BankHttpCorrectionSource> for WorthQueryCorrectionSourceCoordinates {
    fn from(source: BankHttpCorrectionSource) -> Self {
        Self::from_untrusted_fields(
            source.world_owner,
            source.world_commit,
            source.relational.branch_id,
            source.relational.commit_id,
            source.relational.version_id,
            source.relational.parents,
        )
    }
}

impl From<WorthQueryCorrectionSourceCoordinates> for BankHttpCorrectionSource {
    fn from(source: WorthQueryCorrectionSourceCoordinates) -> Self {
        Self {
            world_owner: source.world_owner(),
            world_commit: source.world_commit(),
            relational: BankHttpCorrectionCommit {
                commit_id: source.relational().commit_id.0,
                version_id: source.relational().version_id.0,
                branch_id: source.relational().branch_id.0.clone(),
                parents: source.relational().parents.iter().map(|id| id.0).collect(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correction_wire_fields_keep_their_native_coordinate_meaning() {
        let wire = serde_json::json!({
            "world_owner": 7, "world_commit": 19,
            "relational": {
                "branch_id": "product:fork", "commit_id": 23,
                "version_id": 29, "parents": [17]
            }
        });
        let description: BankHttpCorrectionSource = serde_json::from_value(wire.clone()).unwrap();
        let native = WorthQueryCorrectionSourceCoordinates::from(description);
        assert_eq!((native.world_owner(), native.world_commit()), (7, 19));
        assert_eq!(native.relational().branch_id.0, "product:fork");
        assert_eq!(native.relational().commit_id.0, 23);
        assert_eq!(native.relational().version_id.0, 29);
        assert_eq!(
            native
                .relational()
                .parents
                .iter()
                .map(|id| id.0)
                .collect::<Vec<_>>(),
            vec![17]
        );
        assert_eq!(
            serde_json::to_value(BankHttpCorrectionSource::from(native)).unwrap(),
            wire
        );
    }
}
