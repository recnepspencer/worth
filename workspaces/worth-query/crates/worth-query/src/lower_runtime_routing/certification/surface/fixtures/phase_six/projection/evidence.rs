use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::projection_consumption::ProjectionConsumptionSource;
use worth_foundational::facade::{AspectKey, AspectValue, ScalarAspectType};
use worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts;
use worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts;
use worth_runtime_bridge::facade::{
    encode_snapshot_aspect_read_value, RelationalGroupedProjectionContract,
};
use worth_runtime_bridge::facade::{
    BridgeIdentityEvidence, SnapshotReadPacket, SnapshotReadRecord, SnapshotReadRequest,
    TruthSnapshotIdentity,
};

pub(super) fn aspect_value(value: AspectValue) -> AspectValue {
    encode_snapshot_aspect_read_value(&value)
}

pub(super) fn read_record(
    packet: &SnapshotReadPacket,
    index: usize,
    value: AspectValue,
) -> SnapshotReadRecord {
    SnapshotReadRecord::for_request(&packet.reads()[index], aspect_value(value))
}

pub(super) fn string_read(
    entity_identity: RelationalBridgeRecordIdentityParts,
    aspect: &str,
) -> SnapshotReadRequest {
    SnapshotReadRequest::for_relational_record(
        entity_identity,
        worth_runtime_bridge::facade::SnapshotReadContract::scalar(
            aspect_key(aspect),
            ScalarAspectType::String,
        ),
    )
}

pub(super) fn projection_source_evidence_identity(
    source: &ProjectionConsumptionSource,
    role: &'static str,
) -> WorthQueryEvidenceIdentity {
    let mut builder =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                source.family().as_str(),
            );
    if let Some(basis) = source.basis_digest() {
        builder = builder.field_value(WorthQueryEvidenceTag::new("basis"), basis);
    }
    if let Some(identity) = source.source_identity_handle().evidence_identity() {
        builder = builder.field_evidence_identity(WorthQueryEvidenceTag::new("source"), identity);
    } else {
        builder = builder.field_value(
            WorthQueryEvidenceTag::new("source"),
            source.source_identity(),
        );
    }
    for reference in source.source_reference_identities() {
        builder = builder.field_value(
            WorthQueryEvidenceTag::new(reference.label()),
            reference.identity(),
        );
    }
    builder.seal()
}

pub(super) fn relational_grouped_projection_evidence(
    source: &ProjectionConsumptionSource,
    grouped_digest: &str,
    role: &'static str,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("source"),
            &projection_source_evidence_identity(source, "relational-grouped"),
        )
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_value(WorthQueryEvidenceTag::new("grouped"), grouped_digest)
        .seal()
}

pub(super) fn bridge_grouped_projection_evidence(
    source: &ProjectionConsumptionSource,
    grouped_identity: &BridgeIdentityEvidence,
    role: &'static str,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("source"),
            &projection_source_evidence_identity(source, "bridge-grouped"),
        )
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("grouped"),
            grouped_identity,
        )
        .seal()
}

pub(super) fn projection_snapshot_identity() -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts::new(
        6, 2,
    ))
}

pub(super) fn relational_row_identity(
    entity_identity: RelationalBridgeRecordIdentityParts,
) -> String {
    format!(
        "entity:{}:{}:{}",
        entity_identity.partition_id(),
        entity_identity.local_slot(),
        entity_identity.generation()
    )
}

pub(super) fn grouped_projection_contract(
    grouping_aspect: &str,
    identity_binding_aspect: &str,
    grouping_binding_aspect: &str,
) -> RelationalGroupedProjectionContract {
    RelationalGroupedProjectionContract::new(
        aspect_key(grouping_aspect),
        aspect_key(identity_binding_aspect),
        aspect_key(grouping_binding_aspect),
    )
}

pub(super) fn aspect_key(label: &str) -> AspectKey {
    AspectKey::new(label).expect("fixture aspect key must be foundational")
}
