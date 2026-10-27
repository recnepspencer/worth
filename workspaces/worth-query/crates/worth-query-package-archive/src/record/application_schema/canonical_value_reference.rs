//! Opaque, complete description compared with the installed typed codec.
use super::wire_vocabulary::{decode_optional, write_optional};
use crate::denial::WorthQueryPackageArchiveDenial as Denial;
use crate::record::foundational_value;
use crate::{binary_encoding::BinaryEncodingSink, binary_input::BinaryInput};
use worth_query_declaration::facade::application_schema::{
    ApplicationCanonicalValueReference, ApplicationCanonicalValueReferenceParts,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    reference: &ApplicationCanonicalValueReference,
) -> Result<(), Denial> {
    let parts = reference.parts();
    output.text(&parts.protocol)?;
    output.text(&parts.shape)?;
    for maximum in [
        parts.maximum_wire_bytes,
        parts.maximum_string_bytes,
        parts.maximum_sequence_length,
        parts.maximum_depth,
    ] {
        write_optional(output, maximum.as_ref(), |output, value| output.u64(*value))?;
    }
    foundational_value::write_bool(output, parts.supports_admission)
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCanonicalValueReference, Denial> {
    // RecordDecodeAttempt admits this complete frame's logical bytes before
    // BinaryInput is created. Shape text remains opaque: no grammar walk occurs.
    Ok(ApplicationCanonicalValueReference::from_untrusted_parts(
        ApplicationCanonicalValueReferenceParts {
            protocol: input.text()?.to_owned(),
            shape: input.text()?.to_owned(),
            maximum_wire_bytes: decode_optional(input, |input| input.u64())?,
            maximum_string_bytes: decode_optional(input, |input| input.u64())?,
            maximum_sequence_length: decode_optional(input, |input| input.u64())?,
            maximum_depth: decode_optional(input, |input| input.u64())?,
            supports_admission: foundational_value::decode_bool(input)?,
        },
    ))
}
