use super::super::{
    canonical_value_reference,
    wire_vocabulary::{decode_optional, decode_type_identity, write_optional, write_type_identity},
};
use crate::denial::WorthQueryPackageArchiveDenial as Denial;
use crate::record::{foundational_aspect, foundational_value};
use crate::{binary_encoding::BinaryEncodingSink, binary_input::BinaryInput};
use worth_query_declaration::facade::{
    application_resource::{
        ApplicationResourceValueBindingParts, ApplicationResourceValueBindingReference,
    },
    application_schema::ApplicationNativeInputDecodeProfile,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    reference: &ApplicationResourceValueBindingReference,
) -> Result<(), Denial> {
    let parts = reference.parts();
    write_type_identity(output, &parts.identity)?;
    foundational_aspect::write_scalar_type(output, parts.scalar_family)?;
    write_optional(output, parts.unit.as_deref(), |output, value| {
        output.text(value)
    })?;
    write_optional(output, parts.frame.as_deref(), |output, value| {
        output.text(value)
    })?;
    write_optional(
        output,
        parts.canonical.as_ref(),
        canonical_value_reference::write,
    )?;
    foundational_value::write_bool(output, parts.supports_admitted_decode)?;
    write_optional(
        output,
        parts.native_input_admission.as_ref(),
        |output, profile| {
            output.u64(profile.work_base())?;
            output.u64(profile.work_per_byte())?;
            output.u64(profile.peak_memory_base())?;
            output.u64(profile.peak_memory_per_byte())
        },
    )
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationResourceValueBindingReference, Denial> {
    Ok(
        ApplicationResourceValueBindingReference::from_untrusted_parts(
            ApplicationResourceValueBindingParts {
                identity: decode_type_identity(input)?,
                scalar_family: foundational_aspect::decode_scalar_type(input)?,
                unit: decode_optional(input, |input| Ok(input.text()?.to_owned()))?,
                frame: decode_optional(input, |input| Ok(input.text()?.to_owned()))?,
                canonical: decode_optional(input, canonical_value_reference::decode)?,
                supports_admitted_decode: foundational_value::decode_bool(input)?,
                native_input_admission: decode_optional(input, |input| {
                    Ok(ApplicationNativeInputDecodeProfile::new(
                        input.u64()?,
                        input.u64()?,
                        input.u64()?,
                        input.u64()?,
                    ))
                })?,
            },
        ),
    )
}
