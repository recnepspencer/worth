use crate::aspect_wire::{encode_aspect_field_locator, encode_length_prefixed_aspect_value};
use crate::diagnostics::data::DiagnosticCode;
use crate::schema::data::{EndpointDeletionIntegrityMode, SymmetryMode, UniquenessScope};
use crate::validation::data::InvariantWitnessKey;

use super::{InvariantViolationFields, RelationCardinalityBoundary, RelationEndpointBoundary};

pub(super) fn invariant_violation_witness_key(
    code: DiagnosticCode,
    fields: &InvariantViolationFields,
) -> InvariantWitnessKey {
    invariant_violation_witness_key_checked(code, fields, &mut |_| true)
        .expect("unleased witness construction has no admission refusal")
}

pub(super) fn invariant_violation_witness_key_checked(
    code: DiagnosticCode,
    fields: &InvariantViolationFields,
    admit: &mut impl FnMut(u64) -> bool,
) -> Option<InvariantWitnessKey> {
    macro_rules! admitted_format {
        ($($argument:tt)*) => {{
            let mut count = WitnessTextLength(0);
            std::fmt::write(&mut count, format_args!($($argument)*)).ok()?;
            if !admit(count.0 as u64) { return None; }
            let mut text = String::with_capacity(count.0);
            std::fmt::write(&mut text, format_args!($($argument)*)).ok()?;
            text
        }};
    }
    let key = match fields {
        InvariantViolationFields::None => admitted_format!("none:{code:?}"),
        InvariantViolationFields::MergedIntentLimit {
            merged_intent_count,
            limit,
        } => admitted_format!("merged_intents:{merged_intent_count}:{limit}"),
        InvariantViolationFields::SnapshotEntityLimit {
            version_id, limit, ..
        } => {
            admitted_format!("snapshot_entity_limit:{}:{limit}", version_id.as_u64())
        }
        InvariantViolationFields::UniqueEntityField {
            field_locator,
            value,
        } => return unique_entity_aspect_field_witness_key(code, field_locator, value, admit),
        InvariantViolationFields::SidecarConsistency {
            partition_id,
            slot,
            missing_label,
        } => admitted_format!(
            "sidecar_consistency:{}:{slot}:{missing_label}",
            partition_id.as_u32()
        ),
        InvariantViolationFields::RelationEndpointKindMismatch {
            relation_kind_id,
            source,
            target,
            boundary,
            ..
        } => admitted_format!(
            "endpoint_kind_mismatch:{}:{source:?}:{target:?}:{}",
            relation_kind_id.as_u32(),
            relation_endpoint_boundary_label(*boundary)
        ),
        InvariantViolationFields::RelationEndpointKindSelfEdge {
            relation_kind_id,
            source,
            target,
            ..
        } => admitted_format!(
            "endpoint_kind_self_edge:{}:{source:?}:{target:?}",
            relation_kind_id.as_u32()
        ),
        InvariantViolationFields::RelationEndpointKindCrossContext {
            relation_kind_id,
            source_partition_id,
            target_partition_id,
            ..
        } => admitted_format!(
            "endpoint_kind_cross_context:{}:{}:{}",
            relation_kind_id.as_u32(),
            source_partition_id.as_u32(),
            target_partition_id.as_u32()
        ),
        InvariantViolationFields::RelationCardinalityEndpoint {
            relation_kind_id,
            entity_id,
            boundary,
            ..
        } => admitted_format!(
            "cardinality_endpoint:{}:{entity_id:?}:{}",
            relation_kind_id.as_u32(),
            relation_cardinality_boundary_label(*boundary)
        ),
        InvariantViolationFields::RelationCardinalityPair {
            relation_kind_id,
            source,
            target,
            ..
        } => admitted_format!(
            "cardinality_pair:{}:{source:?}:{target:?}",
            relation_kind_id.as_u32()
        ),
        InvariantViolationFields::RelationUniqueness {
            relation_kind_id,
            scope,
            source,
            target,
            ..
        } => admitted_format!(
            "uniqueness:{}:{source:?}:{target:?}:{}",
            relation_kind_id.as_u32(),
            uniqueness_scope_label(*scope)
        ),
        InvariantViolationFields::RelationSymmetry {
            relation_kind_id,
            source,
            target,
            mode,
            ..
        } => admitted_format!(
            "symmetry:{}:{source:?}:{target:?}:{}",
            relation_kind_id.as_u32(),
            symmetry_mode_label(*mode)
        ),
        InvariantViolationFields::RelationEndpointDeletionIntegrity {
            relation_kind_id,
            entity_id,
            mode,
            ..
        } => admitted_format!(
            "endpoint_deletion:{}:{entity_id:?}:{}",
            relation_kind_id.as_u32(),
            endpoint_deletion_integrity_mode_label(*mode)
        ),
        InvariantViolationFields::StorageInconsistency {
            entity_id,
            partition_id,
            slot,
            field,
            scan,
            lookup,
            failure,
            ..
        } => {
            admitted_format!(
                "storage_inconsistency:{entity_id:?}:{partition_id:?}:{slot:?}:{}:{}:{}:{}",
                field.as_ref().map(|field| field.as_str()).unwrap_or("none"),
                scan.map(|value| value.diagnostic_label()).unwrap_or("none"),
                lookup
                    .map(|value| value.diagnostic_label())
                    .unwrap_or("none"),
                failure
                    .map(|value| value.diagnostic_label())
                    .unwrap_or("none")
            )
        }
        InvariantViolationFields::CustomInvariantFailure {
            identity,
            phase,
            failure,
            ..
        } => admitted_format!(
            "custom_failure:{}:{}.{}:{}:{}",
            identity.semantic_identity().rule_id.as_str(),
            identity.semantic_identity().semantic_version.major,
            identity.semantic_identity().semantic_version.minor,
            phase.diagnostic_label(),
            failure.diagnostic_label()
        ),
        InvariantViolationFields::CustomInvariantViolation { identity } => admitted_format!(
            "custom_violation:{}:{}.{}",
            identity.rule_id.as_str(),
            identity.semantic_version.major,
            identity.semantic_version.minor,
        ),
        InvariantViolationFields::PartitionIsolation {
            relation_kind_id,
            relation_id,
            source_partition_id,
            target_partition_id,
            ..
        } => admitted_format!(
            "partition_isolation:{}:{relation_id:?}:{}:{}",
            relation_kind_id.as_u32(),
            source_partition_id.as_u32(),
            target_partition_id.as_u32()
        ),
        InvariantViolationFields::Acyclicity {
            relation_kind_id,
            source,
            target,
            ..
        } => admitted_format!(
            "acyclicity:{}:{source:?}:{target:?}",
            relation_kind_id.as_u32()
        ),
        InvariantViolationFields::ConnectivityMinimum {
            relation_kind_id,
            source,
            ..
        } => admitted_format!(
            "connectivity_minimum:{}:{source:?}",
            relation_kind_id.as_u32()
        ),
    };
    Some(InvariantWitnessKey::new(key))
}

fn unique_entity_aspect_field_witness_key(
    code: DiagnosticCode,
    field_locator: &worth_foundational::facade::AspectFieldLocator,
    value: &worth_foundational::facade::AspectValue,
    admit: &mut impl FnMut(u64) -> bool,
) -> Option<InvariantWitnessKey> {
    let locator_len = 2usize
        .checked_add(4)?
        .checked_add(field_locator.aspect().aspect_key().as_str().len())?
        .checked_add(4)?;
    let locator_len = field_locator
        .field_path()
        .fields()
        .iter()
        .try_fold(locator_len, |bytes, field| {
            bytes.checked_add(4)?.checked_add(field.as_str().len())
        })?;
    let value_len = crate::aspect_wire::encoded_aspect_value_len(value)?.checked_add(4)?;
    // Each encoded Vec can grow to twice its length. The value encoder also
    // holds its body buffer; hex strings hold exactly two bytes per input byte.
    let encoding = locator_len
        .checked_mul(2)?
        .checked_add(value_len.checked_mul(4)?)?
        .checked_add((field_locator.owned_allocation_capacity_bytes() as usize).checked_mul(2)?)?
        .checked_add(value.owned_allocation_capacity_bytes())?;
    let hex = locator_len.checked_add(value_len)?.checked_mul(2)?;
    if !admit(encoding.checked_add(hex)? as u64) {
        return None;
    }
    let field_locator_canonical_bytes = encode_aspect_field_locator(field_locator);
    let value_canonical_bytes = canonical_aspect_value_witness_basis(value);
    let locator_hex = hex_bytes(&field_locator_canonical_bytes);
    let value_hex = hex_bytes(&value_canonical_bytes);
    let mut count = WitnessTextLength(0);
    std::fmt::write(
        &mut count,
        format_args!("unique_entity_aspect_field:{code:?}:{locator_hex}:{value_hex}"),
    )
    .ok()?;
    if !admit(count.0 as u64) {
        return None;
    }
    let mut key = String::with_capacity(count.0);
    std::fmt::write(
        &mut key,
        format_args!("unique_entity_aspect_field:{code:?}:{locator_hex}:{value_hex}"),
    )
    .ok()?;
    Some(InvariantWitnessKey::unique_entity_aspect_field(
        key,
        field_locator.clone(),
        value.clone(),
        field_locator_canonical_bytes,
        value_canonical_bytes,
    ))
}

fn canonical_aspect_value_witness_basis(
    value: &worth_foundational::facade::AspectValue,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    encode_length_prefixed_aspect_value(&mut bytes, value);
    bytes
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

fn relation_endpoint_boundary_label(boundary: RelationEndpointBoundary) -> &'static str {
    match boundary {
        RelationEndpointBoundary::Source => "source",
        RelationEndpointBoundary::Target => "target",
    }
}

fn relation_cardinality_boundary_label(boundary: RelationCardinalityBoundary) -> &'static str {
    match boundary {
        RelationCardinalityBoundary::Source => "source",
        RelationCardinalityBoundary::Target => "target",
        RelationCardinalityBoundary::Pair => "pair",
    }
}

fn uniqueness_scope_label(scope: UniquenessScope) -> &'static str {
    match scope {
        UniquenessScope::DirectedSemanticEdge => "directed",
        UniquenessScope::NormalizedSymmetricEdge => "normalized",
    }
}

fn symmetry_mode_label(mode: SymmetryMode) -> &'static str {
    match mode {
        SymmetryMode::CanonicalUndirected => "canonical_undirected",
        SymmetryMode::PairedInverseRequired => "paired_inverse_required",
        SymmetryMode::PairedTwinRequired => "paired_twin_required",
        SymmetryMode::InverseProhibited => "inverse_prohibited",
    }
}

fn endpoint_deletion_integrity_mode_label(mode: EndpointDeletionIntegrityMode) -> &'static str {
    match mode {
        EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations => {
            "reject_delete_with_live_relations"
        }
        EndpointDeletionIntegrityMode::RequireRelationDeletionInSameCommit => {
            "require_relation_deletion_in_same_commit"
        }
        EndpointDeletionIntegrityMode::RequireRelationRetirement => "require_relation_retirement",
    }
}

struct WitnessTextLength(usize);
impl std::fmt::Write for WitnessTextLength {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        self.0 = self.0.checked_add(text.len()).ok_or(std::fmt::Error)?;
        Ok(())
    }
}
