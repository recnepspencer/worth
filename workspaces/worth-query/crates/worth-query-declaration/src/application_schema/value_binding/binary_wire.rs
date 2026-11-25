//! Fixed-width canonical binary field wire contract and default scalar binding lookup.
//!
//! Used by `#[derive(WorthBinaryValue)]` and `#[derive(WorthManagedIndex)]` to
//! enforce compile-time fixed wire widths, canonical scalar forms, and default
//! binding resolution without raw floats or unbounded allocations.
use super::{
    ApplicationReadableScalarValueBinding, BinaryValueWriter, BoolApplicationValueBinding,
    CanonicalDateApplicationValueBinding, CanonicalF32ApplicationValueBinding,
    CanonicalF64ApplicationValueBinding, CanonicalTimeApplicationValueBinding,
    CanonicalTimestampApplicationValueBinding, CanonicalTimestampTzApplicationValueBinding,
    I16ApplicationValueBinding, I32ApplicationValueBinding, I64ApplicationValueBinding,
    I8ApplicationValueBinding, InlineBytesApplicationValueBinding,
    InternedStringApplicationValueBinding, StringApplicationValueBinding,
    U16ApplicationValueBinding, U32ApplicationValueBinding, U64ApplicationValueBinding,
    U8ApplicationValueBinding, UuidApplicationValueBinding, ValueCodecDenial, ValueDecodeAdmission,
};
use worth_foundational::facade::{
    AspectValue, CanonicalDate, CanonicalDigestId, CanonicalF32, CanonicalF64, CanonicalTime,
    CanonicalTimestamp, CanonicalTimestampTz, InlineBytes, InternedString,
};

/// Checked compile-time sum of binary field wire widths within [`InlineBytes::MAXIMUM_LENGTH`].
///
/// Panics during `const` evaluation on integer overflow or when the accumulated
/// size exceeds `InlineBytes::MAXIMUM_LENGTH`.
pub const fn worth_binary_checked_wire_bytes(base: usize, add: usize) -> usize {
    let Some(total) = base.checked_add(add) else {
        panic!("WorthBinaryValue wire byte size overflowed usize");
    };
    if total > InlineBytes::MAXIMUM_LENGTH {
        panic!("WorthBinaryValue wire byte size exceeds InlineBytes::MAXIMUM_LENGTH");
    }
    total
}

mod sealed {
    pub trait Sealed {}
}

/// Sealed canonical fixed-width little-endian wire encoding for one field of a
/// `#[derive(WorthBinaryValue)]` struct.
///
/// Sealed inside `worth-query-declaration` to fixed-width, reference-free
/// primitive and canonical scalar carriers so downstream wrappers around
/// unbounded or reference-bearing types cannot forge wire widths.
pub trait WorthBinaryWireField: sealed::Sealed + Sized + 'static {
    const WIRE_BYTES: usize;

    fn encode_binary_field(
        &self,
        writer: &mut dyn BinaryValueWriter,
    ) -> Result<(), ValueCodecDenial>;

    fn decode_binary_field(
        bytes: &[u8],
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, ValueCodecDenial>;
}

/// Resolves a value type to its canonical default readable scalar binding.
///
/// Implemented for all native scalar carriers and automatically derived for
/// structs annotated with `#[derive(WorthBinaryValue)]`.
pub trait WorthQueryDefaultValueBinding: Sized + 'static {
    type Binding: ApplicationReadableScalarValueBinding<Value = Self>;
}

impl sealed::Sealed for bool {}
impl WorthBinaryWireField for bool {
    const WIRE_BYTES: usize = 1;

    fn encode_binary_field(
        &self,
        writer: &mut dyn BinaryValueWriter,
    ) -> Result<(), ValueCodecDenial> {
        writer.write(&[u8::from(*self)])
    }

    fn decode_binary_field(
        bytes: &[u8],
        _admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, ValueCodecDenial> {
        match bytes {
            [0] => Ok(false),
            [1] => Ok(true),
            [_] => Err(ValueCodecDenial::NonCanonical),
            _ => Err(ValueCodecDenial::Malformed),
        }
    }
}

macro_rules! impl_le_int_wire_field {
    ($($ty:ty => $bytes:expr),+ $(,)?) => {
        $(
            impl sealed::Sealed for $ty {}
            impl WorthBinaryWireField for $ty {
                const WIRE_BYTES: usize = $bytes;

                fn encode_binary_field(
                    &self,
                    writer: &mut dyn BinaryValueWriter,
                ) -> Result<(), ValueCodecDenial> {
                    writer.write(&self.to_le_bytes())
                }

                fn decode_binary_field(
                    bytes: &[u8],
                    _admission: &mut ValueDecodeAdmission<'_>,
                ) -> Result<Self, ValueCodecDenial> {
                    let raw = <[u8; $bytes]>::try_from(bytes)
                        .map_err(|_| ValueCodecDenial::Malformed)?;
                    Ok(<$ty>::from_le_bytes(raw))
                }
            }
        )+
    };
}

impl_le_int_wire_field!(
    u8 => 1,
    u16 => 2,
    u32 => 4,
    u64 => 8,
    i8 => 1,
    i16 => 2,
    i32 => 4,
    i64 => 8,
);

impl<const N: usize> sealed::Sealed for [u8; N] {}
impl<const N: usize> WorthBinaryWireField for [u8; N] {
    const WIRE_BYTES: usize = N;

    fn encode_binary_field(
        &self,
        writer: &mut dyn BinaryValueWriter,
    ) -> Result<(), ValueCodecDenial> {
        writer.write(self)
    }

    fn decode_binary_field(
        bytes: &[u8],
        _admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, ValueCodecDenial> {
        <[u8; N]>::try_from(bytes).map_err(|_| ValueCodecDenial::Malformed)
    }
}

impl sealed::Sealed for CanonicalDigestId {}
impl WorthBinaryWireField for CanonicalDigestId {
    const WIRE_BYTES: usize = 32;

    fn encode_binary_field(
        &self,
        writer: &mut dyn BinaryValueWriter,
    ) -> Result<(), ValueCodecDenial> {
        writer.write(self.bytes())
    }

    fn decode_binary_field(
        bytes: &[u8],
        _admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, ValueCodecDenial> {
        let raw = <[u8; 32]>::try_from(bytes).map_err(|_| ValueCodecDenial::Malformed)?;
        Ok(Self::new(raw))
    }
}

impl sealed::Sealed for CanonicalDate {}
impl WorthBinaryWireField for CanonicalDate {
    const WIRE_BYTES: usize = 4;

    fn encode_binary_field(
        &self,
        writer: &mut dyn BinaryValueWriter,
    ) -> Result<(), ValueCodecDenial> {
        writer.write(&self.days_from_unix_epoch.to_le_bytes())
    }

    fn decode_binary_field(
        bytes: &[u8],
        _admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, ValueCodecDenial> {
        let raw = <[u8; 4]>::try_from(bytes).map_err(|_| ValueCodecDenial::Malformed)?;
        Ok(Self {
            days_from_unix_epoch: i32::from_le_bytes(raw),
        })
    }
}

impl sealed::Sealed for CanonicalTime {}
impl WorthBinaryWireField for CanonicalTime {
    const WIRE_BYTES: usize = 8;

    fn encode_binary_field(
        &self,
        writer: &mut dyn BinaryValueWriter,
    ) -> Result<(), ValueCodecDenial> {
        if !AspectValue::Time(*self).has_canonical_representation() {
            return Err(ValueCodecDenial::NonCanonical);
        }
        writer.write(&self.nanos_since_midnight.to_le_bytes())
    }

    fn decode_binary_field(
        bytes: &[u8],
        _admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, ValueCodecDenial> {
        let raw = <[u8; 8]>::try_from(bytes).map_err(|_| ValueCodecDenial::Malformed)?;
        let value = Self {
            nanos_since_midnight: u64::from_le_bytes(raw),
        };
        if !AspectValue::Time(value).has_canonical_representation() {
            return Err(ValueCodecDenial::NonCanonical);
        }
        Ok(value)
    }
}

impl sealed::Sealed for CanonicalTimestamp {}
impl WorthBinaryWireField for CanonicalTimestamp {
    const WIRE_BYTES: usize = 8;

    fn encode_binary_field(
        &self,
        writer: &mut dyn BinaryValueWriter,
    ) -> Result<(), ValueCodecDenial> {
        writer.write(&self.micros_since_unix_epoch.to_le_bytes())
    }

    fn decode_binary_field(
        bytes: &[u8],
        _admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, ValueCodecDenial> {
        let raw = <[u8; 8]>::try_from(bytes).map_err(|_| ValueCodecDenial::Malformed)?;
        Ok(Self {
            micros_since_unix_epoch: i64::from_le_bytes(raw),
        })
    }
}

impl sealed::Sealed for CanonicalTimestampTz {}
impl WorthBinaryWireField for CanonicalTimestampTz {
    const WIRE_BYTES: usize = 12;

    fn encode_binary_field(
        &self,
        writer: &mut dyn BinaryValueWriter,
    ) -> Result<(), ValueCodecDenial> {
        if !AspectValue::TimestampTz(*self).has_canonical_representation() {
            return Err(ValueCodecDenial::NonCanonical);
        }
        writer.write(&self.utc_micros_since_unix_epoch.to_le_bytes())?;
        writer.write(&self.offset_minutes.to_le_bytes())
    }

    fn decode_binary_field(
        bytes: &[u8],
        _admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, ValueCodecDenial> {
        if bytes.len() != Self::WIRE_BYTES {
            return Err(ValueCodecDenial::Malformed);
        }
        let utc_raw = <[u8; 8]>::try_from(&bytes[..8]).map_err(|_| ValueCodecDenial::Malformed)?;
        let off_raw =
            <[u8; 4]>::try_from(&bytes[8..12]).map_err(|_| ValueCodecDenial::Malformed)?;
        let value = Self {
            utc_micros_since_unix_epoch: i64::from_le_bytes(utc_raw),
            offset_minutes: i32::from_le_bytes(off_raw),
        };
        if !AspectValue::TimestampTz(value).has_canonical_representation() {
            return Err(ValueCodecDenial::NonCanonical);
        }
        Ok(value)
    }
}

impl sealed::Sealed for CanonicalF32 {}
impl WorthBinaryWireField for CanonicalF32 {
    const WIRE_BYTES: usize = 4;

    fn encode_binary_field(
        &self,
        writer: &mut dyn BinaryValueWriter,
    ) -> Result<(), ValueCodecDenial> {
        if !AspectValue::Float32(*self).has_canonical_representation() {
            return Err(ValueCodecDenial::NonCanonical);
        }
        writer.write(&self.bits().to_le_bytes())
    }

    fn decode_binary_field(
        bytes: &[u8],
        _admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, ValueCodecDenial> {
        let raw = <[u8; 4]>::try_from(bytes).map_err(|_| ValueCodecDenial::Malformed)?;
        let value = Self(u32::from_le_bytes(raw));
        if !AspectValue::Float32(value).has_canonical_representation() {
            return Err(ValueCodecDenial::NonCanonical);
        }
        Ok(value)
    }
}

impl sealed::Sealed for CanonicalF64 {}
impl WorthBinaryWireField for CanonicalF64 {
    const WIRE_BYTES: usize = 8;

    fn encode_binary_field(
        &self,
        writer: &mut dyn BinaryValueWriter,
    ) -> Result<(), ValueCodecDenial> {
        if !AspectValue::Float64(*self).has_canonical_representation() {
            return Err(ValueCodecDenial::NonCanonical);
        }
        writer.write(&self.bits().to_le_bytes())
    }

    fn decode_binary_field(
        bytes: &[u8],
        _admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, ValueCodecDenial> {
        let raw = <[u8; 8]>::try_from(bytes).map_err(|_| ValueCodecDenial::Malformed)?;
        let value = Self(u64::from_le_bytes(raw));
        if !AspectValue::Float64(value).has_canonical_representation() {
            return Err(ValueCodecDenial::NonCanonical);
        }
        Ok(value)
    }
}

macro_rules! impl_default_value_binding {
    ($($ty:ty => $binding:ty),+ $(,)?) => {
        $(
            impl WorthQueryDefaultValueBinding for $ty {
                type Binding = $binding;
            }
        )+
    };
}

impl_default_value_binding!(
    bool => BoolApplicationValueBinding,
    i8 => I8ApplicationValueBinding,
    i16 => I16ApplicationValueBinding,
    i32 => I32ApplicationValueBinding,
    i64 => I64ApplicationValueBinding,
    u8 => U8ApplicationValueBinding,
    u16 => U16ApplicationValueBinding,
    u32 => U32ApplicationValueBinding,
    u64 => U64ApplicationValueBinding,
    String => StringApplicationValueBinding,
    InternedString => InternedStringApplicationValueBinding,
    CanonicalDate => CanonicalDateApplicationValueBinding,
    CanonicalTime => CanonicalTimeApplicationValueBinding,
    CanonicalTimestamp => CanonicalTimestampApplicationValueBinding,
    CanonicalTimestampTz => CanonicalTimestampTzApplicationValueBinding,
    [u8; 16] => UuidApplicationValueBinding,
    CanonicalF32 => CanonicalF32ApplicationValueBinding,
    CanonicalF64 => CanonicalF64ApplicationValueBinding,
    InlineBytes => InlineBytesApplicationValueBinding,
);

#[cfg(test)]
mod tests;
