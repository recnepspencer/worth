//! Output-role descriptions use native enum tags, not executable authority.
use super::{
    decode_sequence, write_sequence, BinaryEncodingSink, BinaryInput, Denial, Kind,
    RecordDecodeAttempt,
};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationOutputPosture as Posture,
        ApplicationMutationOutputRoleCardinality as Cardinality,
        ApplicationMutationOutputRoleDescription,
    },
    application_schema::{
        ApplicationOperationBodyOutput, ApplicationOperationBodyOutputTarget as Target,
    },
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    outputs: &[ApplicationOperationBodyOutput],
) -> Result<(), Denial> {
    write_sequence(output, outputs, |output, item| {
        output.text(&item.binding)?;
        output.text(&item.role.name)?;
        output.text(&item.role.entity)?;
        output.u16(match item.role.posture {
            Posture::Preserve => 1,
            Posture::Create => 2,
            Posture::Retire => 3,
        })?;
        output.u16(match item.role.cardinality {
            Cardinality::ExactlyOne => 1,
            Cardinality::AtMostOne => 2,
        })?;
        match &item.target {
            Target::Scope => output.u16(1),
            Target::Creation { key } => {
                output.u16(2)?;
                output.text(key)
            }
            Target::Related { relation, from, to } => {
                output.u16(3)?;
                for text in [relation, from, to] {
                    output.text(text)?;
                }
                Ok(())
            }
        }
    })
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    version: u16,
) -> Result<Vec<ApplicationOperationBodyOutput>, Denial> {
    decode_sequence(input, budget, 22, |input, _| {
        let binding = input.text()?.to_owned();
        let name = input.text()?.to_owned();
        let entity = input.text()?.to_owned();
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
        let target = match input.u16()? {
            1 => Target::Scope,
            2 => Target::Creation {
                key: input.text()?.to_owned(),
            },
            3 if version >= 22 => Target::Related {
                relation: input.text()?.to_owned(),
                from: input.text()?.to_owned(),
                to: input.text()?.to_owned(),
            },
            _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
        };
        Ok(ApplicationOperationBodyOutput {
            binding,
            role: ApplicationMutationOutputRoleDescription {
                name,
                entity,
                posture,
                cardinality,
            },
            target,
        })
    })
}
