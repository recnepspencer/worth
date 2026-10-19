//! V26 carries the exact whole-input source, protocol and optional native guard.
use super::*;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectProtocol, ApplicationOperationBodyInputEmission as Emission,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    emission: &Emission,
) -> Result<(), Denial> {
    output.text(&emission.effect)?;
    output.text(emission.input_binding.as_str())?;
    output.text(emission.payload_binding.as_str())?;
    output.text(emission.protocol.identity().as_str())?;
    output.u32(emission.protocol.version().get())?;
    output.u64(emission.maximum_payload_bytes)?;
    super::requirements::write(output, emission.condition.as_slice())
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
    Ok(Emission {
        effect,
        input_binding,
        payload_binding,
        protocol: ApplicationExternalEffectProtocol::new(identity, version_protocol),
        maximum_payload_bytes,
        condition: conditions.pop(),
    })
}
