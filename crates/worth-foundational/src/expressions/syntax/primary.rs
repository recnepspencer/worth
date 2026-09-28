//! Postfix, primary, and type-syntax productions of the V1 grammar.

use crate::expressions::denial::{
    ExpressionDenial, ExpressionDenialDetail, ExpressionOccurrence, ExpressionResult,
};

use super::ast::{ComprehensionKind, NodeId, QualifiedName, SyntaxNode, TypeArgument, TypeSyntax};
use super::parser::Parser;
use super::tokens::{Punct, Token};

/// Reserved intrinsics that take compile-time type arguments in `<...>`.
///
/// Only these names read a following `<` as a type-argument list; anywhere
/// else `<` is the less-than operator.
pub(crate) const GENERIC_INTRINSICS: [&str; 7] = [
    "bits",
    "exact_cast",
    "rounded_cast",
    "truncate",
    "extend",
    "to_bits",
    "slice",
];

fn comprehension(name: &str) -> Option<ComprehensionKind> {
    match name {
        "map" => Some(ComprehensionKind::Map),
        "filter" => Some(ComprehensionKind::Filter),
        "all" => Some(ComprehensionKind::All),
        "any" => Some(ComprehensionKind::Any),
        _ => None,
    }
}

impl Parser<'_, '_> {
    pub(super) fn postfix(&mut self) -> ExpressionResult<NodeId> {
        let start = self.span;
        let mut base = self.primary()?;
        while self.eat(Punct::Dot)? {
            let name = self.identifier("a field or comprehension name")?;
            let node = match comprehension(&name) {
                Some(kind) if self.at(Punct::LeftParen) => {
                    self.advance()?;
                    let binder = self.identifier("a comprehension binder name")?;
                    self.expect(Punct::Comma, "`,`")?;
                    let body = self.expression()?;
                    self.expect(Punct::RightParen, "`)`")?;
                    SyntaxNode::Comprehension {
                        base,
                        kind,
                        binder,
                        body,
                    }
                }
                _ => SyntaxNode::Field { base, field: name },
            };
            base = self.push(node, start)?;
        }
        Ok(base)
    }

    fn primary(&mut self) -> ExpressionResult<NodeId> {
        let start = self.span;
        let node = match self.advance()? {
            Token::True => SyntaxNode::Bool(true),
            Token::False => SyntaxNode::Bool(false),
            Token::Integer(value) => SyntaxNode::Integer(value),
            Token::Float(value) => SyntaxNode::Float(value),
            Token::String(text) => SyntaxNode::String(text),
            Token::None => {
                self.expect(Punct::Less, "`<` after `none`")?;
                let ty = self.type_syntax()?;
                self.expect(Punct::Greater, "`>`")?;
                SyntaxNode::None(ty)
            }
            Token::Punct(Punct::LeftParen) => {
                let inner = self.expression()?;
                self.expect(Punct::RightParen, "`)`")?;
                return Ok(inner);
            }
            Token::Punct(Punct::LeftBracket) => {
                SyntaxNode::List(self.sequence(Punct::RightBracket, Self::expression)?)
            }
            Token::Punct(Punct::LeftBrace) => {
                SyntaxNode::Map(self.sequence(Punct::RightBrace, Self::map_entry)?)
            }
            Token::Identifier(first) => self.named(first)?,
            other => {
                self.token = other;
                self.span = start;
                return Err(self.unexpected("an expression"));
            }
        };
        self.push(node, start)
    }

    fn map_entry(&mut self) -> ExpressionResult<(NodeId, NodeId)> {
        let key = self.expression()?;
        self.expect(Punct::Colon, "`:`")?;
        Ok((key, self.expression()?))
    }

    fn record_field(&mut self) -> ExpressionResult<(Box<str>, NodeId)> {
        let name = self.identifier("a record field name")?;
        self.expect(Punct::Colon, "`:`")?;
        Ok((name, self.expression()?))
    }

    /// Comma-separated items up to `close`, accepting a trailing comma.
    fn sequence<T>(
        &mut self,
        close: Punct,
        item: fn(&mut Self) -> ExpressionResult<T>,
    ) -> ExpressionResult<Vec<T>> {
        let mut items = Vec::new();
        while !self.eat(close)? {
            items.push(item(self)?);
            if !self.eat(Punct::Comma)? {
                self.expect(close, "`,` or a closing delimiter")?;
                break;
            }
        }
        Ok(items)
    }

    /// A name reference, call, generic intrinsic call, or record literal.
    fn named(&mut self, first: Box<str>) -> ExpressionResult<SyntaxNode> {
        let name = self.qualified_rest(first)?;
        let generic = name
            .single()
            .is_some_and(|single| GENERIC_INTRINSICS.contains(&single));
        let type_arguments = if generic && self.eat(Punct::Less)? {
            let arguments = self.sequence(Punct::Greater, Self::type_argument)?;
            if !self.at(Punct::LeftParen) {
                return Err(self.unexpected("`(` after type arguments"));
            }
            arguments
        } else {
            Vec::new()
        };
        if self.eat(Punct::LeftParen)? {
            let arguments = self.sequence(Punct::RightParen, Self::expression)?;
            return Ok(SyntaxNode::Call {
                function: name,
                type_arguments,
                arguments,
            });
        }
        if self.eat(Punct::LeftBrace)? {
            let fields = self.sequence(Punct::RightBrace, Self::record_field)?;
            return Ok(SyntaxNode::Record {
                type_name: name,
                fields,
            });
        }
        Ok(SyntaxNode::Name(name))
    }

    fn qualified_rest(&mut self, first: Box<str>) -> ExpressionResult<QualifiedName> {
        let mut segments = vec![first];
        while self.eat(Punct::PathSeparator)? {
            segments.push(self.identifier("a name segment after `::`")?);
        }
        Ok(QualifiedName(segments))
    }

    pub(super) fn type_syntax(&mut self) -> ExpressionResult<TypeSyntax> {
        self.nested(|parser| {
            let first = parser.identifier("a type name")?;
            let name = parser.qualified_rest(first)?;
            let arguments = if parser.eat(Punct::Less)? {
                parser.sequence(Punct::Greater, Self::type_argument)?
            } else {
                Vec::new()
            };
            Ok(TypeSyntax {
                name,
                arguments,
                version: None,
            })
        })
    }

    /// A width, a type, or a dimension product such as `length*length/time`.
    fn type_argument(&mut self) -> ExpressionResult<TypeArgument> {
        if let Token::Integer(width) = self.token {
            let span = self.span;
            self.advance()?;
            let width = u32::try_from(width).map_err(|_| {
                ExpressionDenial::at(
                    ExpressionDenialDetail::InvalidValue("type width out of range"),
                    ExpressionOccurrence::Source(span),
                )
            })?;
            if width != 1 || !self.at(Punct::Slash) {
                return Ok(TypeArgument::Width(width));
            }
            // `1/time`: a product with only divisors.
            return self.dimension_product(Vec::new());
        }
        let ty = self.type_syntax()?;
        if !(self.at(Punct::Star) || self.at(Punct::Slash)) {
            return Ok(TypeArgument::Type(ty));
        }
        if !ty.arguments.is_empty() {
            return Err(self.unexpected("a dimension name"));
        }
        self.dimension_product(vec![(false, ty.name)])
    }

    fn dimension_product(
        &mut self,
        mut factors: Vec<(bool, QualifiedName)>,
    ) -> ExpressionResult<TypeArgument> {
        loop {
            let divide = if self.eat(Punct::Star)? {
                false
            } else if self.eat(Punct::Slash)? {
                true
            } else {
                return Ok(TypeArgument::Product(factors));
            };
            let first = self.identifier("a dimension name")?;
            factors.push((divide, self.qualified_rest(first)?));
        }
    }
}
