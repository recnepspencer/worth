//! Computed path writes reuse the root-path record and ordinary sequence budget.

use super::*;
use crate::record::application_schema::wire_vocabulary::{decode_presence, write_presence};
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyPathFieldSource,
    ApplicationOperationBodyPathValueAssignment as Assignment,
};
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    steps: &[Assignment],
) -> Result<(), Denial> {
    write_sequence(output, steps, |output, step| {
        super::super::application_query::write_root_path(output, &step.target.path)?;
        for part in [
            step.target.field.entity(),
            step.target.field.aspect(),
            step.target.field.field(),
            step.binding.as_str(),
            &step.value,
        ] {
            output.text(part)?;
        }
        write_presence(output, step.target.presence)
    })
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Vec<Assignment>, Denial> {
    decode_sequence(input, budget, 24, |input, budget| {
        let path = super::super::application_query::decode_root_path(input, budget)?;
        let field = ApplicationFieldBindingLocus::new(input.text()?, input.text()?, input.text()?);
        let binding =
            WorthQueryPortableTypeIdentity::from_untrusted(ARCHIVE_DECODE, input.text()?.into());
        let value = input.text()?.into();
        let presence = decode_presence(input)?;
        let target = ApplicationOperationBodyPathFieldSource {
            path,
            field,
            presence,
        };
        if !target.has_supported_shape() {
            return Err(Denial::new(Kind::InvalidRecordShape));
        }
        Ok(Assignment {
            target,
            binding,
            value,
        })
    })
}
