//! The closed builtin vocabulary. Signatures live in admission; semantics
//! live in the evaluator. Adding a builtin is a new language version, which
//! each builtin and reserved callable name records.

use crate::expressions::language_version::LanguageVersion;
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
    /// `contains_ignore_ascii_case(text, needle)`: V2.
    ContainsIgnoreAsciiCase,
    /// `starts_with_ignore_ascii_case(text, prefix)`: V2.
    StartsWithIgnoreAsciiCase,
    /// `ends_with_ignore_ascii_case(text, suffix)`: V2.
    EndsWithIgnoreAsciiCase,
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
pub(crate) const RESERVED_CALLABLES: [&str; 55] = [
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
    "contains_ignore_ascii_case",
    "starts_with_ignore_ascii_case",
    "ends_with_ignore_ascii_case",
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

/// The reserved callables language V2 introduced; every other one is V1.
const V2_CALLABLES: [&str; 3] = [
    "contains_ignore_ascii_case",
    "starts_with_ignore_ascii_case",
    "ends_with_ignore_ascii_case",
];

pub(crate) fn is_reserved_callable(name: &str) -> bool {
    RESERVED_CALLABLES.contains(&name)
}

/// The language version a single-segment callable name requires. Installed
/// functions cannot take reserved names, so only builtins raise it.
pub(crate) fn callable_language(name: &str) -> LanguageVersion {
    if V2_CALLABLES.contains(&name) {
        LanguageVersion::V2
    } else {
        LanguageVersion::V1
    }
}

impl Builtin {
    /// The language version that introduced this builtin.
    pub(crate) const fn language(self) -> LanguageVersion {
        match self {
            Self::ContainsIgnoreAsciiCase
            | Self::StartsWithIgnoreAsciiCase
            | Self::EndsWithIgnoreAsciiCase => LanguageVersion::V2,
            _ => LanguageVersion::V1,
        }
    }

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
            Self::ContainsIgnoreAsciiCase => "contains_ignore_ascii_case",
            Self::StartsWithIgnoreAsciiCase => "starts_with_ignore_ascii_case",
            Self::EndsWithIgnoreAsciiCase => "ends_with_ignore_ascii_case",
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
