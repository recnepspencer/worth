//! Versioned descriptive creation links retain endpoint direction and local keys.
use super::{
    decode_sequence, write_sequence, BinaryEncodingSink, BinaryInput, Denial, Kind,
    RecordDecodeAttempt,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldBindingLocus, ApplicationOperationBodyCreationLink as Link,
    ApplicationOperationBodyCreationLinkDirection as Direction,
    ApplicationOperationBodyCreationLinkPosture as Posture,
    ApplicationOperationBodyCreationLinkSource as Source,
};
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    links: &[Link],
    version: u16,
) -> Result<(), Denial> {
    write_sequence(output, links, |output, link| {
        if version < 40 && !matches!(link.source, Source::Scope) {
            return Err(Denial::new(Kind::UnsupportedRecordVariant));
        }
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
                Posture::ReplaceAndRetireSingle if version >= 34 => 2,
                Posture::ReplaceAndRetireSingle => {
                    return Err(Denial::new(Kind::UnsupportedRecordVariant))
                }
            })?;
        }
        if version >= 40 {
            match &link.source {
                Source::Scope => output.u16(0)?,
                Source::Principal {
                    mutation_binding,
                    principal_binding,
                    identity,
                } => {
                    output.u16(1)?;
                    for text in [
                        mutation_binding.as_str(),
                        principal_binding.as_str(),
                        identity.entity(),
                        identity.aspect(),
                        identity.field(),
                    ] {
                        output.text(text)?;
                    }
                }
            }
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
                2 if version >= 34 => Posture::ReplaceAndRetireSingle,
                _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
            }
        } else {
            Posture::Append
        };
        let source = if version >= 40 {
            match input.u16()? {
                0 => Source::Scope,
                1 => Source::Principal {
                    mutation_binding: input.text()?.into(),
                    principal_binding: input.text()?.into(),
                    identity: ApplicationFieldBindingLocus::new(
                        input.text()?,
                        input.text()?,
                        input.text()?,
                    ),
                },
                _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
            }
        } else {
            Source::Scope
        };
        Ok(Link {
            relation,
            from,
            to,
            key,
            direction,
            posture,
            source,
        })
    })
}
