//! V26 carries the exact whole-input source, protocol and optional native guard.
use super::*;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectProtocol, ApplicationOperationBodyExternalEmission as Emission,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    emission: &Emission,
    version: u16,
) -> Result<(), Denial> {
    output.text(&emission.effect)?;
    output.text(emission.input_binding.as_str())?;
    output.text(emission.payload_binding.as_str())?;
    output.text(emission.protocol.identity().as_str())?;
    output.u32(emission.protocol.version().get())?;
    output.u64(emission.maximum_payload_bytes)?;
    super::requirements::write(output, emission.condition.as_slice())?;
    if version >= 28 {
        use worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource as Source;
        output.u8(match emission.source {
            Source::OperationInput => 0,
            Source::QueryContext { .. } => 1,
            Source::Evaluation { .. } => 2,
        })?;
    }
    if let worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource::QueryContext { mutation_binding, query }
        | worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource::Evaluation { mutation_binding, query } = &emission.source {
        output.text(mutation_binding)?;
        crate::record::application_query::write(output, query)?;
    }
    Ok(())
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    version: u16,
) -> Result<Emission, Denial> {
    let effect = input.text()?.to_owned();
    let input_binding = WorthQueryPortableTypeIdentity::from_untrusted(input.text()?.to_owned());
    let payload_binding = WorthQueryPortableTypeIdentity::from_untrusted(input.text()?.to_owned());
    let identity = BoundaryProtocolIdentity::parse(input.text()?.to_owned())
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let version_protocol = BoundaryProtocolVersion::try_new(input.u32()?)
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let maximum_payload_bytes = input.u64()?;
    let mut conditions = super::requirements::decode(input, budget, version)?;
    if conditions.len() > 1 {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    use worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource as Source;
    let tag = if version >= 28 {
        input.u8()?
    } else {
        u8::from(version >= 27)
    };
    let source = match tag {
        0 => Source::OperationInput,
        1 | 2 => {
            let mutation_binding = input.text()?.to_owned();
            let query = Box::new(crate::record::application_query::decode_nested(
                input, budget,
            )?);
            if tag == 1 {
                Source::QueryContext {
                    mutation_binding,
                    query,
                }
            } else {
                Source::Evaluation {
                    mutation_binding,
                    query,
                }
            }
        }
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(Emission {
        source,
        effect,
        input_binding,
        payload_binding,
        protocol: ApplicationExternalEffectProtocol::new(identity, version_protocol),
        maximum_payload_bytes,
        condition: conditions.pop(),
    })
}
