//! v21 preserves ordinary succession; v23 adds a declared historical output match.
use super::{BinaryEncodingSink, BinaryInput, Denial, Kind};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationOutputPosture as Posture,
        ApplicationMutationOutputRoleCardinality as Cardinality,
        ApplicationMutationOutputRoleDescription,
    },
    application_schema::{
        ApplicationOperationBodySuccession as Succession,
        ApplicationOperationBodySuccessionOutputMatch as OutputMatch,
    },
};
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    succession: Option<&Succession>,
) -> Result<(), Denial> {
    let tag = match succession {
        None => 0,
        Some(s) if s.output_match().is_some() => 2,
        Some(_) => 1,
    };
    output.u8(tag)?;
    if let Some(succession) = succession {
        output.text(succession.workflow_spec())?;
        output.text(succession.binding())?;
        output.text(succession.role())?;
        if let Some(matching) = succession.output_match() {
            for text in [
                matching.selected_role(),
                matching.producer_binding(),
                &matching.producer_role().name,
                &matching.producer_role().entity,
            ] {
                output.text(text)?;
            }
            output.u16(match matching.producer_role().posture {
                Posture::Preserve => 1,
                Posture::Create => 2,
                Posture::Retire => 3,
            })?;
            output.u16(match matching.producer_role().cardinality {
                Cardinality::ExactlyOne => 1,
                Cardinality::AtMostOne => 2,
            })?;
        }
    }
    Ok(())
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    version: u16,
) -> Result<Option<Succession>, Denial> {
    let tag = input.u8()?;
    match tag {
        0 => Ok(None),
        1 => Ok(Some(decode_base(input)?)),
        2 if version >= 23 => {
            let succession = decode_base(input)?;
            let selected = input.text()?.into();
            let producer = input.text()?.into();
            let name = input.text()?.into();
            let entity = input.text()?.into();
            let posture = match input.u16()? {
                1 => Posture::Preserve,
                2 => Posture::Create,
                3 => Posture::Retire,
                _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
            };
            let cardinality = match input.u16()? {
                1 => Cardinality::ExactlyOne,
                2 => Cardinality::AtMostOne,
                _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
            };
            Ok(Some(succession.with_untrusted_output_match(
                OutputMatch::from_untrusted(
                    selected,
                    producer,
                    ApplicationMutationOutputRoleDescription {
                        name,
                        entity,
                        posture,
                        cardinality,
                    },
                ),
            )))
        }
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
fn decode_base(input: &mut BinaryInput<'_>) -> Result<Succession, Denial> {
    Ok(Succession::from_untrusted(
        input.text()?.into(),
        input.text()?.into(),
        input.text()?.into(),
    ))
}
