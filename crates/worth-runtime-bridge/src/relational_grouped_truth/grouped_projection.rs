use crate::snapshot::TruthSnapshotIdentity;
use crate::source::{GroupedProjectionMemberSource, GroupedProjectionSource};
use worth_foundational::facade::{AspectKey, AspectValue};

use super::canonical_digest::grouped_projection_digest;

use super::row_set::{
    RelationalAuthoritativeRowSetArtifact, RelationalRowIdentity, RelationalRowSetDigest,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalGroupedProjectionContract {
    grouping_aspect: AspectKey,
    identity_binding_aspect_key: AspectKey,
    grouping_binding_aspect_key: AspectKey,
}

impl RelationalGroupedProjectionContract {
    pub fn new(
        grouping_aspect: AspectKey,
        identity_binding_aspect_key: AspectKey,
        grouping_binding_aspect_key: AspectKey,
    ) -> Self {
        Self {
            grouping_aspect,
            identity_binding_aspect_key,
            grouping_binding_aspect_key,
        }
    }

    pub fn grouping_aspect(&self) -> &AspectKey {
        &self.grouping_aspect
    }

    pub fn identity_binding_aspect_key(&self) -> &AspectKey {
        &self.identity_binding_aspect_key
    }

    pub fn grouping_binding_aspect_key(&self) -> &AspectKey {
        &self.grouping_binding_aspect_key
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RelationalGroupedMemberRow {
    row_identity: RelationalRowIdentity,
    identity_value: AspectValue,
    grouping_value: AspectValue,
}

impl RelationalGroupedMemberRow {
    pub fn row_identity(&self) -> &RelationalRowIdentity {
        &self.row_identity
    }

    pub fn identity_value(&self) -> &AspectValue {
        &self.identity_value
    }

    pub fn grouping_value(&self) -> &AspectValue {
        &self.grouping_value
    }
}

impl GroupedProjectionMemberSource for RelationalGroupedMemberRow {
    fn row_identity(&self) -> &str {
        self.row_identity.as_str()
    }

    fn identity_value(&self) -> &AspectValue {
        &self.identity_value
    }

    fn grouping_value(&self) -> &AspectValue {
        &self.grouping_value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalGroupedProjectionDigest(String);

impl RelationalGroupedProjectionDigest {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn from_canonical_bytes(bytes: &[u8]) -> Self {
        Self(super::canonical_digest::digest_with_prefix(
            "relational-grouped-projection",
            bytes,
        ))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RelationalGroupedProjectionArtifact {
    row_set_digest: RelationalRowSetDigest,
    snapshot_identity: TruthSnapshotIdentity,
    contract: RelationalGroupedProjectionContract,
    members: Vec<RelationalGroupedMemberRow>,
    digest: RelationalGroupedProjectionDigest,
}

impl RelationalGroupedProjectionArtifact {
    pub fn row_set_digest(&self) -> &RelationalRowSetDigest {
        &self.row_set_digest
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn contract(&self) -> &RelationalGroupedProjectionContract {
        &self.contract
    }

    pub fn members(&self) -> &[RelationalGroupedMemberRow] {
        &self.members
    }

    pub fn digest(&self) -> &RelationalGroupedProjectionDigest {
        &self.digest
    }
}

impl GroupedProjectionSource for RelationalGroupedProjectionArtifact {
    type Member = RelationalGroupedMemberRow;

    fn basis_snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot_identity
    }

    fn grouping_aspect_key(&self) -> &AspectKey {
        self.contract.grouping_aspect()
    }

    fn identity_binding_aspect_key(&self) -> &AspectKey {
        self.contract.identity_binding_aspect_key()
    }

    fn grouping_binding_aspect_key(&self) -> &AspectKey {
        self.contract.grouping_binding_aspect_key()
    }

    fn members(&self) -> &[Self::Member] {
        &self.members
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalGroupedTruthError {
    PacketResultShapeMismatch,
    AspectValueDecodeFailure {
        request_key: String,
    },
    UntypedRelationalRowIdentity {
        request_key: String,
    },
    UntypedRelationalSnapshotIdentity,
    MissingIdentityAspect {
        row_identity: RelationalRowIdentity,
        aspect_key: AspectKey,
    },
    MissingGroupingAspect {
        row_identity: RelationalRowIdentity,
        aspect_key: AspectKey,
    },
}

pub fn project_relational_grouped_truth(
    row_set: &RelationalAuthoritativeRowSetArtifact,
    contract: RelationalGroupedProjectionContract,
) -> Result<RelationalGroupedProjectionArtifact, RelationalGroupedTruthError> {
    let identity_aspect = contract.identity_binding_aspect_key();
    let grouping_aspect = contract.grouping_binding_aspect_key();

    let mut members = Vec::with_capacity(row_set.rows().len());
    for row in row_set.rows() {
        let Some(identity_value) = row
            .projected_aspect_values()
            .get(identity_aspect)
            .and_then(|value| value.scalar_value())
            .cloned()
        else {
            return Err(RelationalGroupedTruthError::MissingIdentityAspect {
                row_identity: row.row_identity().clone(),
                aspect_key: contract.identity_binding_aspect_key().clone(),
            });
        };
        let Some(grouping_value) = row
            .projected_aspect_values()
            .get(grouping_aspect)
            .and_then(|value| value.scalar_value())
            .cloned()
        else {
            return Err(RelationalGroupedTruthError::MissingGroupingAspect {
                row_identity: row.row_identity().clone(),
                aspect_key: contract.grouping_binding_aspect_key().clone(),
            });
        };

        members.push(RelationalGroupedMemberRow {
            row_identity: row.row_identity().clone(),
            identity_value: identity_value.clone(),
            grouping_value: grouping_value.clone(),
        });
    }

    let digest = grouped_projection_digest(
        row_set.digest(),
        row_set.snapshot_identity(),
        &contract,
        &members,
    )?;

    Ok(RelationalGroupedProjectionArtifact {
        row_set_digest: row_set.digest().clone(),
        snapshot_identity: row_set.snapshot_identity().clone(),
        contract,
        members,
        digest,
    })
}

#[cfg(test)]
mod tests {
    use crate::facade::{
        RelationalBridgeRecordIdentityParts, RelationalBridgeSnapshotIdentityParts,
        SnapshotReadContract, SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord,
        SnapshotReadRequest, TruthSnapshotIdentity,
    };
    use worth_foundational::facade::{AspectKey, AspectValue, ScalarAspectType};

    use super::{project_relational_grouped_truth, RelationalGroupedProjectionContract};
    use crate::relational_grouped_truth::materialize_relational_authoritative_row_set;

    #[test]
    fn relational_grouped_projection_preserves_member_and_grouping_pairing() {
        let packet = SnapshotReadPacket::new(vec![
            string_read(
                RelationalBridgeRecordIdentityParts::entity(0, 1, 1),
                "identity.id",
            ),
            string_read(
                RelationalBridgeRecordIdentityParts::entity(0, 1, 1),
                "status.lane",
            ),
            string_read(
                RelationalBridgeRecordIdentityParts::entity(0, 2, 1),
                "identity.id",
            ),
            string_read(
                RelationalBridgeRecordIdentityParts::entity(0, 2, 1),
                "status.lane",
            ),
        ]);
        let result = SnapshotReadPacketResult::new(
            test_snapshot_identity(),
            vec![
                read_record(&packet, 0, AspectValue::String("task-1".into())),
                read_record(&packet, 1, AspectValue::String("todo".into())),
                read_record(&packet, 2, AspectValue::String("task-2".into())),
                read_record(&packet, 3, AspectValue::String("doing".into())),
            ],
        );
        let row_set = materialize_relational_authoritative_row_set(&packet, &result).unwrap();

        let grouped = project_relational_grouped_truth(
            &row_set,
            RelationalGroupedProjectionContract::new(
                AspectKey::new("status").unwrap(),
                AspectKey::new("identity.id").unwrap(),
                AspectKey::new("status.lane").unwrap(),
            ),
        )
        .unwrap();

        assert_eq!(grouped.members().len(), 2);
        assert_eq!(
            grouped.members()[0].row_identity().parts(),
            RelationalBridgeRecordIdentityParts::entity(0, 1, 1)
        );
        assert_eq!(
            grouped.members()[0].identity_value(),
            &AspectValue::String("task-1".into())
        );
    }

    #[test]
    fn relational_grouped_projection_missing_identity_error_carries_typed_row_identity() {
        let packet = SnapshotReadPacket::new(vec![string_read(
            RelationalBridgeRecordIdentityParts::entity(0, 1, 1),
            "status.lane",
        )]);
        let result = SnapshotReadPacketResult::new(
            test_snapshot_identity(),
            vec![read_record(&packet, 0, AspectValue::String("todo".into()))],
        );
        let row_set = materialize_relational_authoritative_row_set(&packet, &result).unwrap();

        let error = project_relational_grouped_truth(
            &row_set,
            RelationalGroupedProjectionContract::new(
                AspectKey::new("status").unwrap(),
                AspectKey::new("identity.id").unwrap(),
                AspectKey::new("status.lane").unwrap(),
            ),
        )
        .expect_err("missing identity aspect should be denied");

        match error {
            super::RelationalGroupedTruthError::MissingIdentityAspect {
                row_identity,
                aspect_key,
            } => {
                assert_eq!(
                    row_identity.parts(),
                    RelationalBridgeRecordIdentityParts::entity(0, 1, 1)
                );
                assert_eq!(aspect_key, AspectKey::new("identity.id").unwrap());
            }
            other => panic!("expected missing identity aspect error, got {other:?}"),
        }
    }

    fn read_record(
        packet: &SnapshotReadPacket,
        index: usize,
        value: impl Into<crate::facade::SnapshotReadValue>,
    ) -> SnapshotReadRecord {
        SnapshotReadRecord::for_request(&packet.reads()[index], value)
    }

    fn string_read(
        entity_identity: RelationalBridgeRecordIdentityParts,
        aspect: &str,
    ) -> SnapshotReadRequest {
        SnapshotReadRequest::for_relational_record(
            entity_identity,
            SnapshotReadContract::scalar(aspect_key(aspect), ScalarAspectType::String),
        )
    }

    fn aspect_key(value: &str) -> AspectKey {
        AspectKey::new(value).expect("valid test aspect key")
    }

    fn test_snapshot_identity() -> TruthSnapshotIdentity {
        TruthSnapshotIdentity::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts::new(
            1, 1,
        ))
    }
}
