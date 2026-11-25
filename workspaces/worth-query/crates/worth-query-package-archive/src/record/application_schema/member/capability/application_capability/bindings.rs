use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityContextEntitySlotBinding, ApplicationCapabilityFieldBinding,
    ApplicationCapabilityOperationBinding, ApplicationCapabilityPathContextAnchor,
    ApplicationCapabilityRelationBinding, ApplicationCapabilityValueBinding,
    WorthQueryPortableApplicationCapabilityContextEntitySlotBindingParts,
    WorthQueryPortableApplicationCapabilityFieldBindingParts,
    WorthQueryPortableApplicationCapabilityOperationBindingParts,
    WorthQueryPortableApplicationCapabilityPathContextAnchorParts,
    WorthQueryPortableApplicationCapabilityRelationBindingParts,
    WorthQueryPortableApplicationCapabilityValueBindingParts,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::WorthQueryPackageArchiveDenial as Denial;

use super::super::super::super::super::foundational_aspect;
use super::super::super::super::super::foundational_value;
use super::super::super::super::wire_vocabulary::{decode_type_identity, write_type_identity};

pub(super) fn write_field(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityFieldBinding,
) -> Result<(), Denial> {
    output.text(value.entity())?;
    output.text(value.aspect())?;
    output.text(value.field())?;
    foundational_aspect::write_scalar_type(output, value.scalar_family())?;
    output.text(value.value_type())
}
pub(super) fn decode_field(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCapabilityFieldBinding, Denial> {
    Ok(ApplicationCapabilityFieldBinding::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationCapabilityFieldBindingParts {
            entity: input.text()?.to_owned(),
            aspect: input.text()?.to_owned(),
            field: input.text()?.to_owned(),
            scalar_family: foundational_aspect::decode_scalar_type(input)?,
            value_type: input.text()?.to_owned(),
        },
    ))
}

pub(super) fn write_value(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityValueBinding,
) -> Result<(), Denial> {
    write_field(output, value.field())?;
    foundational_value::write_aspect_value(output, value.value())
}
pub(super) fn decode_value(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCapabilityValueBinding, Denial> {
    Ok(ApplicationCapabilityValueBinding::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationCapabilityValueBindingParts {
            field: decode_field(input)?,
            value: foundational_value::decode_aspect_value(input)?,
        },
    ))
}

pub(super) fn write_relation(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityRelationBinding,
) -> Result<(), Denial> {
    output.text(value.relation())?;
    output.text(value.from())?;
    output.text(value.to())
}
pub(super) fn decode_relation(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCapabilityRelationBinding, Denial> {
    Ok(ApplicationCapabilityRelationBinding::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationCapabilityRelationBindingParts {
            relation: input.text()?.to_owned(),
            from: input.text()?.to_owned(),
            to: input.text()?.to_owned(),
        },
    ))
}

pub(super) fn write_operation(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityOperationBinding,
) -> Result<(), Denial> {
    output.text(value.operation())?;
    write_type_identity(output, &value.operation_identity())?;
    write_type_identity(output, &value.input_identity())
}
pub(super) fn decode_operation(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCapabilityOperationBinding, Denial> {
    Ok(ApplicationCapabilityOperationBinding::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationCapabilityOperationBindingParts {
            operation: input.text()?.to_owned(),
            operation_identity: decode_type_identity(input)?,
            input_identity: decode_type_identity(input)?,
        },
    ))
}

pub(super) fn write_context_slot(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityContextEntitySlotBinding,
) -> Result<(), Denial> {
    output.text(value.context())?;
    write_type_identity(output, value.context_identity_ref())?;
    output.text(value.slot())?;
    write_type_identity(output, value.slot_identity_ref())?;
    output.text(value.entity())
}
pub(super) fn decode_context_slot(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCapabilityContextEntitySlotBinding, Denial> {
    Ok(
        ApplicationCapabilityContextEntitySlotBinding::from_untrusted_parts(
            ARCHIVE_DECODE,
            WorthQueryPortableApplicationCapabilityContextEntitySlotBindingParts {
                context: input.text()?.to_owned(),
                context_identity: decode_type_identity(input)?,
                slot: input.text()?.to_owned(),
                slot_identity: decode_type_identity(input)?,
                entity: input.text()?.to_owned(),
            },
        ),
    )
}

pub(super) fn write_anchor(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityPathContextAnchor,
) -> Result<(), Denial> {
    write_relation(output, value.relation())?;
    super::super::super::super::authorization_path::write_direction(output, value.direction())?;
    write_context_slot(output, value.slot())
}
pub(super) fn decode_anchor(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCapabilityPathContextAnchor, Denial> {
    Ok(
        ApplicationCapabilityPathContextAnchor::from_untrusted_parts(
            ARCHIVE_DECODE,
            WorthQueryPortableApplicationCapabilityPathContextAnchorParts {
                relation: decode_relation(input)?,
                direction: super::super::super::super::authorization_path::decode_direction(input)?,
                slot: decode_context_slot(input)?,
            },
        ),
    )
}
