//! Immutable evaluation values.
//!
//! A value carries no type of its own: the admitted program and the operand
//! schema supply it, and binding checks every operand value against its
//! declared slot type before evaluation reads it. Composite parts are shared,
//! so a retained snapshot is never copied wholesale.

use std::cmp::Ordering;
use std::sync::Arc;

use crate::expressions::denial::{ExpressionDenial, ExpressionDenialDetail, ExpressionResult};
use crate::values::{CanonicalDecimal, CanonicalF32, CanonicalF64};

/// Decimal values carry at most 38 significant digits and scale 0..=18.
pub(crate) const DECIMAL_DIGITS: u32 = 38;
pub(crate) const DECIMAL_SCALE: u8 = 18;

/// An immutable expression value.
///
/// Floats are finite with zero normalized to positive zero, decimals are
/// normalized, and map entries are held in canonical key order, so equal
/// values have equal representations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionValue(pub(crate) Repr);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Repr {
    Bool(bool),
    Integer(i128),
    Float32(u32),
    Float64(u64),
    Decimal {
        coefficient: i128,
        scale: u8,
    },
    String(Arc<str>),
    Bytes(Arc<[u8]>),
    Id(Arc<[u8]>),
    Enum(u32),
    /// Canonical SI magnitude bits.
    Quantity(u64),
    /// Little-endian limbs; bits at and above `width` are zero.
    Bits {
        width: u32,
        limbs: Arc<[u64]>,
    },
    /// Bitplanes `(value, unknown)`: 0=(0,0), 1=(1,0), X=(0,1), Z=(1,1).
    Logic4 {
        width: u32,
        value: Arc<[u64]>,
        unknown: Arc<[u64]>,
    },
    None,
    Some(Arc<ExpressionValue>),
    List(Arc<Composite>),
    /// Items alternate key and value, in canonical key order.
    Map(Arc<Composite>),
    /// Field values in schema order.
    Record(Arc<Composite>),
    /// `[key, value]`, as `entries(map)` yields.
    Entry(Arc<Composite>),
}

/// Shared parts plus their logical size, computed once at construction.
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct Composite {
    pub(crate) items: Box<[ExpressionValue]>,
    pub(crate) bytes: u64,
}

impl Composite {
    pub(crate) fn new(items: Vec<ExpressionValue>) -> Arc<Self> {
        let bytes = items
            .iter()
            .fold(8_u64, |sum, item| sum.saturating_add(item.logical_bytes()));
        Arc::new(Self {
            items: items.into_boxed_slice(),
            bytes,
        })
    }
}

fn invalid(reason: &'static str) -> ExpressionDenial {
    ExpressionDenial::new(ExpressionDenialDetail::InvalidValue(reason))
}

impl ExpressionValue {
    pub fn bool(value: bool) -> Self {
        Self(Repr::Bool(value))
    }

    /// An integer; binding checks it against the declared width.
    pub fn integer(value: i128) -> Self {
        Self(Repr::Integer(value))
    }

    /// Denies NaN and infinities; negative zero becomes positive zero.
    pub fn float32(value: CanonicalF32) -> ExpressionResult<Self> {
        let value = value.as_f32();
        if value.is_finite() {
            Ok(Self(Repr::Float32((value + 0.0).to_bits())))
        } else {
            Err(invalid("Float32 values are finite"))
        }
    }

    /// Denies NaN and infinities; negative zero becomes positive zero.
    pub fn float64(value: CanonicalF64) -> ExpressionResult<Self> {
        Self::finite_f64(value.as_f64())
            .map(|bits| Self(Repr::Float64(bits)))
            .ok_or_else(|| invalid("Float64 values are finite"))
    }

    /// A quantity from its canonical SI magnitude.
    pub fn quantity(magnitude: CanonicalF64) -> ExpressionResult<Self> {
        Self::finite_f64(magnitude.as_f64())
            .map(|bits| Self(Repr::Quantity(bits)))
            .ok_or_else(|| invalid("quantity magnitudes are finite"))
    }

    fn finite_f64(value: f64) -> Option<u64> {
        value.is_finite().then(|| (value + 0.0).to_bits())
    }

    /// Reads canonical decimal text and normalizes it for expressions, without
    /// changing the carrier's own identity.
    pub fn decimal(value: &CanonicalDecimal) -> ExpressionResult<Self> {
        let text = value.as_str();
        let (negative, unsigned) = match text.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, text),
        };
        let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
        if whole.is_empty() || !digits(whole) || !digits(fraction) {
            return Err(invalid("decimal text is malformed"));
        }
        let fraction = fraction.trim_end_matches('0');
        let significant = format!("{whole}{fraction}");
        let significant = significant.trim_start_matches('0');
        if significant.len() > DECIMAL_DIGITS as usize || fraction.len() > DECIMAL_SCALE as usize {
            return Err(invalid(
                "decimal values have at most 38 digits and scale 18",
            ));
        }
        let magnitude: i128 = if significant.is_empty() {
            0
        } else {
            significant
                .parse()
                .map_err(|_| invalid("decimal text is malformed"))?
        };
        let coefficient = if negative { -magnitude } else { magnitude };
        Self::decimal_parts(coefficient, fraction.len() as u8)
    }

    /// `coefficient * 10^-scale`, normalized.
    pub fn decimal_parts(coefficient: i128, scale: u8) -> ExpressionResult<Self> {
        crate::expressions::operators::decimal::normalized(coefficient, scale)
            .map(|(coefficient, scale)| Self(Repr::Decimal { coefficient, scale }))
            .ok_or_else(|| invalid("decimal values have at most 38 digits and scale 18"))
    }

    pub fn string(value: impl Into<Arc<str>>) -> Self {
        Self(Repr::String(value.into()))
    }

    pub fn bytes(value: impl Into<Arc<[u8]>>) -> Self {
        Self(Repr::Bytes(value.into()))
    }

    /// An opaque nominal ID. It is never convertible to authority.
    pub fn id(value: impl Into<Arc<[u8]>>) -> Self {
        Self(Repr::Id(value.into()))
    }

    /// The enum variant at `index` in declaration order.
    pub fn variant(index: u32) -> Self {
        Self(Repr::Enum(index))
    }

    /// Little-endian limbs; bits at and above `width` must be zero.
    pub fn bits(width: u32, limbs: Vec<u64>) -> ExpressionResult<Self> {
        check_planes(width, &limbs)?;
        Ok(Self(Repr::Bits {
            width,
            limbs: limbs.into(),
        }))
    }

    /// Bitplanes `(value, unknown)`: 0=(0,0), 1=(1,0), X=(0,1), Z=(1,1).
    pub fn logic4(width: u32, value: Vec<u64>, unknown: Vec<u64>) -> ExpressionResult<Self> {
        check_planes(width, &value)?;
        check_planes(width, &unknown)?;
        Ok(Self(Repr::Logic4 {
            width,
            value: value.into(),
            unknown: unknown.into(),
        }))
    }

    pub fn none() -> Self {
        Self(Repr::None)
    }

    pub fn some(value: Self) -> Self {
        Self(Repr::Some(Arc::new(value)))
    }

    pub fn list(items: Vec<Self>) -> Self {
        Self(Repr::List(Composite::new(items)))
    }

    /// Field values in schema order.
    pub fn record(fields: Vec<Self>) -> Self {
        Self(Repr::Record(Composite::new(fields)))
    }

    /// Sorts entries into canonical key order; duplicate keys deny.
    pub fn map(mut entries: Vec<(Self, Self)>) -> ExpressionResult<Self> {
        entries.sort_by(|(a, _), (b, _)| key_order(a, b));
        if entries
            .windows(2)
            .any(|pair| key_order(&pair[0].0, &pair[1].0) == Ordering::Equal)
        {
            return Err(invalid("map keys are unique"));
        }
        let items = entries.into_iter().flat_map(|(key, value)| [key, value]);
        Ok(Self(Repr::Map(Composite::new(items.collect()))))
    }

    pub(crate) fn entry(key: Self, value: Self) -> Self {
        Self(Repr::Entry(Composite::new(vec![key, value])))
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self.0 {
            Repr::Bool(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i128> {
        match self.0 {
            Repr::Integer(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self.0 {
            Repr::Float32(bits) => Some(f32::from_bits(bits)),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self.0 {
            Repr::Float64(bits) => Some(f64::from_bits(bits)),
            _ => None,
        }
    }

    /// The canonical SI magnitude of a quantity.
    pub fn as_quantity(&self) -> Option<f64> {
        match self.0 {
            Repr::Quantity(bits) => Some(f64::from_bits(bits)),
            _ => None,
        }
    }

    /// `(coefficient, scale)` of a normalized decimal.
    pub fn as_decimal(&self) -> Option<(i128, u8)> {
        match self.0 {
            Repr::Decimal { coefficient, scale } => Some((coefficient, scale)),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match &self.0 {
            Repr::String(text) => Some(text),
            _ => None,
        }
    }

    /// The bytes of a Bytes or ID value.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match &self.0 {
            Repr::Bytes(bytes) | Repr::Id(bytes) => Some(bytes),
            _ => None,
        }
    }

    pub fn as_variant(&self) -> Option<u32> {
        match self.0 {
            Repr::Enum(index) => Some(index),
            _ => None,
        }
    }

    /// `Some(None)` for an absent option, `Some(Some(value))` for a present one.
    pub fn as_option(&self) -> Option<Option<&Self>> {
        match &self.0 {
            Repr::None => Some(None),
            Repr::Some(inner) => Some(Some(inner)),
            _ => None,
        }
    }

    /// List elements, record fields, or an entry's key and value.
    pub fn items(&self) -> Option<&[Self]> {
        match &self.0 {
            Repr::List(parts) | Repr::Record(parts) | Repr::Entry(parts) => Some(&parts.items),
            _ => None,
        }
    }

    /// Map entries in canonical key order.
    pub fn map_entries(&self) -> Option<impl ExactSizeIterator<Item = (&Self, &Self)>> {
        match &self.0 {
            Repr::Map(parts) => Some(parts.items.chunks_exact(2).map(|pair| (&pair[0], &pair[1]))),
            _ => None,
        }
    }

    /// `(width, limbs)` of a Bits value.
    pub fn as_bits(&self) -> Option<(u32, &[u64])> {
        match &self.0 {
            Repr::Bits { width, limbs } => Some((*width, limbs)),
            _ => None,
        }
    }

    /// `(width, value plane, unknown plane)` of a Logic4 value.
    pub fn as_logic4(&self) -> Option<(u32, &[u64], &[u64])> {
        match &self.0 {
            Repr::Logic4 {
                width,
                value,
                unknown,
            } => Some((*width, value, unknown)),
            _ => None,
        }
    }

    /// The logical size charged for reading, retaining, or emitting a value.
    pub fn logical_bytes(&self) -> u64 {
        match &self.0 {
            Repr::Bool(_) | Repr::None => 1,
            Repr::Enum(_) => 4,
            Repr::Float32(_) => 4,
            Repr::Float64(_) | Repr::Quantity(_) => 8,
            Repr::Integer(_) => 16,
            Repr::Decimal { .. } => 17,
            Repr::String(text) => text.len() as u64,
            Repr::Bytes(bytes) | Repr::Id(bytes) => bytes.len() as u64,
            Repr::Bits { limbs, .. } => limbs.len() as u64 * 8,
            Repr::Logic4 { value, .. } => value.len() as u64 * 16,
            Repr::Some(inner) => inner.logical_bytes().saturating_add(1),
            Repr::List(parts) | Repr::Map(parts) | Repr::Record(parts) | Repr::Entry(parts) => {
                parts.bytes
            }
        }
    }
}

/// Planes hold exactly `ceil(width / 64)` limbs with no bits past `width`.
fn check_planes(width: u32, limbs: &[u64]) -> ExpressionResult<()> {
    let expected = (width as usize).div_ceil(64);
    let clean = match (width % 64, limbs.last()) {
        (0, _) | (_, None) => true,
        (used, Some(top)) => top >> used == 0,
    };
    if width == 0 || limbs.len() != expected || !clean {
        Err(invalid("bus planes must match the declared width"))
    } else {
        Ok(())
    }
}

/// Canonical map key order: numeric for integers, unsigned bytewise for
/// String and ID keys.
pub(crate) fn key_order(a: &ExpressionValue, b: &ExpressionValue) -> Ordering {
    match (&a.0, &b.0) {
        (Repr::Integer(a), Repr::Integer(b)) => a.cmp(b),
        (Repr::String(a), Repr::String(b)) => a.as_bytes().cmp(b.as_bytes()),
        (Repr::Id(a), Repr::Id(b)) => a.cmp(b),
        _ => Ordering::Equal,
    }
}
