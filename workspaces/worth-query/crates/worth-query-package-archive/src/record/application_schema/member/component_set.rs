//! One additive member in the existing record grammar; no live graph authority.
use super::*;
use crate::record::sequence::{decode_sequence, write_sequence};
use worth_query_declaration::facade::application_schema::{
    ApplicationComponentSetMeaning, WorthQueryPortableApplicationComponentSetParts,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    definition: &ApplicationComponentSetMeaning,
) -> Result<(), Denial> {
    output.text(definition.name())?;
    output.text(definition.member())?;
    write_sequence(output, definition.relations(), |output, relation| {
        output.text(relation)
    })
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationSchemaMember, Denial> {
    budget.require_nesting_depth(2)?;
    let name = input.text()?.to_owned();
    let member = input.text()?.to_owned();
    // The sole sequence decoder admits nested entries and minimum frame bytes
    // before allocating the source roster.
    let relations = decode_sequence(input, budget, 4, |input, _| Ok(input.text()?.to_owned()))?;
    Ok(ApplicationSchemaMember::ComponentSet {
        definition: ApplicationComponentSetMeaning::from_untrusted_parts(
            WorthQueryPortableApplicationComponentSetParts {
                name,
                member,
                relations,
            },
        ),
    })
}
