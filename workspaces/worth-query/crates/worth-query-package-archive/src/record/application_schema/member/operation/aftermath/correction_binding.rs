//! Portable encoding of the typed source and corrective action association.
use worth_query_declaration::facade::application_aftermath::{
    DeclaredCorrectionBinding, DeclaredCorrectionBindingParts, DeclaredReconciliationProcedure,
};

use super::super::super::mutation_description;
mod authored_identity;
mod disclosure;
mod source_outputs;
use crate::denial::WorthQueryPackageArchiveDenial as Denial;
use crate::record::application_schema::wire_vocabulary::{
    decode_optional, decode_type_identity, write_optional, write_type_identity,
};
use crate::record::decode_budget::RecordDecodeAttempt;
use crate::{binary_encoding::BinaryEncodingSink, binary_input::BinaryInput};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    binding: &DeclaredCorrectionBinding,
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
    disclosure::write(output, binding.disclosure())?;
    source_outputs::write(output, binding.source_output_fields())?;
    write_optional(
        output,
        binding.authored_identity(),
        authored_identity::write,
    )?;
    write_optional(
        output,
        binding.completed_external_reconciliation(),
        |output, procedure| output.text(procedure.procedure_slot()),
    )
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<DeclaredCorrectionBinding, Denial> {
    Ok(DeclaredCorrectionBinding::from_untrusted_parts(
        DeclaredCorrectionBindingParts {
            identity: decode_type_identity(input)?,
            source: mutation_description::decode_description(input, budget)?,
            inverse: mutation_description::decode_description(input, budget)?,
            source_handler: input.text()?.to_owned(),
            inverse_handler: input.text()?.to_owned(),
            source_query: decode_optional(input, |input| Ok(input.text()?.to_owned()))?,
            inverse_query: decode_optional(input, |input| Ok(input.text()?.to_owned()))?,
            disclosure: disclosure::decode(input)?,
            source_output_fields: source_outputs::decode(input, budget)?,
            authored_identity: decode_optional(input, authored_identity::decode)?,
            completed_external_reconciliation: decode_optional(input, |input| {
                DeclaredReconciliationProcedure::new(input.text()?.to_owned()).map_err(|_| {
                    Denial::new(
                        crate::denial::WorthQueryPackageArchiveDenialKind::InvalidRecordShape,
                    )
                })
            })?,
        },
    ))
}
