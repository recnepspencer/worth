use super::{maximum, BinaryValueDelegate, BinaryValueWriter, ValueDecodeAdmission, D};
use crate::value_shape::decode_control::RecordingAdmission;

/// Standalone representation admission uses exactly the same owner and
/// canonicality check, without claiming a native request grant.
pub fn decode_binary_value<Delegate: BinaryValueDelegate>(
    raw: &[u8],
) -> Result<Delegate::Value, D> {
    decode_recording::<Delegate>(raw, &mut |_, _| Ok(()), false)
}

/// No secondary request, allocation grant or cancellation source is invented.
pub fn decode_binary_value_with_admission<Delegate: BinaryValueDelegate>(
    raw: &[u8],
    admission: &mut ValueDecodeAdmission<'_>,
) -> Result<Delegate::Value, D> {
    decode_recording::<Delegate>(raw, admission, true)
}

fn decode_recording<Delegate: BinaryValueDelegate>(
    raw: &[u8],
    admission: &mut ValueDecodeAdmission<'_>,
    require_admission: bool,
) -> Result<Delegate::Value, D> {
    if raw.len() > maximum::<Delegate>()? {
        return Err(D::WireBytes);
    }
    let mut latched = RecordingAdmission {
        inner: admission,
        refusal: None,
        require_admission,
    };
    let result = decode::<Delegate>(raw, &mut latched);
    if let Some(cause) = latched.refusal {
        return Err(cause);
    }
    result
}

fn decode<Delegate: BinaryValueDelegate>(
    raw: &[u8],
    admission: &mut ValueDecodeAdmission<'_>,
) -> Result<Delegate::Value, D> {
    admission.admit(raw.len(), 0)?;
    let value = Delegate::admit(raw, admission)?;
    let mut comparison = Comparison {
        raw,
        cursor: 0,
        admission,
        refusal: None,
    };
    let result = Delegate::encode(&value, &mut comparison);
    if let Some(cause) = comparison.refusal {
        return Err(cause);
    }
    result?;
    if comparison.cursor != raw.len() {
        return Err(D::InternalInconsistency);
    }
    Ok(value)
}

/// Independent of the owner's decoding/normalization algorithm: compare each
/// emitted byte to the admitted source, without allocating a second encoding.
struct Comparison<'a, 'request> {
    raw: &'a [u8],
    cursor: usize,
    admission: &'a mut ValueDecodeAdmission<'request>,
    refusal: Option<D>,
}

impl Comparison<'_, '_> {
    fn compare(&mut self, bytes: &[u8]) -> Result<(), D> {
        self.admission.admit(bytes.len() + 1, 0)?;
        let end = self
            .cursor
            .checked_add(bytes.len())
            .ok_or(D::InternalInconsistency)?;
        if self.raw.get(self.cursor..end) != Some(bytes) {
            return Err(D::InternalInconsistency);
        }
        self.cursor = end;
        Ok(())
    }
}

impl BinaryValueWriter for Comparison<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> Result<(), D> {
        if let Some(cause) = self.refusal {
            return Err(cause);
        }
        self.compare(bytes)
            .inspect_err(|cause| self.refusal = Some(*cause))
    }
}
