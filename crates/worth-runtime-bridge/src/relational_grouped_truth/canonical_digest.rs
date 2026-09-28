use crate::relational_source::identity_parts::{
    RelationalBridgeRecordIdentityKind, RelationalBridgeRecordIdentityParts,
};
use crate::snapshot::{SnapshotReadValue, TruthSnapshotIdentity};
use sha2::{Digest, Sha256};
use worth_foundational::facade::{
    prepare_aspect_value_identity_basis, prepare_struct_aspect_value_identity_basis, AspectKey,
    AspectValue,
};

use super::grouped_projection::{
    RelationalGroupedMemberRow, RelationalGroupedProjectionContract,
    RelationalGroupedProjectionDigest, RelationalGroupedTruthError,
};
use super::row_set::{RelationalAuthoritativeRowArtifact, RelationalRowSetDigest};

pub(super) fn row_set_digest(
    snapshot_identity: &TruthSnapshotIdentity,
    rows: &[RelationalAuthoritativeRowArtifact],
) -> Result<RelationalRowSetDigest, RelationalGroupedTruthError> {
    let mut bytes = Vec::new();
    bytes.push(1);
    encode_snapshot_identity(&mut bytes, snapshot_identity)?;
    for row in rows {
        bytes.push(2);
        encode_row_identity(&mut bytes, row.row_identity().parts());
        for (aspect_key, value) in row.projected_aspect_values().iter() {
            encode_aspect_entry(&mut bytes, aspect_key, value);
        }
    }
    Ok(RelationalRowSetDigest::from_canonical_bytes(&bytes))
}

pub(super) fn grouped_projection_digest(
    row_set_digest: &RelationalRowSetDigest,
    snapshot_identity: &TruthSnapshotIdentity,
    contract: &RelationalGroupedProjectionContract,
    members: &[RelationalGroupedMemberRow],
) -> Result<RelationalGroupedProjectionDigest, RelationalGroupedTruthError> {
    let mut bytes = Vec::new();
    bytes.push(16);
    encode_string(&mut bytes, row_set_digest.as_str());
    encode_snapshot_identity(&mut bytes, snapshot_identity)?;
    encode_string(&mut bytes, contract.grouping_aspect().as_str());
    encode_string(&mut bytes, contract.identity_binding_aspect_key().as_str());
    encode_string(&mut bytes, contract.grouping_binding_aspect_key().as_str());
    for member in members {
        bytes.push(17);
        encode_row_identity(&mut bytes, member.row_identity().parts());
        encode_aspect_value(&mut bytes, member.identity_value());
        encode_aspect_value(&mut bytes, member.grouping_value());
    }
    Ok(RelationalGroupedProjectionDigest::from_canonical_bytes(
        &bytes,
    ))
}

fn encode_aspect_entry(bytes: &mut Vec<u8>, aspect_key: &AspectKey, value: &SnapshotReadValue) {
    bytes.push(3);
    encode_string(bytes, aspect_key.as_str());
    encode_snapshot_read_value(bytes, value)
}

fn encode_snapshot_identity(
    bytes: &mut Vec<u8>,
    snapshot_identity: &TruthSnapshotIdentity,
) -> Result<(), RelationalGroupedTruthError> {
    let snapshot = snapshot_identity
        .relational_snapshot_parts()
        .ok_or(RelationalGroupedTruthError::UntypedRelationalSnapshotIdentity)?;
    bytes.push(4);
    encode_u64(bytes, snapshot.snapshot_id());
    encode_u64(bytes, snapshot.version_id());
    Ok(())
}

fn encode_row_identity(bytes: &mut Vec<u8>, parts: RelationalBridgeRecordIdentityParts) {
    bytes.push(5);
    let kind = match parts.kind() {
        RelationalBridgeRecordIdentityKind::Entity => 1,
        RelationalBridgeRecordIdentityKind::Relation => 2,
    };
    bytes.push(kind);
    encode_u32(bytes, parts.partition_id());
    encode_u64(bytes, parts.local_slot());
    encode_u32(bytes, parts.generation());
}

fn encode_aspect_value(bytes: &mut Vec<u8>, value: &AspectValue) {
    encode_string(bytes, prepare_aspect_value_identity_basis(value).as_str());
}

fn encode_snapshot_read_value(bytes: &mut Vec<u8>, value: &SnapshotReadValue) {
    match value {
        SnapshotReadValue::Scalar(value) => {
            bytes.push(18);
            encode_aspect_value(bytes, value);
        }
        SnapshotReadValue::Struct(value) => {
            bytes.push(19);
            encode_string(
                bytes,
                prepare_struct_aspect_value_identity_basis(value).as_str(),
            );
        }
    }
}

fn encode_string(bytes: &mut Vec<u8>, value: &str) {
    encode_length_prefixed_bytes(bytes, value.as_bytes());
}

fn encode_length_prefixed_bytes(bytes: &mut Vec<u8>, value: &[u8]) {
    encode_u32(bytes, value.len() as u32);
    bytes.extend_from_slice(value);
}

fn encode_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn encode_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub(super) fn digest_with_prefix(prefix: &str, bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{prefix}:sha256:{digest:x}")
}
