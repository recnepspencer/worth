use super::{maximum, BinaryValueDelegate, BinaryValueWriter, D};
use crate::values::InlineBytes;

/// O(encoded bytes), with geometric fallible growth capped by the owner's
/// finite wire bound. Moving the result into the scalar does not copy it.
pub fn encode_binary_value<Delegate: BinaryValueDelegate>(
    value: &Delegate::Value,
) -> Result<InlineBytes, D> {
    let mut writer = BoundedWriter {
        bytes: Vec::new(),
        maximum: maximum::<Delegate>()?,
        refusal: None,
    };
    let result = Delegate::encode(value, &mut writer);
    if let Some(cause) = writer.refusal {
        return Err(cause);
    }
    result?;
    InlineBytes::try_new(writer.bytes).map_err(|_| D::WireBytes)
}

struct BoundedWriter {
    bytes: Vec<u8>,
    maximum: usize,
    refusal: Option<D>,
}

impl BoundedWriter {
    fn append(&mut self, bytes: &[u8]) -> Result<(), D> {
        let end = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .ok_or(D::WireBytes)?;
        if end > self.maximum {
            return Err(D::WireBytes);
        }
        if end > self.bytes.capacity() {
            let target = self
                .bytes
                .capacity()
                .saturating_mul(2)
                .max(end)
                .min(self.maximum);
            self.bytes
                .try_reserve_exact(target - self.bytes.len())
                .map_err(|_| D::Allocation)?;
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
}

impl BinaryValueWriter for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> Result<(), D> {
        if let Some(cause) = self.refusal {
            return Err(cause);
        }
        self.append(bytes)
            .inspect_err(|cause| self.refusal = Some(*cause))
    }
}
