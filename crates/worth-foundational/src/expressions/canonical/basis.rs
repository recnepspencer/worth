//! Stable identity tags for program operations, literal values, and the
//! nominal declarations a program references. Tags are spelled explicitly;
//! Debug text never enters identity.

use std::collections::BTreeMap;

use crate::canonicalization::{CanonicalBasisValue, CanonicalFloatWidth, CanonicalIntegerWidth};
use crate::expressions::program::{Literal, Op};
use crate::expressions::syntax::ast::{BinaryOp, ComprehensionKind, UnaryOp};
use crate::expressions::types::{
    ExpressionSchema, ExpressionType, ROUNDING_NAME, ROUNDING_VARIANTS,
};

use super::identity::hex;

pub(super) fn op_tag(op: &Op) -> String {
    match op {
        Op::Literal(_) => "literal".to_string(),
        Op::Operand(slot) => format!("operand:{slot}"),
        Op::Local(index) => format!("local:{index}"),
        Op::Field(index) => format!("field:{index}"),
        Op::Unary(op) => format!("unary:{}", unary_tag(*op)),
        Op::Binary(op) => format!("binary:{}", binary_tag(*op)),
        Op::Conditional => "conditional".to_string(),
        Op::Let => "let".to_string(),
        Op::Comprehension(kind) => format!("comprehension:{}", comprehension_tag(*kind)),
        Op::Builtin(builtin) => format!("builtin:{}", builtin.tag()),
        Op::Call(index) => format!("call:{index}"),
        Op::List => "list".to_string(),
        Op::Map => "map".to_string(),
        Op::Record => "record".to_string(),
    }
}

fn unary_tag(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Not => "not",
        UnaryOp::Negate => "negate",
    }
}

fn binary_tag(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Multiply => "multiply",
        BinaryOp::Divide => "divide",
        BinaryOp::Remainder => "remainder",
        BinaryOp::Add => "add",
        BinaryOp::Subtract => "subtract",
        BinaryOp::Less => "less",
        BinaryOp::LessEqual => "less_equal",
        BinaryOp::Greater => "greater",
        BinaryOp::GreaterEqual => "greater_equal",
        BinaryOp::Equal => "equal",
        BinaryOp::NotEqual => "not_equal",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
        BinaryOp::Coalesce => "coalesce",
    }
}

fn comprehension_tag(kind: ComprehensionKind) -> &'static str {
    match kind {
        ComprehensionKind::Map => "map",
        ComprehensionKind::Filter => "filter",
        ComprehensionKind::All => "all",
        ComprehensionKind::Any => "any",
    }
}

/// The exact value of a literal node; its node type disambiguates encodings.
pub(super) fn literal_value(op: &Op) -> Option<CanonicalBasisValue> {
    let Op::Literal(literal) = op else {
        return None;
    };
    let text = |value: String| CanonicalBasisValue::ExactText(value.into());
    Some(match literal {
        Literal::Bool(value) => CanonicalBasisValue::Bool(*value),
        Literal::Integer(value) => CanonicalBasisValue::SignedInteger {
            width: CanonicalIntegerWidth::Bits128,
            value: *value,
        },
        Literal::Float32(bits) => CanonicalBasisValue::FloatBits {
            width: CanonicalFloatWidth::Bits32,
            bits: u64::from(*bits),
        },
        Literal::Float64(bits) | Literal::Quantity(bits) => CanonicalBasisValue::FloatBits {
            width: CanonicalFloatWidth::Bits64,
            bits: *bits,
        },
        Literal::Decimal { coefficient, scale } => {
            CanonicalBasisValue::DecimalText(format!("{coefficient}e-{scale}").into())
        }
        Literal::String(value) => text(value.to_string()),
        Literal::Bytes(bytes) => text(hex(bytes)),
        Literal::Enum(variant) => CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits32,
            value: u128::from(*variant),
        },
        Literal::Bits(limbs) => text(limbs_hex(limbs)),
        Literal::Logic4 { value, unknown } => {
            text(format!("{}:{}", limbs_hex(value), limbs_hex(unknown)))
        }
        Literal::None => CanonicalBasisValue::Null,
    })
}

/// Most significant limb first; the node type carries the exact width.
fn limbs_hex(limbs: &[u64]) -> String {
    limbs
        .iter()
        .rev()
        .map(|limb| format!("{limb:016x}"))
        .collect()
}

/// Every nominal declaration reachable from `types`, keyed by versioned name.
///
/// Records reference only earlier records, so the walk terminates; the
/// visited map also keeps each declaration to one entry.
pub(super) fn nominal_declarations<'a>(
    schema: &ExpressionSchema,
    types: impl Iterator<Item = &'a ExpressionType>,
) -> BTreeMap<String, String> {
    let mut declarations = BTreeMap::new();
    let mut pending: Vec<&ExpressionType> = types.collect();
    while let Some(ty) = pending.pop() {
        match ty {
            ExpressionType::Option(inner) | ExpressionType::List(inner) => pending.push(inner),
            ExpressionType::Map(key, value) | ExpressionType::MapEntry(key, value) => {
                pending.extend([&**key, &**value]);
            }
            ExpressionType::Id(name) => {
                declarations
                    .entry(name.to_string())
                    .or_insert_with(|| format!("id {name}"));
            }
            ExpressionType::Enum(name) if declarations.contains_key(&name.to_string()) => {}
            ExpressionType::Enum(name) => {
                let variants: Vec<&str> = match schema.enumeration(name.name()) {
                    Some(declaration) => declaration.variants().collect(),
                    None if name.name() == ROUNDING_NAME => ROUNDING_VARIANTS.to_vec(),
                    None => unreachable!("admitted enum types resolve in their schema"),
                };
                declarations.insert(
                    name.to_string(),
                    format!("enum {name} {{{}}}", variants.join(", ")),
                );
            }
            ExpressionType::Record(name) if declarations.contains_key(&name.to_string()) => {}
            ExpressionType::Record(name) => {
                let declaration = schema
                    .record(name.name())
                    .expect("admitted record types resolve");
                let fields: Vec<String> = declaration
                    .fields()
                    .map(|(field, field_type)| {
                        pending.push(field_type);
                        format!("{field}: {field_type}")
                    })
                    .collect();
                declarations.insert(
                    name.to_string(),
                    format!("record {name} {{{}}}", fields.join(", ")),
                );
            }
            _ => {}
        }
    }
    declarations
}
