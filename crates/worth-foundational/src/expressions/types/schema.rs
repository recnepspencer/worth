//! Closed operand schemas: nominal declarations plus typed operands.

use std::collections::BTreeMap;

use crate::expressions::denial::{ExpressionDenial, ExpressionDenialDetail, SyntaxDenial};
use crate::expressions::syntax::GENERIC_INTRINSICS;

use super::name::{check_qualified_name, is_identifier};
use super::{ExpressionType, ExpressionTypeName};

/// Nested type structure is bounded like syntax, so checks never recurse deep.
const TYPE_DEPTH: u32 = 64;
const BIT_WIDTH: u32 = 4096;

/// Built-in type names that nominal declarations cannot reuse.
const BUILTIN_TYPE_NAMES: [&str; 20] = [
    "Bool", "Int8", "Int16", "Int32", "Int64", "UInt8", "UInt16", "UInt32", "UInt64", "Float32",
    "Float64", "Decimal", "String", "Bytes", "Option", "List", "Map", "Quantity", "Bits", "Logic4",
];

pub(crate) const ROUNDING_NAME: &str = "Rounding";
pub(crate) const ROUNDING_VARIANTS: [&str; 4] = [
    "NearestEven",
    "TowardZero",
    "TowardPositive",
    "TowardNegative",
];

/// A versioned record declaration. Field order is schema order.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionRecordDeclaration {
    name: ExpressionTypeName,
    fields: Vec<(Box<str>, ExpressionType)>,
    /// Field positions by name, so lookups cost a logarithmic probe.
    positions: BTreeMap<Box<str>, usize>,
    /// The widest bus in any field, through nested records.
    widest_bus: u32,
}

impl ExpressionRecordDeclaration {
    pub fn name(&self) -> &ExpressionTypeName {
        &self.name
    }

    pub fn fields(&self) -> impl ExactSizeIterator<Item = (&str, &ExpressionType)> {
        self.fields.iter().map(|(name, ty)| (&**name, ty))
    }

    pub(crate) fn field(&self, name: &str) -> Option<(usize, &ExpressionType)> {
        let index = *self.positions.get(name)?;
        Some((index, &self.fields[index].1))
    }
}

/// A versioned enum declaration. Variant order is schema order.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionEnumDeclaration {
    name: ExpressionTypeName,
    variants: Vec<Box<str>>,
    /// Variant positions by name, so lookups cost a logarithmic probe.
    positions: BTreeMap<Box<str>, usize>,
}

impl ExpressionEnumDeclaration {
    pub fn name(&self) -> &ExpressionTypeName {
        &self.name
    }

    pub fn variants(&self) -> impl ExactSizeIterator<Item = &str> {
        self.variants.iter().map(|variant| &**variant)
    }

    pub(crate) fn variant(&self, name: &str) -> Option<usize> {
        self.positions.get(name).copied()
    }
}

/// A closed operand schema.
///
/// Expressions resolve operands, records, enums, and identifiers only against
/// this schema. Records may reference only records declared before them, so
/// record types are acyclic by construction.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExpressionSchema {
    records: BTreeMap<Box<str>, ExpressionRecordDeclaration>,
    enums: BTreeMap<Box<str>, ExpressionEnumDeclaration>,
    ids: BTreeMap<Box<str>, ExpressionTypeName>,
    /// Operands sorted by name; a slot is a position in this order.
    operands: Vec<(Box<str>, ExpressionType)>,
}

impl ExpressionSchema {
    pub fn builder() -> ExpressionSchemaBuilder {
        ExpressionSchemaBuilder {
            schema: Self::default(),
            operands: BTreeMap::new(),
        }
    }

    pub fn record(&self, name: &str) -> Option<&ExpressionRecordDeclaration> {
        self.records.get(name)
    }

    pub fn enumeration(&self, name: &str) -> Option<&ExpressionEnumDeclaration> {
        self.enums.get(name)
    }

    pub fn identifier(&self, name: &str) -> Option<&ExpressionTypeName> {
        self.ids.get(name)
    }

    /// Declared operands in canonical name order.
    pub fn operands(&self) -> impl ExactSizeIterator<Item = (&str, &ExpressionType)> {
        self.operands.iter().map(|(name, ty)| (&**name, ty))
    }

    pub fn operand(&self, name: &str) -> Option<&ExpressionType> {
        self.operand_index(name)
            .map(|index| &self.operands[index].1)
    }

    /// The position of `name` in canonical operand order, by binary search.
    pub(crate) fn operand_index(&self, name: &str) -> Option<usize> {
        self.operands
            .binary_search_by(|(operand, _)| (**operand).cmp(name))
            .ok()
    }

    pub(crate) fn operand_at(&self, index: usize) -> Option<(&str, &ExpressionType)> {
        self.operands.get(index).map(|(name, ty)| (&**name, ty))
    }

    /// This schema's type declarations without its operands: the vocabulary
    /// installed function signatures and bodies resolve against.
    pub(crate) fn declarations_only(&self) -> Self {
        Self {
            operands: Vec::new(),
            ..self.clone()
        }
    }

    /// Whether both schemas declare exactly the same records, enums, and IDs.
    pub(crate) fn same_declarations(&self, other: &Self) -> bool {
        self.records == other.records && self.enums == other.enums && self.ids == other.ids
    }

    /// The widest `Bits` or `Logic4` width `ty` carries anywhere inside it,
    /// record fields included; zero without buses.
    pub(crate) fn widest_bus(&self, ty: &ExpressionType) -> u32 {
        match ty {
            ExpressionType::Bits(width) | ExpressionType::Logic4(width) => *width,
            ExpressionType::Option(inner) | ExpressionType::List(inner) => self.widest_bus(inner),
            ExpressionType::Map(key, value) | ExpressionType::MapEntry(key, value) => {
                self.widest_bus(key).max(self.widest_bus(value))
            }
            ExpressionType::Record(name) => self
                .records
                .get(name.name())
                .map_or(0, |record| record.widest_bus),
            _ => 0,
        }
    }

    /// Declared records, enums, and IDs: the size of the nominal lookup table.
    pub(crate) fn nominal_count(&self) -> usize {
        self.records.len() + self.enums.len() + self.ids.len()
    }

    /// The nominal kind declared under `name`, or the built-in `Rounding` enum.
    pub(crate) fn nominal(&self, name: &str) -> Option<ExpressionType> {
        if let Some(record) = self.records.get(name) {
            return Some(ExpressionType::Record(record.name.clone()));
        }
        if let Some(enumeration) = self.enums.get(name) {
            return Some(ExpressionType::Enum(enumeration.name.clone()));
        }
        if let Some(id) = self.ids.get(name) {
            return Some(ExpressionType::Id(id.clone()));
        }
        (name == ROUNDING_NAME).then(rounding_type)
    }

    /// Denies a type that references undeclared nominals, uses an invalid
    /// width or map key, names `MapEntry`, or nests past the depth ceiling.
    pub(crate) fn check_type(&self, ty: &ExpressionType) -> Result<(), ExpressionDenial> {
        self.check_type_at(ty, 0)
    }

    fn check_type_at(&self, ty: &ExpressionType, depth: u32) -> Result<(), ExpressionDenial> {
        let invalid = |reason| {
            Err(ExpressionDenial::new(ExpressionDenialDetail::InvalidValue(
                reason,
            )))
        };
        if depth >= TYPE_DEPTH {
            return invalid("type nesting exceeds the depth ceiling");
        }
        match ty {
            ExpressionType::Id(name)
            | ExpressionType::Enum(name)
            | ExpressionType::Record(name) => {
                if self.nominal(name.name()).as_ref() == Some(ty) {
                    Ok(())
                } else {
                    Err(ExpressionDenial::new(
                        ExpressionDenialDetail::UnknownBinding(name.to_string()),
                    ))
                }
            }
            ExpressionType::Option(inner) | ExpressionType::List(inner) => {
                self.check_type_at(inner, depth + 1)
            }
            ExpressionType::Map(key, value) => {
                if !key.is_map_key() {
                    return invalid("map keys are String, integer, or nominal ID types");
                }
                self.check_type_at(key, depth + 1)?;
                self.check_type_at(value, depth + 1)
            }
            ExpressionType::MapEntry(..) => invalid("MapEntry is produced only by entries()"),
            ExpressionType::Bits(width) | ExpressionType::Logic4(width) => {
                if (1..=BIT_WIDTH).contains(width) {
                    Ok(())
                } else {
                    invalid("bit widths are 1 through 4096")
                }
            }
            _ => Ok(()),
        }
    }
}

pub(crate) fn rounding_type() -> ExpressionType {
    ExpressionType::Enum(ExpressionTypeName::new(ROUNDING_NAME, 1).expect("valid builtin name"))
}

/// Builds an [`ExpressionSchema`], checking each declaration as it is added.
#[derive(Debug, Clone)]
pub struct ExpressionSchemaBuilder {
    schema: ExpressionSchema,
    operands: BTreeMap<Box<str>, ExpressionType>,
}

impl ExpressionSchemaBuilder {
    /// Declares a record. Field types may reference only earlier declarations.
    pub fn record<'a>(
        mut self,
        name: &str,
        version: u32,
        fields: impl IntoIterator<Item = (&'a str, ExpressionType)>,
    ) -> Result<Self, ExpressionDenial> {
        let name = self.nominal_name(name, version)?;
        let mut declared: Vec<(Box<str>, ExpressionType)> = Vec::new();
        let mut positions = BTreeMap::new();
        for (field, ty) in fields {
            check_member(field, positions.contains_key(field))?;
            self.schema.check_type(&ty)?;
            positions.insert(field.into(), declared.len());
            declared.push((field.into(), ty));
        }
        let key: Box<str> = name.name().into();
        let widest_bus = declared
            .iter()
            .map(|(_, ty)| self.schema.widest_bus(ty))
            .max()
            .unwrap_or(0);
        self.schema.records.insert(
            key,
            ExpressionRecordDeclaration {
                name,
                fields: declared,
                positions,
                widest_bus,
            },
        );
        Ok(self)
    }

    /// Declares an enum with at least one variant.
    pub fn enumeration<'a>(
        mut self,
        name: &str,
        version: u32,
        variants: impl IntoIterator<Item = &'a str>,
    ) -> Result<Self, ExpressionDenial> {
        let name = self.nominal_name(name, version)?;
        let mut declared: Vec<Box<str>> = Vec::new();
        let mut positions = BTreeMap::new();
        for variant in variants {
            check_member(variant, positions.contains_key(variant))?;
            positions.insert(variant.into(), declared.len());
            declared.push(variant.into());
        }
        if declared.is_empty() {
            return Err(ExpressionDenial::new(ExpressionDenialDetail::InvalidValue(
                "an enum declares at least one variant",
            )));
        }
        let key: Box<str> = name.name().into();
        self.schema.enums.insert(
            key,
            ExpressionEnumDeclaration {
                name,
                variants: declared,
                positions,
            },
        );
        Ok(self)
    }

    /// Declares an opaque nominal identifier type.
    pub fn identifier(mut self, name: &str, version: u32) -> Result<Self, ExpressionDenial> {
        let name = self.nominal_name(name, version)?;
        self.schema.ids.insert(name.name().into(), name);
        Ok(self)
    }

    /// Declares a typed operand. Qualified operand names are allowed.
    pub fn operand(mut self, name: &str, ty: ExpressionType) -> Result<Self, ExpressionDenial> {
        check_qualified_name(name)?;
        if GENERIC_INTRINSICS.contains(&name) {
            return Err(ExpressionDenial::new(
                ExpressionDenialDetail::AmbiguousBinding(name.to_string()),
            ));
        }
        if self.operands.contains_key(name) {
            return Err(duplicate(name));
        }
        self.schema.check_type(&ty)?;
        self.operands.insert(name.into(), ty);
        Ok(self)
    }

    pub fn build(mut self) -> ExpressionSchema {
        self.schema.operands = self.operands.into_iter().collect();
        self.schema
    }

    fn nominal_name(
        &self,
        name: &str,
        version: u32,
    ) -> Result<ExpressionTypeName, ExpressionDenial> {
        let name = ExpressionTypeName::new(name, version)?;
        if BUILTIN_TYPE_NAMES.contains(&name.name()) || self.schema.nominal(name.name()).is_some() {
            return Err(duplicate(name.name()));
        }
        Ok(name)
    }
}

fn check_member(name: &str, exists: bool) -> Result<(), ExpressionDenial> {
    if !is_identifier(name) {
        return Err(ExpressionDenial::new(ExpressionDenialDetail::Syntax(
            SyntaxDenial::InvalidIdentifier,
        )));
    }
    if exists {
        return Err(duplicate(name));
    }
    Ok(())
}

fn duplicate(name: &str) -> ExpressionDenial {
    ExpressionDenial::new(ExpressionDenialDetail::AmbiguousBinding(name.to_string()))
}
