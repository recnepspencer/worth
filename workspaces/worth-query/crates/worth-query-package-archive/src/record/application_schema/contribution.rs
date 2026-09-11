use worth_query_declaration::facade::application_schema::ApplicationSchemaContributionProvenance;

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::WorthQueryPackageArchiveDenial as Denial;

use super::super::decode_budget::RecordDecodeAttempt;
use super::super::sequence::{decode_sequence, write_sequence};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    contribution: &ApplicationSchemaContributionProvenance,
) -> Result<(), Denial> {
    output.text(contribution.identity().as_str())?;
    write_sequence(output, contribution.member_ordinals(), |output, ordinal| {
        output.u32(*ordinal)
    })
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationSchemaContributionProvenance, Denial> {
    let identity = input.text()?.to_owned();
    let member_ordinals = decode_sequence(input, budget, 4, |input, _| input.u32())?;
    Ok(ApplicationSchemaContributionProvenance::from_untrusted_parts(identity, member_ordinals))
}
