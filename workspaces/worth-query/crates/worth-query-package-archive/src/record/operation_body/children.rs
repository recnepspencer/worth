//! Flat prior-parent site carriage reuses the existing initializer and link codecs.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyChildCreation as Child;
pub(super) fn write(output: &mut dyn BinaryEncodingSink, children: &[Child]) -> Result<(), Denial> {
    write_sequence(output, children, |output, child| {
        if !child.has_supported_shape() {
            return Err(Denial::new(Kind::InvalidRecordShape));
        }
        output.text(&child.parent_key)?;
        super::creation_fields::write(output, &child.creation)?;
        super::creation_links::write(output, child.creation.links(), 36)
    })
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    version: u16,
) -> Result<Vec<Child>, Denial> {
    decode_sequence(input, budget, 24, |input, budget| {
        let parent_key = input.text()?.to_owned();
        let creation = super::creation_fields::decode(input, budget, version, true)?
            .expect("explicit child site");
        let links = super::creation_links::decode(input, budget, version)?;
        if version < 39 && links.len() != 1 {
            return Err(Denial::new(Kind::InvalidRecordShape));
        }
        Ok(Child {
            parent_key,
            creation: creation.with_untrusted_links(links),
        })
    })
}
