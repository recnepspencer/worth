//! Reserved typed literal constructors and unit-bearing quantity calls.
//!
//! Constructors take literals only and fold at admission; nonliteral values
//! convert through the explicit cast contracts instead.

use crate::expressions::denial::{
    ExpressionDenial, ExpressionDenialDetail, ExpressionResource, ExpressionResult,
};
use crate::expressions::functions::Builtin;
use crate::expressions::numeric::scaled_decimal;
use crate::expressions::profile::check_limit;
use crate::expressions::program::{Literal, Op};
use crate::expressions::syntax::ast::{NodeId, SyntaxNode, TypeArgument, UnaryOp};
use crate::expressions::syntax::literal::DecimalText;
use crate::expressions::types::units::resolve_unit;
use crate::expressions::types::{ExpressionType, IntegerType};

use super::Checker;

/// Decimal values carry at most 38 significant digits and scale 0..=18.
const DECIMAL_DIGITS: usize = 38;
const DECIMAL_SCALE: usize = 18;

/// Integer constructor names, in `IntegerType::ALL` order. Names are
/// case-sensitive like every other name.
const INTEGER_CONSTRUCTORS: [&str; 8] = [
    "int8", "int16", "int32", "int64", "uint8", "uint16", "uint32", "uint64",
];

/// The reserved constructors. Dispatch goes through this table, so every
/// constructor is subject to the one type-argument rule.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Constructor {
    Integer(IntegerType),
    Float32,
    Float64,
    Decimal,
    Bytes,
    Logic4,
    /// `bits<N>`: the only constructor with a type argument.
    Bits,
    Quantity,
    Magnitude,
}

impl Constructor {
    fn named(name: &str) -> Option<Self> {
        if let Some(index) = INTEGER_CONSTRUCTORS.iter().position(|c| *c == name) {
            return Some(Self::Integer(IntegerType::ALL[index]));
        }
        Some(match name {
            "float32" => Self::Float32,
            "float64" => Self::Float64,
            "decimal" => Self::Decimal,
            "bytes" => Self::Bytes,
            "logic4" => Self::Logic4,
            "bits" => Self::Bits,
            "quantity" => Self::Quantity,
            "magnitude" => Self::Magnitude,
            _ => return None,
        })
    }
}

/// A numeric literal read directly from syntax, sign included.
enum NumberLiteral<'a> {
    Integer(bool, u128),
    Float(bool, &'a DecimalText),
}

impl Checker<'_> {
    pub(super) fn constructor(
        &mut self,
        id: NodeId,
        name: &str,
        type_arguments: &[TypeArgument],
        arguments: &[NodeId],
    ) -> Option<ExpressionResult<u32>> {
        let constructor = Constructor::named(name)?;
        if !type_arguments.is_empty() && constructor != Constructor::Bits {
            return Some(Err(self.deny(
                id,
                ExpressionDenialDetail::UnsupportedFeature("this call takes no type arguments"),
            )));
        }
        let result = match (constructor, arguments) {
            (Constructor::Integer(integer), [argument]) => {
                self.integer_literal(id, integer, *argument)
            }
            (Constructor::Float32 | Constructor::Float64, [argument]) => {
                self.float_literal(id, constructor == Constructor::Float32, *argument)
            }
            (Constructor::Decimal, [argument]) => self.text_literal(id, *argument, decimal_literal),
            (Constructor::Bytes, [argument]) => self.text_literal(id, *argument, bytes_literal),
            (Constructor::Logic4, [argument]) => self.text_literal(id, *argument, logic4_literal),
            (Constructor::Bits, [argument]) => match type_arguments {
                [TypeArgument::Width(width)] => {
                    let width = *width;
                    self.text_literal(id, *argument, move |text| bits_literal(text, width))
                }
                _ => Err(self.deny(
                    id,
                    ExpressionDenialDetail::TypeRequired("bits<N>(\"binary\")"),
                )),
            },
            (Constructor::Quantity, [magnitude, unit]) => self.quantity(id, *magnitude, *unit),
            (Constructor::Magnitude, [quantity, unit]) => self.magnitude(id, *quantity, *unit),
            _ => return None,
        };
        Some(result)
    }

    fn number_literal(&self, node: NodeId) -> Option<NumberLiteral<'_>> {
        let (negative, node) = match self.tree.node(node) {
            SyntaxNode::Unary {
                op: UnaryOp::Negate,
                operand,
            } => (true, *operand),
            _ => (false, node),
        };
        match self.tree.node(node) {
            SyntaxNode::Integer(value) => Some(NumberLiteral::Integer(negative, *value)),
            SyntaxNode::Float(decimal) => Some(NumberLiteral::Float(negative, decimal)),
            _ => None,
        }
    }

    fn literal_required(&self, id: NodeId) -> ExpressionDenial {
        self.deny(
            id,
            ExpressionDenialDetail::UnsupportedFeature(
                "typed constructors take a literal; convert values with exact_cast",
            ),
        )
    }

    fn integer_literal(
        &mut self,
        id: NodeId,
        integer: IntegerType,
        argument: NodeId,
    ) -> ExpressionResult<u32> {
        let Some(NumberLiteral::Integer(negative, magnitude)) = self.number_literal(argument)
        else {
            return Err(self.literal_required(id));
        };
        let value = i128::try_from(magnitude)
            .ok()
            .map(|magnitude| if negative { -magnitude } else { magnitude })
            .filter(|value| (integer.min()..=integer.max()).contains(value))
            .ok_or_else(|| {
                self.deny(
                    id,
                    ExpressionDenialDetail::InvalidValue("integer literal out of range"),
                )
            })?;
        self.literal(
            id,
            Literal::Integer(value),
            ExpressionType::Integer(integer),
        )
    }

    fn float_literal(
        &mut self,
        id: NodeId,
        single: bool,
        argument: NodeId,
    ) -> ExpressionResult<u32> {
        let (negative, decimal) = match self.number_literal(argument) {
            Some(NumberLiteral::Float(negative, decimal)) => (negative, decimal.clone()),
            Some(NumberLiteral::Integer(negative, magnitude)) => (
                negative,
                DecimalText::parse(&magnitude.to_string()).expect("integer digits"),
            ),
            None => return Err(self.literal_required(id)),
        };
        let not_finite = || ExpressionDenialDetail::InvalidValue("float literal is not finite");
        // Negation is exact; adding positive zero normalizes negative zero.
        let (literal, ty) = if single {
            let value = decimal
                .to_f32()
                .ok_or_else(|| self.deny(id, not_finite()))?;
            let value = if negative { -value } else { value } + 0.0;
            (Literal::Float32(value.to_bits()), ExpressionType::Float32)
        } else {
            let value = decimal
                .to_f64()
                .ok_or_else(|| self.deny(id, not_finite()))?;
            let value = if negative { -value } else { value } + 0.0;
            (Literal::Float64(value.to_bits()), ExpressionType::Float64)
        };
        self.literal(id, literal, ty)
    }

    fn text_literal(
        &mut self,
        id: NodeId,
        argument: NodeId,
        parse: impl FnOnce(&str) -> Result<(Literal, ExpressionType), &'static str>,
    ) -> ExpressionResult<u32> {
        let SyntaxNode::String(text) = self.tree.node(argument) else {
            return Err(self.literal_required(id));
        };
        // Parsing reads and copies the text; charge it first.
        self.meter.charge(text.len() as u64)?;
        let (literal, ty) = parse(text)
            .map_err(|reason| self.deny(id, ExpressionDenialDetail::InvalidValue(reason)))?;
        if let ExpressionType::Bits(width) | ExpressionType::Logic4(width) = ty {
            check_limit(
                self.context.profile,
                ExpressionResource::BitWidth,
                u64::from(width),
            )?;
        }
        self.literal(id, literal, ty)
    }

    /// Folds a literal magnitude with one rounding; otherwise scales a
    /// Float64 value at evaluation.
    fn quantity(&mut self, id: NodeId, magnitude: NodeId, unit: NodeId) -> ExpressionResult<u32> {
        let unit = resolve_unit(self.tree, unit)?;
        let ty = ExpressionType::Quantity(unit.dimension);
        let literal = match self.number_literal(magnitude) {
            Some(NumberLiteral::Float(negative, decimal)) => Some((negative, decimal.clone())),
            Some(NumberLiteral::Integer(negative, value)) => Some((
                negative,
                DecimalText::parse(&value.to_string()).expect("integer digits"),
            )),
            None => None,
        };
        if let Some((negative, decimal)) = literal {
            self.meter.charge(decimal.digits().len() as u64 * 64)?;
            let bits = scaled_decimal(&decimal, negative, unit.scale)
                .ok_or_else(|| {
                    self.deny(
                        id,
                        ExpressionDenialDetail::InvalidValue("quantity literal out of range"),
                    )
                })?
                .to_bits();
            return self.literal(id, Literal::Quantity(bits), ty);
        }
        let magnitude = self.check(magnitude, Some(&ExpressionType::Float64))?;
        self.emit(
            id,
            Op::Builtin(Builtin::Quantity(unit.scale)),
            ty,
            vec![magnitude],
        )
    }

    fn magnitude(&mut self, id: NodeId, quantity: NodeId, unit: NodeId) -> ExpressionResult<u32> {
        let unit = resolve_unit(self.tree, unit)?;
        let quantity = self.check(quantity, Some(&ExpressionType::Quantity(unit.dimension)))?;
        let op = Op::Builtin(Builtin::Magnitude(unit.scale));
        self.emit(id, op, ExpressionType::Float64, vec![quantity])
    }
}

/// `-?digits[.digits]`, normalized: redundant zeros stripped, one zero.
fn decimal_literal(text: &str) -> Result<(Literal, ExpressionType), &'static str> {
    const MALFORMED: &str = "decimal literal is malformed";
    let (negative, unsigned) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    if whole.is_empty()
        || !digits(whole)
        || !digits(fraction)
        || (unsigned.contains('.') && fraction.is_empty())
    {
        return Err(MALFORMED);
    }
    let fraction = fraction.trim_end_matches('0');
    let significant = format!("{whole}{fraction}");
    let significant = significant.trim_start_matches('0');
    if significant.len() > DECIMAL_DIGITS || fraction.len() > DECIMAL_SCALE {
        return Err("decimal literal exceeds 38 digits or scale 18");
    }
    let magnitude: i128 = if significant.is_empty() {
        0
    } else {
        significant.parse().map_err(|_| MALFORMED)?
    };
    let coefficient = if negative { -magnitude } else { magnitude };
    let scale = if coefficient == 0 {
        0
    } else {
        fraction.len() as u8
    };
    Ok((
        Literal::Decimal { coefficient, scale },
        ExpressionType::Decimal,
    ))
}

fn bytes_literal(text: &str) -> Result<(Literal, ExpressionType), &'static str> {
    if !text.len().is_multiple_of(2) {
        return Err("bytes literal has an odd number of hex digits");
    }
    let bytes = text
        .as_bytes()
        .chunks(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).map_err(|_| "bytes literal is not hex")?;
            u8::from_str_radix(pair, 16).map_err(|_| "bytes literal is not hex")
        })
        .collect::<Result<Vec<u8>, _>>()?;
    Ok((
        Literal::Bytes(bytes.into_boxed_slice()),
        ExpressionType::Bytes,
    ))
}

/// Most significant bit first; the text length must equal the width.
fn bits_literal(text: &str, width: u32) -> Result<(Literal, ExpressionType), &'static str> {
    if width == 0 || text.len() != width as usize {
        return Err("bits literal length must equal its declared width");
    }
    let mut limbs = vec![0_u64; (width as usize).div_ceil(64)];
    for (bit, symbol) in text.bytes().rev().enumerate() {
        match symbol {
            b'0' => {}
            b'1' => limbs[bit / 64] |= 1 << (bit % 64),
            _ => return Err("bits literal digits are 0 or 1"),
        }
    }
    Ok((
        Literal::Bits(limbs.into_boxed_slice()),
        ExpressionType::Bits(width),
    ))
}

/// Most significant symbol first; the width is the text length.
fn logic4_literal(text: &str) -> Result<(Literal, ExpressionType), &'static str> {
    let width = u32::try_from(text.len()).map_err(|_| "logic4 literal is too wide")?;
    if width == 0 {
        return Err("logic4 literal has at least one symbol");
    }
    let limbs = text.len().div_ceil(64);
    let (mut value, mut unknown) = (vec![0_u64; limbs], vec![0_u64; limbs]);
    for (bit, symbol) in text.bytes().rev().enumerate() {
        let (value_bit, unknown_bit) = match symbol {
            b'0' => (0, 0),
            b'1' => (1, 0),
            b'X' => (0, 1),
            b'Z' => (1, 1),
            _ => return Err("logic4 symbols are 0, 1, X, or Z"),
        };
        value[bit / 64] |= value_bit << (bit % 64);
        unknown[bit / 64] |= unknown_bit << (bit % 64);
    }
    let literal = Literal::Logic4 {
        value: value.into_boxed_slice(),
        unknown: unknown.into_boxed_slice(),
    };
    Ok((literal, ExpressionType::Logic4(width)))
}
