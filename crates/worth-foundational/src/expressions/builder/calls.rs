//! Calls, typed constructors, and collection literals on the builder.
//!
//! Type arguments are resolved types; nominal types keep the exact version
//! the builder meant, and admission denies a schema declaring another.

use crate::expressions::syntax::ast::{
    BinaryOp, QualifiedName, SyntaxNode, TypeArgument, TypeSyntax,
};
use crate::expressions::types::{BaseDimension, ExpressionDimension, ExpressionType};

use super::{Dynamic, ExpressionBuilder, Numeric, Term, TermKind};

/// A type argument to a generic intrinsic such as `exact_cast<T>` or
/// `truncate<N>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionTypeArgument {
    Type(ExpressionType),
    Width(u32),
}

impl ExpressionBuilder {
    /// A builtin, typed constructor, or installed function call.
    pub fn call(&mut self, function: &str, arguments: &[Term<Dynamic>]) -> Term<Dynamic> {
        self.generic_call(function, &[], arguments)
    }

    /// A generic intrinsic call such as `exact_cast<Int32>(x)`.
    pub fn generic_call(
        &mut self,
        function: &str,
        type_arguments: &[ExpressionTypeArgument],
        arguments: &[Term<Dynamic>],
    ) -> Term<Dynamic> {
        let function = self.qualified(function);
        let type_arguments = type_arguments
            .iter()
            .map(|argument| match argument {
                ExpressionTypeArgument::Type(ty) => TypeArgument::Type(type_syntax(ty)),
                ExpressionTypeArgument::Width(width) => TypeArgument::Width(*width),
            })
            .collect();
        let arguments = arguments
            .iter()
            .map(|argument| self.own(*argument))
            .collect();
        self.push(SyntaxNode::Call {
            function,
            type_arguments,
            arguments,
        })
    }

    /// `quantity(magnitude, unit)`; the unit is catalog names with integer
    /// powers, such as `[("m", 1), ("s", -2)]`.
    pub fn quantity(&mut self, magnitude: Term<Numeric>, unit: &[(&str, i8)]) -> Term<Numeric> {
        let unit = self.unit(unit);
        self.call("quantity", &[magnitude.dynamic(), unit])
            .numeric()
    }

    /// `magnitude(quantity, unit)`: the Float64 magnitude in `unit`.
    pub fn magnitude<K: TermKind>(
        &mut self,
        quantity: Term<K>,
        unit: &[(&str, i8)],
    ) -> Term<Numeric> {
        let unit = self.unit(unit);
        self.call("magnitude", &[quantity.dynamic(), unit])
            .numeric()
    }

    /// `none<T>`.
    pub fn none(&mut self, ty: &ExpressionType) -> Term<Dynamic> {
        self.push(SyntaxNode::None(type_syntax(ty)))
    }

    pub fn list_of(&mut self, items: &[Term<Dynamic>]) -> Term<Dynamic> {
        let items = items.iter().map(|item| self.own(*item)).collect();
        self.push(SyntaxNode::List(items))
    }

    pub fn map_of(&mut self, entries: &[(Term<Dynamic>, Term<Dynamic>)]) -> Term<Dynamic> {
        let entries = entries
            .iter()
            .map(|(key, value)| (self.own(*key), self.own(*value)))
            .collect();
        self.push(SyntaxNode::Map(entries))
    }

    /// `type_name { field: value, ... }`.
    pub fn record(&mut self, type_name: &str, fields: &[(&str, Term<Dynamic>)]) -> Term<Dynamic> {
        let type_name = self.qualified(type_name);
        let fields = fields
            .iter()
            .map(|(field, value)| (self.identifier(field), self.own(*value)))
            .collect();
        self.push(SyntaxNode::Record { type_name, fields })
    }

    /// Numerator units multiply left to right; denominator units divide, as
    /// `kg*m/s/s` parses. A unit with only divisors starts from `1`.
    fn unit(&mut self, unit: &[(&str, i8)]) -> Term<Dynamic> {
        let factors = |negative: bool| {
            unit.iter()
                .filter(move |(_, power)| (*power < 0) == negative)
                .flat_map(|(name, power)| std::iter::repeat_n(*name, power.unsigned_abs().into()))
        };
        let mut term: Option<Term<Dynamic>> = None;
        for name in factors(false).collect::<Vec<_>>() {
            let factor = self.name(name);
            term = Some(match term {
                Some(left) => self.unit_step(BinaryOp::Multiply, left, factor),
                None => factor,
            });
        }
        let mut term = match term {
            Some(term) => term,
            None => self.push(SyntaxNode::Integer(1)),
        };
        for name in factors(true).collect::<Vec<_>>() {
            let factor = self.name(name);
            term = self.unit_step(BinaryOp::Divide, term, factor);
        }
        term
    }

    fn unit_step(
        &mut self,
        op: BinaryOp,
        left: Term<Dynamic>,
        right: Term<Dynamic>,
    ) -> Term<Dynamic> {
        self.push(SyntaxNode::Binary {
            op,
            left: left.node,
            right: right.node,
        })
    }
}

fn plain(name: &str, arguments: Vec<TypeArgument>) -> TypeSyntax {
    TypeSyntax {
        name: QualifiedName(name.split("::").map(Box::from).collect()),
        arguments,
        version: None,
    }
}

/// The type syntax that resolves to `ty`, spelled as source would spell it.
fn type_syntax(ty: &ExpressionType) -> TypeSyntax {
    let nested = |inner: &ExpressionType| TypeArgument::Type(type_syntax(inner));
    match ty {
        ExpressionType::Id(name) | ExpressionType::Enum(name) | ExpressionType::Record(name) => {
            TypeSyntax {
                version: Some(name.version()),
                ..plain(name.name(), Vec::new())
            }
        }
        ExpressionType::Option(inner) => plain("Option", vec![nested(inner)]),
        ExpressionType::List(inner) => plain("List", vec![nested(inner)]),
        ExpressionType::Map(key, value) => plain("Map", vec![nested(key), nested(value)]),
        ExpressionType::MapEntry(key, value) => plain("MapEntry", vec![nested(key), nested(value)]),
        ExpressionType::Quantity(dimension) => {
            plain("Quantity", vec![dimension_argument(*dimension)])
        }
        ExpressionType::Bits(width) => plain("Bits", vec![TypeArgument::Width(*width)]),
        ExpressionType::Logic4(width) => plain("Logic4", vec![TypeArgument::Width(*width)]),
        scalar => plain(&scalar.to_string(), Vec::new()),
    }
}

/// Positive exponents first, then divisors, each base repeated per power.
fn dimension_argument(dimension: ExpressionDimension) -> TypeArgument {
    if dimension.is_dimensionless() {
        return TypeArgument::Type(plain("dimensionless", Vec::new()));
    }
    let factors = |negative: bool| {
        BaseDimension::ALL.into_iter().flat_map(move |base| {
            let exponent = dimension.exponent(base);
            let count = if (exponent < 0) == negative {
                exponent.unsigned_abs()
            } else {
                0
            };
            std::iter::repeat_n(
                (negative, QualifiedName(vec![base.name().into()])),
                count.into(),
            )
        })
    };
    let product: Vec<_> = factors(false).chain(factors(true)).collect();
    match product.as_slice() {
        [(false, name)] => TypeArgument::Type(TypeSyntax {
            name: name.clone(),
            arguments: Vec::new(),
            version: None,
        }),
        _ => TypeArgument::Product(product),
    }
}
