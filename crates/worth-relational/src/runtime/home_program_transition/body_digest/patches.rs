//! Borrowed aspect patch inputs, using the canonical Native leaf encoders.
use super::{BodyOutput, RelationalHomeProgramBodyDenial as Denial};
use crate::aspect_wire::{write_aspect_field_locator, write_aspect_value, CanonicalAspectOutput};
use crate::transactions::data::AspectFieldPatch;
use worth_foundational::facade::{
    ContractValidationInput, PortableAspectPatchOperation, PortableRecordAspectPatch,
};

pub(super) fn fields(
    out: &mut BodyOutput<'_, '_, '_>,
    patch: &AspectFieldPatch,
) -> Result<(), Denial> {
    out.length(patch.len())?;
    for (locator, value) in patch.iter() {
        write_aspect_field_locator(out, locator)?;
        write_aspect_value(out, value)?;
    }
    Ok(())
}

pub(super) fn portable(
    out: &mut BodyOutput<'_, '_, '_>,
    patch: &PortableRecordAspectPatch,
) -> Result<(), Denial> {
    out.length(patch.operations().len())?;
    for operation in patch.operations() {
        match operation {
            PortableAspectPatchOperation::SetWhole { basis, value } => {
                out.byte(0)?;
                out.scalar(basis)?;
                match value {
                    ContractValidationInput::Scalar(value) => {
                        out.byte(0)?;
                        write_aspect_value(out, value)?;
                    }
                    ContractValidationInput::Struct(value) => {
                        out.byte(1)?;
                        out.length(value.fields().count())?;
                        for (field, value) in value.fields() {
                            out.text(field.as_str())?;
                            write_aspect_value(out, value)?;
                        }
                    }
                }
            }
            PortableAspectPatchOperation::ClearWhole { basis } => {
                out.byte(1)?;
                out.scalar(basis)?;
            }
            PortableAspectPatchOperation::PatchFields {
                basis,
                selected_fields,
                field_sets,
                field_clears,
            } => {
                out.byte(2)?;
                out.scalar(basis)?;
                out.length(selected_fields.len())?;
                for field in selected_fields {
                    out.text(field.as_str())?;
                }
                out.length(field_sets.len())?;
                for field in field_sets {
                    out.text(field.field().as_str())?;
                    write_aspect_value(out, field.value())?;
                }
                out.length(field_clears.len())?;
                for field in field_clears {
                    out.text(field.as_str())?;
                }
            }
        }
    }
    Ok(())
}
