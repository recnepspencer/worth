use worth_query_declaration::facade::application_aftermath::{
    DeclaredInverseBinding, DeclaredInverseBindingParts,
};

use super::super::super::mutation_description;
mod disclosure;
use crate::denial::WorthQueryPackageArchiveDenial as Denial;
use crate::record::application_schema::wire_vocabulary::{
    decode_optional, decode_type_identity, write_optional, write_type_identity,
};
use crate::record::decode_budget::RecordDecodeAttempt;
use crate::{binary_encoding::BinaryEncodingSink, binary_input::BinaryInput};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    binding: &DeclaredInverseBinding,
) -> Result<(), Denial> {
    write_type_identity(output, binding.identity())?;
    mutation_description::write(output, binding.source())?;
    mutation_description::write(output, binding.inverse())?;
    output.text(binding.source_handler())?;
    output.text(binding.inverse_handler())?;
    write_optional(output, binding.source_query(), |output, query| {
        output.text(query)
    })?;
    write_optional(output, binding.inverse_query(), |output, query| {
        output.text(query)
    })?;
    disclosure::write(output, binding.disclosure())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<DeclaredInverseBinding, Denial> {
    Ok(DeclaredInverseBinding::from_untrusted_parts(
        DeclaredInverseBindingParts {
            identity: decode_type_identity(input)?,
            source: mutation_description::decode_description(input, budget)?,
            inverse: mutation_description::decode_description(input, budget)?,
            source_handler: input.text()?.to_owned(),
            inverse_handler: input.text()?.to_owned(),
            source_query: decode_optional(input, |input| Ok(input.text()?.to_owned()))?,
            inverse_query: decode_optional(input, |input| Ok(input.text()?.to_owned()))?,
            disclosure: disclosure::decode(input)?,
        },
    ))
}
