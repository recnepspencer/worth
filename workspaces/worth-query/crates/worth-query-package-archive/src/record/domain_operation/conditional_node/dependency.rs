use worth_foundational::facade::{
    AspectBinding, AuthoritativeAspectChangeKind as Change, FieldKey, TruthPartitionRole,
};
use worth_query_installation::facade::{
    WorthQueryConditionalGraphReadRole, WorthQuerySemanticLocality,
    WorthQuerySemanticTruthDependency,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::decode_budget::RecordDecodeAttempt;
use crate::record::foundational_aspect::{
    decode_aspect_contract, decode_projection_mask, write_aspect_contract, write_projection_mask,
};
use crate::record::sequence::{decode_sequence, require_canonical_sequence, write_sequence};

pub(super) fn write_dependency(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQuerySemanticTruthDependency,
) -> Result<(), Denial> {
    output.text(value.graph_read_role().as_str())?;
    write_aspect_contract(output, value.contract())?;
    write_projection_mask(output, value.projection_mask())?;
    write_binding(output, value.binding())?;
    write_locality(output, value.locality())?;
    write_sequence(output, value.relevant_changes(), |output, change| {
        output.u16(change_tag(*change))
    })
}

pub(super) fn decode_dependency(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQuerySemanticTruthDependency, Denial> {
    let role =
        WorthQueryConditionalGraphReadRole::new(input.text()?.to_owned()).map_err(|_| invalid())?;
    let contract = decode_aspect_contract(input, budget)?;
    let mask = decode_projection_mask(input, budget)?;
    let binding = decode_binding(input)?;
    let locality = decode_locality(input)?;
    let changes = decode_sequence(input, budget, 2, |input, _| change(input.u16()?))?;
    require_canonical_sequence(&changes)?;
    WorthQuerySemanticTruthDependency::new(role, contract, mask, binding, locality, changes)
        .map_err(|_| invalid())
}

pub(super) fn write_locality(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQuerySemanticLocality,
) -> Result<(), Denial> {
    match value {
        WorthQuerySemanticLocality::SourceRecord => output.u16(1),
        WorthQuerySemanticLocality::SourcePartition(role) => {
            output.u16(2)?;
            output.text(role.as_str())
        }
        WorthQuerySemanticLocality::WholeLogicalGraph => output.u16(3),
    }
}

pub(super) fn decode_locality(
    input: &mut BinaryInput<'_>,
) -> Result<WorthQuerySemanticLocality, Denial> {
    match input.u16()? {
        1 => Ok(WorthQuerySemanticLocality::SourceRecord),
        2 => Ok(WorthQuerySemanticLocality::SourcePartition(
            TruthPartitionRole::new(input.text()?.to_owned()).map_err(|_| invalid())?,
        )),
        3 => Ok(WorthQuerySemanticLocality::WholeLogicalGraph),
        _ => unsupported(),
    }
}

fn write_binding(output: &mut dyn BinaryEncodingSink, value: &AspectBinding) -> Result<(), Denial> {
    match value {
        AspectBinding::EntityField { field } => {
            output.u16(1)?;
            output.text(field.as_str())
        }
        AspectBinding::RelationField { field } => {
            output.u16(2)?;
            output.text(field.as_str())
        }
        AspectBinding::RelationSourceEndpoint => output.u16(3),
        AspectBinding::RelationTargetEndpoint => output.u16(4),
        AspectBinding::StructuralRegion => output.u16(5),
        AspectBinding::StructuralPartition => output.u16(6),
        AspectBinding::StructuralFacet => output.u16(7),
        AspectBinding::LifecycleTransition => output.u16(8),
        _ => unsupported(),
    }
}

fn decode_binding(input: &mut BinaryInput<'_>) -> Result<AspectBinding, Denial> {
    match input.u16()? {
        1 => Ok(AspectBinding::EntityField {
            field: decode_field(input)?,
        }),
        2 => Ok(AspectBinding::RelationField {
            field: decode_field(input)?,
        }),
        3 => Ok(AspectBinding::RelationSourceEndpoint),
        4 => Ok(AspectBinding::RelationTargetEndpoint),
        5 => Ok(AspectBinding::StructuralRegion),
        6 => Ok(AspectBinding::StructuralPartition),
        7 => Ok(AspectBinding::StructuralFacet),
        8 => Ok(AspectBinding::LifecycleTransition),
        _ => unsupported(),
    }
}

fn decode_field(input: &mut BinaryInput<'_>) -> Result<FieldKey, Denial> {
    FieldKey::new(input.text()?.to_owned()).ok_or_else(invalid)
}

fn change_tag(value: Change) -> u16 {
    match value {
        Change::WholeAspectSet => 1,
        Change::WholeAspectClear => 2,
        Change::FieldSet => 3,
        Change::FieldClear => 4,
        Change::RelationSourceEndpoint => 5,
        Change::RelationTargetEndpoint => 6,
        Change::StructuralCreate => 7,
        Change::StructuralUpdate => 8,
        Change::StructuralDelete => 9,
        Change::StructuralRetainForAudit => 10,
        Change::LifecycleCreate => 11,
        Change::LifecycleDelete => 12,
        Change::LifecycleRetainForAudit => 13,
        Change::Opaque => 14,
        Change::StructuralMaterializationSuspended => 15,
        Change::StructuralRematerialized => 16,
    }
}

fn change(tag: u16) -> Result<Change, Denial> {
    Ok(match tag {
        1 => Change::WholeAspectSet,
        2 => Change::WholeAspectClear,
        3 => Change::FieldSet,
        4 => Change::FieldClear,
        5 => Change::RelationSourceEndpoint,
        6 => Change::RelationTargetEndpoint,
        7 => Change::StructuralCreate,
        8 => Change::StructuralUpdate,
        9 => Change::StructuralDelete,
        10 => Change::StructuralRetainForAudit,
        11 => Change::LifecycleCreate,
        12 => Change::LifecycleDelete,
        13 => Change::LifecycleRetainForAudit,
        14 => Change::Opaque,
        15 => Change::StructuralMaterializationSuspended,
        16 => Change::StructuralRematerialized,
        _ => return unsupported(),
    })
}

fn unsupported<T>() -> Result<T, Denial> {
    Err(Denial::new(Kind::UnsupportedRecordVariant))
}
fn invalid() -> Denial {
    Denial::new(Kind::InvalidRecordShape)
}
