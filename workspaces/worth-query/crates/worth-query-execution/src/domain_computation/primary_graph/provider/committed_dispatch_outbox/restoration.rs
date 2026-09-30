//! Restoration of the durable operation-bound dispatch-outbox projection.

use worth_foundational::facade::{
    AspectValue, BoundaryProtocolIdentity, BoundaryProtocolVersion, CanonicalDigestId,
    InternedString,
};

use super::Denial;
use crate::domain_computation::application_aftermath::{
    ExternalEffectCorrelationIdentity, WorthQueryDispatchOutboxLayout,
    WorthQueryDispatchOutboxRecord, WorthQueryDispatchOutboxRestoredFields,
};

pub(super) fn required_fields(
    layout: &WorthQueryDispatchOutboxLayout,
) -> Result<Vec<worth_foundational::facade::FieldKey>, Denial> {
    [
        &layout.correlation_locator,
        &layout.family_locator,
        &layout.effect_locator,
        &layout.protocol_identity_locator,
        &layout.protocol_version_locator,
        &layout.maximum_payload_bytes_locator,
        &layout.payload_locator,
        &layout.outcome_identity_locator,
        &layout.operation_slot_locator,
    ]
    .into_iter()
    .map(|locator| {
        locator
            .field_path()
            .fields()
            .first()
            .cloned()
            .ok_or(Denial::Malformed)
    })
    .collect()
}

pub(super) fn restore_record(
    values: Vec<AspectValue>,
) -> Result<WorthQueryDispatchOutboxRecord, Denial> {
    let [correlation, family, effect, protocol_identity, protocol_version, maximum, payload, outcome, operation_slot]: [AspectValue; 9] =
        values.try_into().map_err(|_| Denial::Malformed)?;
    let correlation = decode_digest(raw_string(correlation)?)?;
    let payload = decode_hex(raw_string(payload)?)?;
    let AspectValue::UInt64(maximum) = maximum else {
        return Err(Denial::Malformed);
    };
    let AspectValue::UInt64(protocol_version) = protocol_version else {
        return Err(Denial::Malformed);
    };
    let protocol_version = u32::try_from(protocol_version)
        .ok()
        .and_then(|value| BoundaryProtocolVersion::try_new(value).ok())
        .ok_or(Denial::Malformed)?;
    let AspectValue::UInt64(outcome) = outcome else {
        return Err(Denial::Malformed);
    };
    WorthQueryDispatchOutboxRecord::restore(WorthQueryDispatchOutboxRestoredFields {
        correlation: ExternalEffectCorrelationIdentity::from_digest(correlation),
        correlation_family: raw_string(family)?,
        effect: raw_string(effect)?,
        protocol_identity: BoundaryProtocolIdentity::parse(raw_string(protocol_identity)?)
            .map_err(|_| Denial::Malformed)?,
        protocol_version,
        maximum_payload_bytes: maximum,
        payload,
        outcome_identity: outcome,
        operation_slot: match raw_string(operation_slot)?.as_str() {
            "" => None,
            slot => Some(slot.to_owned()),
        },
    })
    .ok_or(Denial::Malformed)
}

fn raw_string(value: AspectValue) -> Result<String, Denial> {
    match value {
        AspectValue::String(InternedString::Raw(value)) => Ok(value),
        _ => Err(Denial::Malformed),
    }
}

fn decode_digest(value: String) -> Result<CanonicalDigestId, Denial> {
    let bytes: [u8; 32] = decode_hex(value)?
        .try_into()
        .map_err(|_| Denial::Malformed)?;
    Ok(CanonicalDigestId::new(bytes))
}

fn decode_hex(value: String) -> Result<Vec<u8>, Denial> {
    if !value.len().is_multiple_of(2) || !value.is_ascii() {
        return Err(Denial::Malformed);
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).map_err(|_| Denial::Malformed)?;
            u8::from_str_radix(text, 16).map_err(|_| Denial::Malformed)
        })
        .collect()
}

#[cfg(test)]
pub(super) fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
