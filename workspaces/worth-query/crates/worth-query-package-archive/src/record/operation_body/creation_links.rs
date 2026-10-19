//! Versioned descriptive creation links retain endpoint direction and local keys.
use super::{
    decode_sequence, write_sequence, BinaryEncodingSink, BinaryInput, Denial, Kind,
    RecordDecodeAttempt,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyCreationLink as Link,
    ApplicationOperationBodyCreationLinkDirection as Direction,
    ApplicationOperationBodyCreationLinkPosture as Posture,
};
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    links: &[Link],
    version: u16,
) -> Result<(), Denial> {
    write_sequence(output, links, |output, link| {
        output.text(&link.relation)?;
        output.text(&link.from)?;
        output.text(&link.to)?;
        output.text(&link.key)?;
        output.u16(match link.direction {
            Direction::FromScope => 1,
            Direction::ToScope => 2,
        })?;
        if version >= 19 {
            output.u16(match link.posture {
                Posture::Append => 0,
                Posture::ReplaceSingle => 1,
            })?;
        }
        Ok(())
    })
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    version: u16,
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
        let posture = if version >= 19 {
            match input.u16()? {
                0 => Posture::Append,
                1 => Posture::ReplaceSingle,
                _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
            }
        } else {
            Posture::Append
        };
        Ok(Link {
            relation,
            from,
            to,
            key,
            direction,
            posture,
        })
    })
}
