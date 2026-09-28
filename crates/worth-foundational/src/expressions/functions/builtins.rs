//! The closed V1 builtin vocabulary. Signatures live in admission; semantics
//! live in the evaluator. Adding a builtin is a new language version.

use crate::expressions::types::units::UnitScale;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Builtin {
    Min,
    Max,
    /// `min(list)`: `None` for an empty list.
    MinOf,
    /// `max(list)`: `None` for an empty list.
    MaxOf,
    Abs,
    Clamp,
    Sqrt,
    Near,
    Sum,
    DecimalDiv,
    Quantize,
    /// `quantity(Float64, unit)`: scale to canonical SI, rounding once.
    Quantity(UnitScale),
    /// `magnitude(quantity, unit)`: the magnitude in `unit`, rounding once.
    Magnitude(UnitScale),
    IsSome,
    Some,
    Unwrap,
    Length,
    Get,
    Contains,
    Entries,
    StartsWith,
    EndsWith,
    /// `slice(string, low, high)`: scalar offsets, `None` for invalid bounds.
    Substring,
    ExactCast,
    RoundedCast,
    Truncate,
    Extend,
    ToBits,
    /// `slice<low, high>(bits)`: half-open, statically sized.
    BitSlice {
        low: u32,
    },
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    ShiftLeft,
    ShiftRight,
    Concat,
    BitsAdd,
    WrappingAdd,
    LogicEq,
    /// `case_equal(a, b)`: exact symbol equality of `Logic4` buses as Bool.
    CaseEqual,
    Mux,
}

/// Every single-segment callable name, including literal constructors.
///
/// Let binders and installed functions cannot use these names.
pub(crate) const RESERVED_CALLABLES: [&str; 52] = [
    "min",
    "max",
    "abs",
    "clamp",
    "sqrt",
    "near",
    "sum",
    "decimal_div",
    "quantize",
    "quantity",
    "magnitude",
    "is_some",
    "some",
    "unwrap",
    "length",
    "get",
    "contains",
    "entries",
    "starts_with",
    "ends_with",
    "slice",
    "exact_cast",
    "rounded_cast",
    "truncate",
    "extend",
    "to_bits",
    "bit_and",
    "bit_or",
    "bit_xor",
    "bit_not",
    "shift_left",
    "shift_right",
    "concat",
    "bits_add",
    "wrapping_add",
    "logic_eq",
    "case_equal",
    "mux",
    "int8",
    "int16",
    "int32",
    "int64",
    "uint8",
    "uint16",
    "uint32",
    "uint64",
    "float32",
    "float64",
    "decimal",
    "bytes",
    "bits",
    "logic4",
];

pub(crate) fn is_reserved_callable(name: &str) -> bool {
    RESERVED_CALLABLES.contains(&name)
}

impl Builtin {
    /// The stable identity tag of this builtin, including static parameters.
    pub(crate) fn tag(self) -> String {
        let name = match self {
            Self::Min => "min",
            Self::Max => "max",
            Self::MinOf => "min_of",
            Self::MaxOf => "max_of",
            Self::Abs => "abs",
            Self::Clamp => "clamp",
            Self::Sqrt => "sqrt",
            Self::Near => "near",
            Self::Sum => "sum",
            Self::DecimalDiv => "decimal_div",
            Self::Quantize => "quantize",
            Self::Quantity(scale) => return format!("quantity:{}", scale_tag(scale)),
            Self::Magnitude(scale) => return format!("magnitude:{}", scale_tag(scale)),
            Self::IsSome => "is_some",
            Self::Some => "some",
            Self::Unwrap => "unwrap",
            Self::Length => "length",
            Self::Get => "get",
            Self::Contains => "contains",
            Self::Entries => "entries",
            Self::StartsWith => "starts_with",
            Self::EndsWith => "ends_with",
            Self::Substring => "substring",
            Self::ExactCast => "exact_cast",
            Self::RoundedCast => "rounded_cast",
            Self::Truncate => "truncate",
            Self::Extend => "extend",
            Self::ToBits => "to_bits",
            Self::BitSlice { low } => return format!("bit_slice:{low}"),
            Self::BitAnd => "bit_and",
            Self::BitOr => "bit_or",
            Self::BitXor => "bit_xor",
            Self::BitNot => "bit_not",
            Self::ShiftLeft => "shift_left",
            Self::ShiftRight => "shift_right",
            Self::Concat => "concat",
            Self::BitsAdd => "bits_add",
            Self::WrappingAdd => "wrapping_add",
            Self::LogicEq => "logic_eq",
            Self::CaseEqual => "case_equal",
            Self::Mux => "mux",
        };
        name.to_string()
    }
}

fn scale_tag(scale: UnitScale) -> String {
    format!(
        "{}/{}*deg^{}",
        scale.numerator, scale.denominator, scale.degree_power
    )
}
