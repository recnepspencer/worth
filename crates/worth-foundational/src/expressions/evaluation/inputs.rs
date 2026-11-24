//! Sealed, immutable operand snapshots.
//!
//! Inputs bind values to a schema's declared operands before evaluation
//! starts. Evaluation reads only what was bound: there is no callback that
//! could do arbitrary work or I/O, and a retained continuation keeps the
//! shared snapshot alive without copying it.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::expressions::denial::{ExpressionDenial, ExpressionDenialDetail, ExpressionResult};
use crate::expressions::types::{ExpressionSchema, ExpressionType, IntegerType};

use super::value::{ExpressionValue, Repr};

/// Operand values bound against one schema.
#[derive(Debug, Clone)]
pub struct ExpressionInputs {
    schema: ExpressionSchema,
    values: Arc<BTreeMap<Box<str>, ExpressionValue>>,
}

impl ExpressionInputs {
    pub fn builder(schema: &ExpressionSchema) -> ExpressionInputsBuilder {
        ExpressionInputsBuilder {
            schema: schema.clone(),
            values: BTreeMap::new(),
        }
    }

    pub fn schema(&self) -> &ExpressionSchema {
        &self.schema
    }

    /// The value bound to operand `name`, if any.
    pub fn value(&self, name: &str) -> Option<&ExpressionValue> {
        self.values.get(name)
    }

    /// Heap bytes owned by `ExpressionInputs::builder(schema)` (cloning
    /// `schema`'s operand table) plus the peak transient `conform` worklist
    /// for scalar and optional-scalar schemas.
    ///
    /// Returns `None` if any operand has a composite type (`List`, `Map`,
    /// `MapEntry`, `Record`, or nested `Option`) whose `conform` worklist
    /// depends on runtime value shape, or on arithmetic overflow.
    pub fn scalar_builder_schema_and_conform_bytes(schema: &ExpressionSchema) -> Option<usize> {
        let mut schema_bytes = schema
            .operands()
            .len()
            .checked_mul(size_of::<(Box<str>, ExpressionType)>())?;
        let mut peak_conform = 0_usize;
        for (name, ty) in schema.operands() {
            let conform_bytes = scalar_conform_peak_bytes(ty)?;
            let ty_bytes = ty.clone_allocation_bytes()?;
            schema_bytes = schema_bytes
                .checked_add(name.len())?
                .checked_add(ty_bytes)?;
            peak_conform = peak_conform.max(conform_bytes);
        }
        schema_bytes.checked_add(peak_conform)
    }
}

fn is_scalar_type(ty: &ExpressionType) -> bool {
    matches!(
        ty,
        ExpressionType::Bool
            | ExpressionType::Integer(_)
            | ExpressionType::Float32
            | ExpressionType::Float64
            | ExpressionType::Decimal
            | ExpressionType::String
            | ExpressionType::Bytes
            | ExpressionType::Id(_)
            | ExpressionType::Enum(_)
            | ExpressionType::Quantity(_)
            | ExpressionType::Bits(_)
            | ExpressionType::Logic4(_)
            | ExpressionType::Date
    )
}

fn scalar_conform_peak_bytes(ty: &ExpressionType) -> Option<usize> {
    let slot = size_of::<(&ExpressionValue, ExpressionType)>();
    if is_scalar_type(ty) {
        return slot.checked_add(ty.clone_allocation_bytes()?);
    }
    match ty {
        ExpressionType::Option(inner) if is_scalar_type(inner) => slot
            .checked_add(ty.clone_allocation_bytes()?)?
            .checked_add(inner.clone_allocation_bytes()?),
        _ => None,
    }
}

/// Binds operand values one at a time, checking each against its declared
/// type.
#[derive(Debug)]
pub struct ExpressionInputsBuilder {
    schema: ExpressionSchema,
    values: BTreeMap<Box<str>, ExpressionValue>,
}

impl ExpressionInputsBuilder {
    /// Binds `value` to operand `name`. Unknown and repeated operands deny,
    /// as does a value that does not conform to the declared type.
    pub fn bind(mut self, name: &str, value: ExpressionValue) -> ExpressionResult<Self> {
        let Some(ty) = self.schema.operand(name) else {
            return Err(ExpressionDenial::new(
                ExpressionDenialDetail::UnknownBinding(name.to_string()),
            ));
        };
        if self.values.contains_key(name) {
            return Err(ExpressionDenial::new(
                ExpressionDenialDetail::AmbiguousBinding(name.to_string()),
            ));
        }
        conform(&self.schema, &value, ty)?;
        self.values.insert(name.into(), value);
        Ok(self)
    }

    /// Operands left unbound deny with `MissingOperand` only if evaluation
    /// reads them.
    pub fn build(self) -> ExpressionInputs {
        ExpressionInputs {
            schema: self.schema,
            values: Arc::new(self.values),
        }
    }
}

/// The value's shape, for mismatch details.
fn shape(value: &ExpressionValue) -> &'static str {
    match &value.0 {
        Repr::Bool(_) => "Bool",
        Repr::Integer(_) => "integer",
        Repr::Float32(_) => "Float32",
        Repr::Float64(_) => "Float64",
        Repr::Decimal { .. } => "Decimal",
        Repr::String(_) => "String",
        Repr::Bytes(_) => "Bytes",
        Repr::Id(_) => "ID",
        Repr::Enum(_) => "enum variant",
        Repr::Quantity(_) => "quantity",
        Repr::Bits { .. } => "Bits",
        Repr::Logic4 { .. } => "Logic4",
        Repr::None | Repr::Some(_) => "option",
        Repr::List(_) => "list",
        Repr::Map(_) => "map",
        Repr::Record(_) => "record",
        Repr::Entry(_) => "map entry",
        Repr::Date(_) => "Date",
    }
}

/// Checks `value` against `ty` with an explicit worklist, so nesting depth
/// never becomes native recursion.
fn conform(
    schema: &ExpressionSchema,
    value: &ExpressionValue,
    ty: &ExpressionType,
) -> ExpressionResult<()> {
    let mut pending = vec![(value, ty.clone())];
    while let Some((value, ty)) = pending.pop() {
        let fits = match (&value.0, &ty) {
            (Repr::Bool(_), ExpressionType::Bool)
            | (Repr::Float32(_), ExpressionType::Float32)
            | (Repr::Float64(_), ExpressionType::Float64)
            | (Repr::Decimal { .. }, ExpressionType::Decimal)
            | (Repr::String(_), ExpressionType::String)
            | (Repr::Bytes(_), ExpressionType::Bytes)
            | (Repr::Id(_), ExpressionType::Id(_))
            | (Repr::Quantity(_), ExpressionType::Quantity(_))
            | (Repr::Date(_), ExpressionType::Date)
            | (Repr::None, ExpressionType::Option(_)) => true,
            (Repr::Integer(integer), ExpressionType::Integer(width)) => {
                if !(IntegerType::min(*width)..=IntegerType::max(*width)).contains(integer) {
                    return Err(ExpressionDenial::new(ExpressionDenialDetail::InvalidValue(
                        "an integer operand exceeds its declared width",
                    )));
                }
                true
            }
            (Repr::Bits { width, .. }, ExpressionType::Bits(declared))
            | (Repr::Logic4 { width, .. }, ExpressionType::Logic4(declared)) => width == declared,
            (Repr::Enum(index), ExpressionType::Enum(name)) => {
                schema.enumeration(name.name()).is_some_and(|declaration| {
                    declaration.name() == name && (*index as usize) < declaration.variants().len()
                })
            }
            (Repr::Some(inner), ExpressionType::Option(inner_ty)) => {
                pending.push((inner, (**inner_ty).clone()));
                true
            }
            (Repr::List(parts), ExpressionType::List(element)) => {
                pending.extend(parts.items.iter().map(|item| (item, (**element).clone())));
                true
            }
            (Repr::Map(parts), ExpressionType::Map(key, entry))
            | (Repr::Entry(parts), ExpressionType::MapEntry(key, entry)) => {
                for pair in parts.items.as_chunks::<2>().0 {
                    pending.push((&pair[0], (**key).clone()));
                    pending.push((&pair[1], (**entry).clone()));
                }
                true
            }
            (Repr::Record(parts), ExpressionType::Record(name)) => {
                match schema.record(name.name()) {
                    Some(declaration)
                        if declaration.name() == name
                            && declaration.fields().len() == parts.items.len() =>
                    {
                        let fields = declaration.fields().map(|(_, field)| field.clone());
                        pending.extend(parts.items.iter().zip(fields));
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        };
        if !fits {
            return Err(ExpressionDenial::new(
                ExpressionDenialDetail::TypeMismatch {
                    expected: ty.to_string(),
                    found: shape(value).to_string(),
                },
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expressions::types::ExpressionTypeName;

    #[test]
    fn scalar_quote_bounds_scalar_and_optional_scalar_schemas_and_rejects_composites() {
        let id_name = ExpressionTypeName::new("work::Task", 1).unwrap();
        let scalar_schema = ExpressionSchema::builder()
            .identifier("work::Task", 1)
            .unwrap()
            .operand("flag", ExpressionType::Bool)
            .unwrap()
            .operand("note", ExpressionType::option(ExpressionType::String))
            .unwrap()
            .operand("task", ExpressionType::option(ExpressionType::Id(id_name)))
            .unwrap()
            .build();
        let expected_schema = 3 * size_of::<(Box<str>, ExpressionType)>()
            + "flag".len()
            + "note".len()
            + size_of::<ExpressionType>()
            + "task".len()
            + size_of::<ExpressionType>()
            + "work::Task".len();
        let expected_peak_conform = size_of::<(&ExpressionValue, ExpressionType)>()
            + size_of::<ExpressionType>()
            + 2 * "work::Task".len();
        assert_eq!(
            ExpressionInputs::scalar_builder_schema_and_conform_bytes(&scalar_schema),
            Some(expected_schema + expected_peak_conform)
        );

        let list_schema = ExpressionSchema::builder()
            .operand("items", ExpressionType::list(ExpressionType::Bool))
            .unwrap()
            .build();
        assert_eq!(
            ExpressionInputs::scalar_builder_schema_and_conform_bytes(&list_schema),
            None
        );

        let map_schema = ExpressionSchema::builder()
            .operand(
                "attrs",
                ExpressionType::map(ExpressionType::String, ExpressionType::Bool),
            )
            .unwrap()
            .build();
        assert_eq!(
            ExpressionInputs::scalar_builder_schema_and_conform_bytes(&map_schema),
            None
        );

        let rec_name = ExpressionTypeName::new("work::Meta", 1).unwrap();
        let record_schema = ExpressionSchema::builder()
            .record("work::Meta", 1, [("ok", ExpressionType::Bool)])
            .unwrap()
            .operand("meta", ExpressionType::Record(rec_name))
            .unwrap()
            .build();
        assert_eq!(
            ExpressionInputs::scalar_builder_schema_and_conform_bytes(&record_schema),
            None
        );

        let nested_opt_schema = ExpressionSchema::builder()
            .operand(
                "nested",
                ExpressionType::option(ExpressionType::option(ExpressionType::Bool)),
            )
            .unwrap()
            .build();
        assert_eq!(
            ExpressionInputs::scalar_builder_schema_and_conform_bytes(&nested_opt_schema),
            None
        );

        let opt_list_schema = ExpressionSchema::builder()
            .operand(
                "maybe_items",
                ExpressionType::option(ExpressionType::list(ExpressionType::Bool)),
            )
            .unwrap()
            .build();
        assert_eq!(
            ExpressionInputs::scalar_builder_schema_and_conform_bytes(&opt_list_schema),
            None
        );
    }
}
