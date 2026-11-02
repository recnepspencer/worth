//! Owner-defined binary atoms. Width is platform meaning; admission and
//! canonical encoding remain the domain owner's responsibility.
mod decoding;
mod encoding;

use super::{ValueCodecDenial as D, ValueDecodeAdmission};
pub use decoding::{decode_binary_value, decode_binary_value_with_admission};
pub use encoding::encode_binary_value;

/// A fallible whole-chunk output. Implementations latch their first refusal,
/// including when an owner discards the returned error.
pub trait BinaryValueWriter {
    fn write(&mut self, bytes: &[u8]) -> Result<(), D>;
}

/// An entry-owned delegate can adopt a foreign pure value without implementing
/// a platform trait on that value. No Serde or Query trait is required.
pub trait BinaryValueDelegate {
    type Value: 'static;
    const MAXIMUM_WIRE_BYTES: usize;

    /// Emit only the owner's canonical, versioned representation.
    fn encode(value: &Self::Value, output: &mut dyn BinaryValueWriter) -> Result<(), D>;

    /// Reject non-canonical spellings, unsupported versions and invalid domain
    /// values. Charge variable work and allocation before performing it, using
    /// this original loan. The platform also charges input and re-encoding.
    fn admit(raw: &[u8], admission: &mut ValueDecodeAdmission<'_>) -> Result<Self::Value, D>;
}

fn maximum<Delegate: BinaryValueDelegate>() -> Result<usize, D> {
    let maximum = Delegate::MAXIMUM_WIRE_BYTES;
    if maximum == 0 || maximum > crate::values::InlineBytes::MAXIMUM_LENGTH {
        return Err(D::UnsupportedShape);
    }
    Ok(maximum)
}

#[cfg(test)]
mod cost;
#[cfg(test)]
mod tests;
