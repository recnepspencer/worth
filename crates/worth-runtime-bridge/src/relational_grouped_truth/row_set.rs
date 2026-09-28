use std::collections::BTreeMap;

use crate::relational_source::identity_parts::{
    RelationalBridgeRecordIdentityKind, RelationalBridgeRecordIdentityParts,
};
use crate::snapshot::{
    SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadValue, TruthSnapshotIdentity,
};
use worth_foundational::facade::AspectKey;

use super::canonical_digest::row_set_digest;
use super::grouped_projection::RelationalGroupedTruthError;
use super::snapshot_aspect_reads::decode_snapshot_aspect_read_value;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RelationalRowIdentity {
    parts: RelationalBridgeRecordIdentityParts,
    diagnostic_label: String,
}

impl RelationalRowIdentity {
    pub fn as_str(&self) -> &str {
        &self.diagnostic_label
    }

    pub fn parts(&self) -> RelationalBridgeRecordIdentityParts {
        self.parts
    }

    pub(crate) fn new(parts: RelationalBridgeRecordIdentityParts) -> Self {
        Self {
            parts,
            diagnostic_label: row_identity_label(parts),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalRowSetDigest(String);

impl RelationalRowSetDigest {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn from_canonical_bytes(bytes: &[u8]) -> Self {
        Self(super::canonical_digest::digest_with_prefix(
            "relational-row-set",
            bytes,
        ))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RelationalProjectedAspectValueSet {
    values: BTreeMap<AspectKey, SnapshotReadValue>,
}

impl RelationalProjectedAspectValueSet {
    pub fn get(&self, aspect_key: &AspectKey) -> Option<&SnapshotReadValue> {
        self.values.get(aspect_key)
    }

    pub fn contains_key(&self, aspect_key: &AspectKey) -> bool {
        self.values.contains_key(aspect_key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&AspectKey, &SnapshotReadValue)> {
        self.values.iter()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn from_projected_values(values: BTreeMap<AspectKey, SnapshotReadValue>) -> Self {
        Self { values }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RelationalAuthoritativeRowArtifact {
    row_identity: RelationalRowIdentity,
    projected_aspect_values: RelationalProjectedAspectValueSet,
}

impl RelationalAuthoritativeRowArtifact {
    pub fn row_identity(&self) -> &RelationalRowIdentity {
        &self.row_identity
    }

    pub fn projected_aspect_values(&self) -> &RelationalProjectedAspectValueSet {
        &self.projected_aspect_values
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RelationalAuthoritativeRowSetArtifact {
    snapshot_identity: TruthSnapshotIdentity,
    rows: Vec<RelationalAuthoritativeRowArtifact>,
    digest: RelationalRowSetDigest,
}

impl RelationalAuthoritativeRowSetArtifact {
    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn rows(&self) -> &[RelationalAuthoritativeRowArtifact] {
        &self.rows
    }

    pub fn digest(&self) -> &RelationalRowSetDigest {
        &self.digest
    }
}

pub fn materialize_relational_authoritative_row_set(
    packet: &SnapshotReadPacket,
    result: &SnapshotReadPacketResult,
) -> Result<RelationalAuthoritativeRowSetArtifact, RelationalGroupedTruthError> {
    if packet.reads().len() != result.records().len() {
        return Err(RelationalGroupedTruthError::PacketResultShapeMismatch);
    }

    let mut rows: BTreeMap<RelationalRowIdentity, BTreeMap<AspectKey, SnapshotReadValue>> =
        BTreeMap::new();
    for (read, record) in packet.reads().iter().zip(result.records().iter()) {
        let Some(aspect_read) = decode_snapshot_aspect_read_value(record)? else {
            continue;
        };
        let row_identity = read.relational_record_identity_parts().ok_or_else(|| {
            RelationalGroupedTruthError::UntypedRelationalRowIdentity {
                request_key: read.correlation_id().as_str().to_string(),
            }
        })?;
        rows.entry(RelationalRowIdentity::new(row_identity))
            .or_default()
            .insert(read.aspect_key().clone(), aspect_read.value().clone());
    }

    let rows =
        rows.into_iter()
            .map(
                |(row_identity, aspect_values)| RelationalAuthoritativeRowArtifact {
                    row_identity,
                    projected_aspect_values:
                        RelationalProjectedAspectValueSet::from_projected_values(aspect_values),
                },
            )
            .collect::<Vec<_>>();

    let digest = row_set_digest(result.snapshot_identity(), &rows)?;

    Ok(RelationalAuthoritativeRowSetArtifact {
        snapshot_identity: result.snapshot_identity().clone(),
        rows,
        digest,
    })
}

fn row_identity_label(parts: RelationalBridgeRecordIdentityParts) -> String {
    let kind = match parts.kind() {
        RelationalBridgeRecordIdentityKind::Entity => "entity",
        RelationalBridgeRecordIdentityKind::Relation => "relation",
    };
    format!(
        "{kind}:{}:{}:{}",
        parts.partition_id(),
        parts.local_slot(),
        parts.generation()
    )
}

#[cfg(test)]
mod tests {
    use crate::facade::{
        RelationalBridgeRecordIdentityParts, RelationalBridgeSnapshotIdentityParts,
        SnapshotReadContract, SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord,
        SnapshotReadRequest, TruthSnapshotIdentity,
    };
    use worth_foundational::facade::{
        AspectKey, AspectValue, FieldKey, InternedString, ScalarAspectType, StructAspectValue,
        Symbol,
    };

    use super::materialize_relational_authoritative_row_set;

    #[test]
    fn relational_row_set_preserves_row_identity_and_aspect_values() {
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

        assert_eq!(row_set.rows().len(), 2);
        assert_eq!(
            row_set.rows()[0].row_identity().parts(),
            RelationalBridgeRecordIdentityParts::entity(0, 1, 1)
        );
        assert!(row_set.rows()[0]
            .projected_aspect_values()
            .contains_key(&AspectKey::new("identity.id").unwrap()));
        assert_eq!(
            row_set.rows()[0]
                .projected_aspect_values()
                .get(&AspectKey::new("identity.id").unwrap()),
            Some(&crate::facade::SnapshotReadValue::Scalar(
                AspectValue::String("task-1".into())
            ))
        );
    }

    #[test]
    fn relational_row_set_preserves_struct_snapshot_values() {
        let packet = SnapshotReadPacket::new(vec![string_read(
            RelationalBridgeRecordIdentityParts::entity(0, 1, 1),
            "identity.id",
        )]);
        let result = SnapshotReadPacketResult::new(
            test_snapshot_identity(),
            vec![read_record(
                &packet,
                0,
                StructAspectValue::new([(
                    FieldKey::new("name").expect("valid field key"),
                    AspectValue::String("bad".into()),
                )])
                .expect("valid struct aspect value"),
            )],
        );

        let row_set = materialize_relational_authoritative_row_set(&packet, &result)
            .expect("relational row materialization must preserve native structs");
        assert!(matches!(
            row_set.rows()[0]
                .projected_aspect_values()
                .get(&AspectKey::new("identity.id").unwrap()),
            Some(crate::facade::SnapshotReadValue::Struct(_))
        ));
    }

    #[test]
    fn relational_row_set_omits_authoritatively_absent_aspects_without_panicking() {
        let packet = SnapshotReadPacket::new(vec![
            string_read(
                RelationalBridgeRecordIdentityParts::entity(0, 1, 1),
                "identity.id",
            ),
            string_read(
                RelationalBridgeRecordIdentityParts::entity(0, 1, 1),
                "status.lane",
            ),
        ]);
        let result = SnapshotReadPacketResult::new(
            test_snapshot_identity(),
            vec![
                read_record(&packet, 0, AspectValue::String("task-1".into())),
                SnapshotReadRecord::absent_for_request(&packet.reads()[1]),
            ],
        );

        let row_set = materialize_relational_authoritative_row_set(&packet, &result).unwrap();
        assert_eq!(row_set.rows().len(), 1);
        assert_eq!(row_set.rows()[0].projected_aspect_values().len(), 1);
        assert!(!row_set.rows()[0]
            .projected_aspect_values()
            .contains_key(&AspectKey::new("status.lane").unwrap()));
    }

    #[test]
    fn relational_row_set_rejects_untyped_snapshot_read_identity() {
        let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_coarse(
            // allowed-untyped-negative-test
            "legacy-row",
            SnapshotReadContract::scalar(aspect_key("identity.id"), ScalarAspectType::String),
        )]);
        let result = SnapshotReadPacketResult::new(
            test_snapshot_identity(),
            vec![read_record(
                &packet,
                0,
                AspectValue::String("task-1".into()),
            )],
        );

        let error = materialize_relational_authoritative_row_set(&packet, &result)
            .expect_err("grouped truth row materialization requires typed relational row identity");

        assert!(matches!(
            error,
            super::RelationalGroupedTruthError::UntypedRelationalRowIdentity { .. }
        ));
    }

    #[test]
    fn relational_row_set_digest_preserves_interned_string_family() {
        let raw = row_set_with_identity_value(AspectValue::String(InternedString::Raw(
            "symbol:7".to_string(),
        )));
        let symbol =
            row_set_with_identity_value(AspectValue::String(InternedString::Symbol(Symbol(7))));

        assert_ne!(raw.digest(), symbol.digest());
    }

    fn row_set_with_identity_value(
        value: AspectValue,
    ) -> super::RelationalAuthoritativeRowSetArtifact {
        let packet = SnapshotReadPacket::new(vec![string_read(
            RelationalBridgeRecordIdentityParts::entity(0, 1, 1),
            "identity.id",
        )]);
        let result = SnapshotReadPacketResult::new(
            test_snapshot_identity(),
            vec![read_record(&packet, 0, value)],
        );
        materialize_relational_authoritative_row_set(&packet, &result).unwrap()
    }

    fn test_snapshot_identity() -> TruthSnapshotIdentity {
        TruthSnapshotIdentity::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts::new(
            1, 1,
        ))
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
}
