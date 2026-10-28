//! Native-request decoding. Standalone representation decoding remains separate.
use super::{
    CanonicalValue, DelegatedValueSeed, ValueCodecDenial as D, ValueDecodeAdmission,
    ValueDecodeBounds, ValueDelegate, ValueSeedDelegate,
};
use serde::de::DeserializeSeed;
use serde::Serialize;
use std::io::Write;

/// Decode through one borrowed admission callback supplied by the native owner.
/// The callback is not authority itself. The first refused grant wins, including
/// when an opaque seed catches its error and returns a value anyway.
pub fn decode_canonical_value_with_admission<Value: CanonicalValue>(
    raw: &str,
    bounds: ValueDecodeBounds,
    admission: &mut ValueDecodeAdmission<'_>,
) -> Result<Value, D> {
    decode_recording::<Value>(raw, bounds, admission, true)
}

pub(super) fn decode_recording<Value: CanonicalValue>(
    raw: &str,
    bounds: ValueDecodeBounds,
    admission: &mut ValueDecodeAdmission<'_>,
    require_admission: bool,
) -> Result<Value, D> {
    decode_delegate_recording::<ValueDelegate<Value>>(raw, bounds, admission, require_admission)
}

/// Bounded canonical decoding using the same seed contract as a declared field.
pub fn decode_delegated_canonical_value<Delegate: ValueSeedDelegate>(
    raw: &str,
    bounds: ValueDecodeBounds,
) -> Result<Delegate::Value, D> {
    decode_delegate_recording::<Delegate>(raw, bounds, &mut |_, _| Ok(()), false)
}

/// The caller's admission loan reaches every nested seed and canonical check.
pub fn decode_delegated_canonical_value_with_admission<Delegate: ValueSeedDelegate>(
    raw: &str,
    bounds: ValueDecodeBounds,
    admission: &mut ValueDecodeAdmission<'_>,
) -> Result<Delegate::Value, D> {
    decode_delegate_recording::<Delegate>(raw, bounds, admission, true)
}

fn decode_delegate_recording<Delegate: ValueSeedDelegate>(
    raw: &str,
    bounds: ValueDecodeBounds,
    admission: &mut ValueDecodeAdmission<'_>,
    require_admission: bool,
) -> Result<Delegate::Value, D> {
    if bounds
        .maximum_wire_bytes()
        .is_some_and(|maximum| raw.len() > maximum)
    {
        return Err(D::WireBytes);
    }
    if require_admission && !Delegate::SUPPORTS_ADMISSION {
        return Err(D::UnsupportedAdmission);
    }
    let mut latched = super::decode_control::RecordingAdmission {
        inner: admission,
        refusal: None,
        require_admission,
    };
    let result = decode::<Delegate>(raw, bounds, &mut latched, require_admission);
    if let Some(denial) = latched.refusal {
        return Err(denial);
    }
    result
}

fn decode<Delegate: ValueSeedDelegate>(
    raw: &str,
    bounds: ValueDecodeBounds,
    admission: &mut ValueDecodeAdmission<'_>,
    require_admission: bool,
) -> Result<Delegate::Value, D> {
    admission.admit(raw.len(), 0)?;
    if require_admission {
        super::ValueShapeDefinition::from_delegate::<Delegate>().inspect_admitted(admission)?;
    } else {
        super::ValueShapeDefinition::from_delegate::<Delegate>().inspect()?;
    }
    let mut decoder = serde_json::Deserializer::from_str(raw);
    decoder.disable_recursion_limit();
    let seed = DelegatedValueSeed::<Delegate>::new(bounds, Some(admission));
    let value = seed
        .deserialize(serde_stacker::Deserializer::new(&mut decoder))
        .map_err(|_| D::Malformed)?;
    decoder.end().map_err(|_| D::Malformed)?;
    Delegate::validate(&value, bounds)?;
    // Compare canonical serialization directly against the admitted source.
    // This is the actual serializer, with no second owned encoding allocation.
    let mut comparison = CanonicalComparison {
        raw: raw.as_bytes(),
        cursor: 0,
        admission,
        denial: None,
    };
    let mut serializer = serde_json::Serializer::new(&mut comparison);
    if value
        .serialize(serde_stacker::Serializer::new(&mut serializer))
        .is_err()
    {
        return Err(comparison.denial.unwrap_or(D::Malformed));
    }
    if let Some(denial) = comparison.denial {
        return Err(denial);
    }
    if comparison.cursor != raw.len() {
        return Err(D::NonCanonical);
    }
    Ok(value)
}

struct CanonicalComparison<'a, 'b> {
    raw: &'a [u8],
    cursor: usize,
    admission: &'a mut ValueDecodeAdmission<'b>,
    denial: Option<D>,
}
impl Write for CanonicalComparison<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let result = if let Some(denial) = self.denial {
            Err(denial)
        } else {
            self.admission.admit(bytes.len(), 0).and_then(|()| {
                let end = self
                    .cursor
                    .checked_add(bytes.len())
                    .ok_or(D::NonCanonical)?;
                if self.raw.get(self.cursor..end) != Some(bytes) {
                    return Err(D::NonCanonical);
                }
                self.cursor = end;
                Ok(())
            })
        };
        if let Err(denial) = result {
            self.denial = Some(denial);
            return Err(std::io::Error::other(denial));
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
