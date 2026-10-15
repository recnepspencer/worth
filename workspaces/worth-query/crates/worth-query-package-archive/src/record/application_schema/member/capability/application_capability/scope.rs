use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityCardinalityDimension, ApplicationCapabilityConstraintDefinition,
    ApplicationCapabilityCurrentnessDefinition,
    ApplicationCapabilityDelegationActivationDefinition, ApplicationCapabilityDelegationDefinition,
    ApplicationCapabilityFieldDimension, ApplicationCapabilityRelationDimension,
    ApplicationCapabilityRevocationDefinition, ApplicationCapabilityTargetDefinition,
    ApplicationCapabilityValidityDefinition, ApplicationCapabilityValidityTimeline,
    ApplicationCapabilityWorkflowDefinition,
    WorthQueryPortableApplicationCapabilityConstraintParts,
    WorthQueryPortableApplicationCapabilityDelegationActivationParts,
    WorthQueryPortableApplicationCapabilityDelegationParts,
    WorthQueryPortableApplicationCapabilityRevocationParts,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::super::super::super::super::decode_budget::RecordDecodeAttempt;
use super::super::super::super::super::sequence::{decode_sequence, write_sequence};
use super::super::super::super::wire_vocabulary::{decode_type_identity, write_type_identity};
use super::bindings;

pub(super) fn write_target(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityTargetDefinition,
) -> Result<(), Denial> {
    bindings::write_value(output, value.action())?;
    bindings::write_relation(output, value.resource())?;
    if !value.resource_path().is_direct() {
        super::resource_path::write(output, value.resource_path())?;
    }
    write_relation_dimension(output, value.relation())?;
    write_field_dimension(output, value.field())?;
    bindings::write_value(output, value.purpose())
}
pub(super) fn decode_target(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    scoped_resource: bool,
) -> Result<ApplicationCapabilityTargetDefinition, Denial> {
    let action = bindings::decode_value(input)?;
    let head = bindings::decode_relation(input)?;
    let resource = if scoped_resource {
        super::resource_path::decode(input, budget, head)?
    } else {
        head.into()
    };
    Ok(ApplicationCapabilityTargetDefinition::new(
        action,
        resource,
        decode_relation_dimension(input)?,
        decode_field_dimension(input)?,
        bindings::decode_value(input)?,
    ))
}

pub(super) fn write_constraints(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityConstraintDefinition,
) -> Result<(), Denial> {
    write_field_dimension(output, value.magnitude())?;
    write_cardinality(output, value.cardinality())?;
    write_currentness(output, value.currentness())?;
    output.text(value.context())?;
    write_type_identity(output, &value.context_identity())
}
pub(super) fn decode_constraints(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCapabilityConstraintDefinition, Denial> {
    Ok(
        ApplicationCapabilityConstraintDefinition::from_untrusted_parts(
            WorthQueryPortableApplicationCapabilityConstraintParts {
                magnitude: decode_field_dimension(input)?,
                cardinality: decode_cardinality(input)?,
                currentness: decode_currentness(input)?,
                context: input.text()?.to_owned(),
                context_type: decode_type_identity(input)?,
            },
        ),
    )
}

pub(super) fn write_delegation(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityDelegationDefinition,
) -> Result<(), Denial> {
    bindings::write_relation(output, value.parent())?;
    bindings::write_relation(output, value.grantor())?;
    bindings::write_relation(output, value.grantee())?;
    bindings::write_field(output, value.limit())?;
    output.text(value.provenance())?;
    write_type_identity(output, &value.provenance_identity())?;
    match value.activation() {
        None => output.u16(0)?,
        Some(value) => {
            output.u16(1)?;
            write_activation(output, value)?;
        }
    }
    match value.revocation() {
        None => output.u16(0),
        Some(value) => {
            output.u16(1)?;
            write_revocation(output, value)
        }
    }
}
pub(super) fn decode_delegation(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationCapabilityDelegationDefinition, Denial> {
    let parent = bindings::decode_relation(input)?;
    let grantor = bindings::decode_relation(input)?;
    let grantee = bindings::decode_relation(input)?;
    let limit = bindings::decode_field(input)?;
    let provenance = input.text()?.to_owned();
    let provenance_type = decode_type_identity(input)?;
    let activation = match input.u16()? {
        0 => None,
        1 => Some(decode_activation(input, budget)?),
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let revocation = match input.u16()? {
        0 => None,
        1 => Some(decode_revocation(input)?),
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(
        ApplicationCapabilityDelegationDefinition::from_untrusted_parts(
            WorthQueryPortableApplicationCapabilityDelegationParts {
                parent,
                grantor,
                grantee,
                limit,
                provenance,
                provenance_type,
                activation,
                revocation,
            },
        ),
    )
}

fn write_activation(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityDelegationActivationDefinition,
) -> Result<(), Denial> {
    bindings::write_operation(output, value.operation())?;
    bindings::write_field(output, value.identity())?;
    write_sequence(output, value.context_relations(), bindings::write_relation)
}
fn decode_activation(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationCapabilityDelegationActivationDefinition, Denial> {
    Ok(
        ApplicationCapabilityDelegationActivationDefinition::from_untrusted_parts(
            WorthQueryPortableApplicationCapabilityDelegationActivationParts {
                operation: bindings::decode_operation(input)?,
                identity: bindings::decode_field(input)?,
                context_relations: decode_sequence(input, budget, 12, |input, _| {
                    bindings::decode_relation(input)
                })?,
            },
        ),
    )
}
fn write_revocation(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityRevocationDefinition,
) -> Result<(), Denial> {
    bindings::write_operation(output, value.operation())?;
    bindings::write_field(output, value.identity())?;
    bindings::write_value(output, value.revoked_status())
}
fn decode_revocation(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCapabilityRevocationDefinition, Denial> {
    Ok(
        ApplicationCapabilityRevocationDefinition::from_untrusted_parts(
            WorthQueryPortableApplicationCapabilityRevocationParts {
                operation: bindings::decode_operation(input)?,
                identity: bindings::decode_field(input)?,
                revoked_status: bindings::decode_value(input)?,
            },
        ),
    )
}

fn write_currentness(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityCurrentnessDefinition,
) -> Result<(), Denial> {
    bindings::write_value(output, value.active_status())?;
    bindings::write_field(output, value.workflow().grant())?;
    bindings::write_field(output, value.workflow().resource())?;
    write_validity(output, value.validity())
}
fn decode_currentness(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCapabilityCurrentnessDefinition, Denial> {
    Ok(ApplicationCapabilityCurrentnessDefinition::new(
        bindings::decode_value(input)?,
        ApplicationCapabilityWorkflowDefinition::new(
            bindings::decode_field(input)?,
            bindings::decode_field(input)?,
        ),
        decode_validity(input)?,
    ))
}

pub(super) fn write_validity(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityValidityDefinition,
) -> Result<(), Denial> {
    output.u16(match value.timeline() {
        ApplicationCapabilityValidityTimeline::UnixEpochSeconds => 1,
        ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds => 2,
    })?;
    bindings::write_field(output, value.not_before())?;
    bindings::write_field(output, value.not_after())
}
pub(super) fn decode_validity(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCapabilityValidityDefinition, Denial> {
    let timeline = match input.u16()? {
        1 => ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
        2 => ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(ApplicationCapabilityValidityDefinition::new(
        timeline,
        bindings::decode_field(input)?,
        bindings::decode_field(input)?,
    ))
}

fn write_field_dimension(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityFieldDimension,
) -> Result<(), Denial> {
    match value {
        ApplicationCapabilityFieldDimension::NotApplicable => output.u16(0),
        ApplicationCapabilityFieldDimension::Bound(value) => {
            output.u16(1)?;
            bindings::write_field(output, value)
        }
    }
}
fn decode_field_dimension(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCapabilityFieldDimension, Denial> {
    match input.u16()? {
        0 => Ok(ApplicationCapabilityFieldDimension::NotApplicable),
        1 => Ok(ApplicationCapabilityFieldDimension::Bound(
            bindings::decode_field(input)?,
        )),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
fn write_relation_dimension(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityRelationDimension,
) -> Result<(), Denial> {
    match value {
        ApplicationCapabilityRelationDimension::NotApplicable => output.u16(0),
        ApplicationCapabilityRelationDimension::Bound(value) => {
            output.u16(1)?;
            bindings::write_relation(output, value)
        }
    }
}
fn decode_relation_dimension(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCapabilityRelationDimension, Denial> {
    match input.u16()? {
        0 => Ok(ApplicationCapabilityRelationDimension::NotApplicable),
        1 => Ok(ApplicationCapabilityRelationDimension::Bound(
            bindings::decode_relation(input)?,
        )),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
fn write_cardinality(
    output: &mut dyn BinaryEncodingSink,
    value: ApplicationCapabilityCardinalityDimension,
) -> Result<(), Denial> {
    match value {
        ApplicationCapabilityCardinalityDimension::One => output.u16(1),
        ApplicationCapabilityCardinalityDimension::Many => output.u16(2),
        ApplicationCapabilityCardinalityDimension::Bounded(maximum) => {
            output.u16(3)?;
            output.u32(maximum)
        }
    }
}
fn decode_cardinality(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCapabilityCardinalityDimension, Denial> {
    match input.u16()? {
        1 => Ok(ApplicationCapabilityCardinalityDimension::One),
        2 => Ok(ApplicationCapabilityCardinalityDimension::Many),
        3 => Ok(ApplicationCapabilityCardinalityDimension::Bounded(
            input.u32()?,
        )),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
