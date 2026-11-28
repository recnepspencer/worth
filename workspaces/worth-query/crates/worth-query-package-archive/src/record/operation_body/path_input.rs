//! Path-input guards reuse the existing root-path wire and nested-entry budget.

use super::*;
use crate::record::application_schema::wire_vocabulary::{decode_presence, write_presence};
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyPathFieldSource as Source,
    ApplicationOperationBodyPathInputExpectation as Guard,
};
pub(super) fn write(output: &mut dyn BinaryEncodingSink, guards: &[Guard]) -> Result<(), Denial> {
    write_sequence(output, guards, |output, guard| {
        super::super::application_query::write_root_path(output, &guard.source.path)?;
        for part in [
            guard.source.field.entity(),
            guard.source.field.aspect(),
            guard.source.field.field(),
            guard.binding.as_str(),
            &guard.member,
        ] {
            output.text(part)?;
        }
        super::super::foundational_aspect::write_scalar_type(output, guard.scalar_family)?;
        write_presence(output, guard.source.presence)
    })
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Vec<Guard>, Denial> {
    decode_sequence(input, budget, 24, |input, budget| {
        let path = super::super::application_query::decode_root_path(input, budget)?;
        let field = ApplicationFieldBindingLocus::new(input.text()?, input.text()?, input.text()?);
        let binding =
            WorthQueryPortableTypeIdentity::from_untrusted(ARCHIVE_DECODE, input.text()?.into());
        let member = input.text()?.into();
        let scalar_family = super::super::foundational_aspect::decode_scalar_type(input)?;
        let presence = decode_presence(input)?;
        let source = Source {
            path,
            field,
            presence,
        };
        if !source.has_supported_shape() {
            return Err(Denial::new(Kind::InvalidRecordShape));
        }
        Ok(Guard {
            source,
            binding,
            scalar_family,
            member,
        })
    })
}
