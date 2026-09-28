//! The closed V1 expression type vocabulary.

mod dimension;
mod name;
mod resolve;
mod schema;
pub(crate) mod units;

use std::fmt;

pub use dimension::{BaseDimension, ExpressionDimension};
pub(crate) use name::is_identifier;
pub use name::ExpressionTypeName;
pub(crate) use resolve::resolve_type;
pub(crate) use schema::{rounding_type, ROUNDING_NAME, ROUNDING_VARIANTS};
pub use schema::{
    ExpressionEnumDeclaration, ExpressionRecordDeclaration, ExpressionSchema,
    ExpressionSchemaBuilder,
};

/// A declared-width integer type. Arithmetic is checked; nothing wraps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntegerType {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
}

impl IntegerType {
    pub(crate) const ALL: [Self; 8] = [
        Self::Int8,
        Self::Int16,
        Self::Int32,
        Self::Int64,
        Self::UInt8,
        Self::UInt16,
        Self::UInt32,
        Self::UInt64,
    ];

    pub const fn is_signed(self) -> bool {
        matches!(self, Self::Int8 | Self::Int16 | Self::Int32 | Self::Int64)
    }

    pub const fn bits(self) -> u32 {
        match self {
            Self::Int8 | Self::UInt8 => 8,
            Self::Int16 | Self::UInt16 => 16,
            Self::Int32 | Self::UInt32 => 32,
            Self::Int64 | Self::UInt64 => 64,
        }
    }

    pub const fn min(self) -> i128 {
        if self.is_signed() {
            -(1_i128 << (self.bits() - 1))
        } else {
            0
        }
    }

    pub const fn max(self) -> i128 {
        if self.is_signed() {
            (1_i128 << (self.bits() - 1)) - 1
        } else {
            (1_i128 << self.bits()) - 1
        }
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Int8 => "Int8",
            Self::Int16 => "Int16",
            Self::Int32 => "Int32",
            Self::Int64 => "Int64",
            Self::UInt8 => "UInt8",
            Self::UInt16 => "UInt16",
            Self::UInt32 => "UInt32",
            Self::UInt64 => "UInt64",
        }
    }
}

/// A V1 expression type.
///
/// Nominal types name an exact schema declaration version. `MapEntry` exists
/// only as the element type `entries(map)` produces; schemas cannot declare it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExpressionType {
    Bool,
    Integer(IntegerType),
    Float32,
    Float64,
    Decimal,
    String,
    Bytes,
    Id(ExpressionTypeName),
    Enum(ExpressionTypeName),
    Record(ExpressionTypeName),
    Option(Box<ExpressionType>),
    List(Box<ExpressionType>),
    Map(Box<ExpressionType>, Box<ExpressionType>),
    MapEntry(Box<ExpressionType>, Box<ExpressionType>),
    Quantity(ExpressionDimension),
    Bits(u32),
    Logic4(u32),
}

impl ExpressionType {
    pub const INT64: Self = Self::Integer(IntegerType::Int64);

    pub fn option(inner: Self) -> Self {
        Self::Option(Box::new(inner))
    }

    pub fn list(element: Self) -> Self {
        Self::List(Box::new(element))
    }

    pub fn map(key: Self, value: Self) -> Self {
        Self::Map(Box::new(key), Box::new(value))
    }

    pub(crate) fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::Integer(_) | Self::Float32 | Self::Float64 | Self::Decimal | Self::Quantity(_)
        )
    }

    pub(crate) fn is_float(&self) -> bool {
        matches!(self, Self::Float32 | Self::Float64)
    }

    /// Types with a language ordering: numeric scalars, quantities, String,
    /// and Bytes.
    pub(crate) fn is_ordered(&self) -> bool {
        self.is_numeric() || matches!(self, Self::String | Self::Bytes)
    }

    /// Map keys have a canonical ordering: String, integers, and nominal IDs.
    pub(crate) fn is_map_key(&self) -> bool {
        matches!(self, Self::String | Self::Integer(_) | Self::Id(_))
    }
}

impl fmt::Display for ExpressionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool => f.write_str("Bool"),
            Self::Integer(integer) => f.write_str(integer.name()),
            Self::Float32 => f.write_str("Float32"),
            Self::Float64 => f.write_str("Float64"),
            Self::Decimal => f.write_str("Decimal"),
            Self::String => f.write_str("String"),
            Self::Bytes => f.write_str("Bytes"),
            Self::Id(name) => write!(f, "Id<{name}>"),
            Self::Enum(name) => write!(f, "Enum<{name}>"),
            Self::Record(name) => write!(f, "Record<{name}>"),
            Self::Option(inner) => write!(f, "Option<{inner}>"),
            Self::List(element) => write!(f, "List<{element}>"),
            Self::Map(key, value) => write!(f, "Map<{key}, {value}>"),
            Self::MapEntry(key, value) => write!(f, "MapEntry<{key}, {value}>"),
            Self::Quantity(dimension) => write!(f, "Quantity<{dimension}>"),
            Self::Bits(width) => write!(f, "Bits<{width}>"),
            Self::Logic4(width) => write!(f, "Logic4<{width}>"),
        }
    }
}
