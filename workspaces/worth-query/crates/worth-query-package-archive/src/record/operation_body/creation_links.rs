//! Versioned descriptive creation links retain endpoint direction and local keys.
use super::{
    decode_sequence, write_sequence, BinaryEncodingSink, BinaryInput, Denial, Kind,
    RecordDecodeAttempt,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyCreationLink as Link,
    ApplicationOperationBodyCreationLinkDirection as Direction,
};
pub(super) fn write(output: &mut dyn BinaryEncodingSink, links: &[Link]) -> Result<(), Denial> {
    write_sequence(output, links, |output, link| {
        output.text(&link.relation)?;
        output.text(&link.from)?;
        output.text(&link.to)?;
        output.text(&link.key)?;
        output.u16(match link.direction {
            Direction::FromScope => 1,
            Direction::ToScope => 2,
        })
    })
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Vec<Link>, Denial> {
    decode_sequence(input, budget, 18, |input, _| {
        let relation = input.text()?.to_owned();
        let from = input.text()?.to_owned();
        let to = input.text()?.to_owned();
        let key = input.text()?.to_owned();
        let direction = match input.u16()? {
            1 => Direction::FromScope,
            2 => Direction::ToScope,
            _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
        };
        Ok(Link {
            relation,
            from,
            to,
            key,
            direction,
        })
    })
}
