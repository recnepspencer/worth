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
                for pair in parts.items.chunks_exact(2) {
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
