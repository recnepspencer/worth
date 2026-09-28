//! Bounded recursive-descent parser for the normative V1 grammar.
//!
//! Recursion is guarded: every descent that can nest charges the syntax-depth
//! ceiling before recursing, and the ceiling is small enough that hostile
//! nesting is denied long before the native stack is at risk.

use crate::expressions::denial::{
    ExpressionDenial, ExpressionDenialDetail, ExpressionOccurrence, ExpressionResource,
    ExpressionResult, SyntaxDenial,
};
use crate::expressions::profile::AdmissionMeter;

use super::ast::{BinaryOp, NodeId, SyntaxNode, SyntaxTree, UnaryOp};
use super::tokens::{Lexer, Punct, Token};
use super::SourceSpan;

/// Structural ceilings the parser enforces while it builds the tree.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SyntaxLimits {
    pub(crate) max_nodes: u32,
    pub(crate) max_depth: u16,
}

pub(crate) struct Parser<'source, 'meter> {
    lexer: Lexer<'source>,
    pub(super) token: Token,
    pub(super) span: SourceSpan,
    previous_end: u32,
    nesting: u16,
    limits: SyntaxLimits,
    pub(super) tree: SyntaxTree,
    meter: &'meter mut AdmissionMeter,
}

pub(crate) fn parse_source(
    source: &str,
    limits: SyntaxLimits,
    meter: &mut AdmissionMeter,
) -> ExpressionResult<SyntaxTree> {
    let mut parser = Parser {
        lexer: Lexer::new(source),
        token: Token::End,
        span: SourceSpan::new(0, 0),
        previous_end: 0,
        nesting: 0,
        limits,
        tree: SyntaxTree::new(true),
        meter,
    };
    parser.advance()?;
    parser.expression()?;
    if parser.token != Token::End {
        return Err(parser.syntax(SyntaxDenial::TrailingInput));
    }
    Ok(parser.tree)
}

impl Parser<'_, '_> {
    pub(super) fn advance(&mut self) -> ExpressionResult<Token> {
        let (token, span) = self.lexer.next_token()?;
        self.meter
            .charge(1 + u64::from(span.end().saturating_sub(span.start())))?;
        self.previous_end = self.span.end();
        let previous = std::mem::replace(&mut self.token, token);
        self.span = span;
        Ok(previous)
    }

    pub(super) fn at(&self, punct: Punct) -> bool {
        self.token == Token::Punct(punct)
    }

    pub(super) fn eat(&mut self, punct: Punct) -> ExpressionResult<bool> {
        if self.at(punct) {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(super) fn expect(&mut self, punct: Punct, expected: &'static str) -> ExpressionResult<()> {
        if self.eat(punct)? {
            Ok(())
        } else {
            Err(self.unexpected(expected))
        }
    }

    pub(super) fn unexpected(&self, expected: &'static str) -> ExpressionDenial {
        if self.token == Token::End {
            self.syntax(SyntaxDenial::UnexpectedEnd { expected })
        } else {
            self.syntax(SyntaxDenial::UnexpectedToken { expected })
        }
    }

    pub(super) fn syntax(&self, denial: SyntaxDenial) -> ExpressionDenial {
        ExpressionDenial::at(
            ExpressionDenialDetail::Syntax(denial),
            ExpressionOccurrence::Source(self.span),
        )
    }

    pub(super) fn identifier(&mut self, expected: &'static str) -> ExpressionResult<Box<str>> {
        if !matches!(self.token, Token::Identifier(_)) {
            return Err(self.unexpected(expected));
        }
        match self.advance()? {
            Token::Identifier(name) => Ok(name),
            _ => unreachable!("the current token was checked to be an identifier"),
        }
    }

    /// The span from `start` to the end of the last consumed token.
    pub(super) fn span_from(&self, start: SourceSpan) -> SourceSpan {
        start.to(SourceSpan::new(self.previous_end, self.previous_end))
    }

    pub(super) fn push(&mut self, node: SyntaxNode, start: SourceSpan) -> ExpressionResult<NodeId> {
        if self.tree.len() >= self.limits.max_nodes as usize {
            return Err(ExpressionDenial::at(
                ExpressionDenialDetail::ResourceExceeded {
                    resource: ExpressionResource::SyntaxNodes,
                    limit: u64::from(self.limits.max_nodes),
                },
                ExpressionOccurrence::Source(start),
            ));
        }
        self.meter.charge(1)?;
        let span = self.span_from(start);
        let (id, depth) = self.tree.push(node, Some(span));
        if depth > self.limits.max_depth {
            return Err(self.too_deep(span));
        }
        Ok(id)
    }

    fn too_deep(&self, span: SourceSpan) -> ExpressionDenial {
        ExpressionDenial::at(
            ExpressionDenialDetail::ResourceExceeded {
                resource: ExpressionResource::SyntaxDepth,
                limit: u64::from(self.limits.max_depth),
            },
            ExpressionOccurrence::Source(span),
        )
    }

    /// Runs `parse` one nesting level deeper, denying before recursion
    /// could exceed the syntax-depth ceiling.
    pub(super) fn nested<T>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> ExpressionResult<T>,
    ) -> ExpressionResult<T> {
        if self.nesting >= self.limits.max_depth {
            return Err(self.too_deep(self.span));
        }
        self.nesting += 1;
        let result = parse(self);
        self.nesting -= 1;
        result
    }

    pub(super) fn expression(&mut self) -> ExpressionResult<NodeId> {
        self.nested(|parser| {
            if parser.token != Token::Let {
                return parser.conditional();
            }
            let start = parser.span;
            parser.advance()?;
            let binder = parser.identifier("a let binder name")?;
            parser.expect(Punct::Assign, "`=`")?;
            let value = parser.expression()?;
            parser.expect(Punct::Semicolon, "`;`")?;
            let body = parser.expression()?;
            parser.push(
                SyntaxNode::Let {
                    binder,
                    value,
                    body,
                },
                start,
            )
        })
    }

    fn conditional(&mut self) -> ExpressionResult<NodeId> {
        let start = self.span;
        let condition = self.coalesce()?;
        if !self.eat(Punct::Question)? {
            return Ok(condition);
        }
        let then = self.expression()?;
        self.expect(Punct::Colon, "`:`")?;
        let otherwise = self.expression()?;
        self.push(
            SyntaxNode::Conditional {
                condition,
                then,
                otherwise,
            },
            start,
        )
    }

    fn coalesce(&mut self) -> ExpressionResult<NodeId> {
        let start = self.span;
        let left = self.disjunction()?;
        if !self.eat(Punct::Coalesce)? {
            return Ok(left);
        }
        let right = self.nested(Self::coalesce)?;
        self.binary(BinaryOp::Coalesce, left, right, start)
    }

    fn binary(
        &mut self,
        op: BinaryOp,
        left: NodeId,
        right: NodeId,
        start: SourceSpan,
    ) -> ExpressionResult<NodeId> {
        self.push(SyntaxNode::Binary { op, left, right }, start)
    }

    fn left_chain(
        &mut self,
        operand: fn(&mut Self) -> ExpressionResult<NodeId>,
        operator: fn(&Token) -> Option<BinaryOp>,
    ) -> ExpressionResult<NodeId> {
        let start = self.span;
        let mut left = operand(self)?;
        while let Some(op) = operator(&self.token) {
            self.advance()?;
            let right = operand(self)?;
            left = self.binary(op, left, right, start)?;
        }
        Ok(left)
    }

    fn disjunction(&mut self) -> ExpressionResult<NodeId> {
        self.left_chain(Self::conjunction, |token| {
            (*token == Token::Punct(Punct::OrOr)).then_some(BinaryOp::Or)
        })
    }

    fn conjunction(&mut self) -> ExpressionResult<NodeId> {
        self.left_chain(Self::equality, |token| {
            (*token == Token::Punct(Punct::AndAnd)).then_some(BinaryOp::And)
        })
    }

    fn single_step(
        &mut self,
        operand: fn(&mut Self) -> ExpressionResult<NodeId>,
        operator: fn(&Token) -> Option<BinaryOp>,
        chained: SyntaxDenial,
    ) -> ExpressionResult<NodeId> {
        let start = self.span;
        let left = operand(self)?;
        let Some(op) = operator(&self.token) else {
            return Ok(left);
        };
        self.advance()?;
        let right = operand(self)?;
        if operator(&self.token).is_some() {
            return Err(self.syntax(chained));
        }
        self.binary(op, left, right, start)
    }

    fn equality(&mut self) -> ExpressionResult<NodeId> {
        self.single_step(
            Self::relation,
            |token| match token {
                Token::Punct(Punct::EqualEqual) => Some(BinaryOp::Equal),
                Token::Punct(Punct::NotEqual) => Some(BinaryOp::NotEqual),
                _ => None,
            },
            SyntaxDenial::ChainedEquality,
        )
    }

    fn relation(&mut self) -> ExpressionResult<NodeId> {
        self.single_step(
            Self::additive,
            |token| match token {
                Token::Punct(Punct::Less) => Some(BinaryOp::Less),
                Token::Punct(Punct::LessEqual) => Some(BinaryOp::LessEqual),
                Token::Punct(Punct::Greater) => Some(BinaryOp::Greater),
                Token::Punct(Punct::GreaterEqual) => Some(BinaryOp::GreaterEqual),
                _ => None,
            },
            SyntaxDenial::ChainedComparison,
        )
    }

    fn additive(&mut self) -> ExpressionResult<NodeId> {
        self.left_chain(Self::product, |token| match token {
            Token::Punct(Punct::Plus) => Some(BinaryOp::Add),
            Token::Punct(Punct::Minus) => Some(BinaryOp::Subtract),
            _ => None,
        })
    }

    fn product(&mut self) -> ExpressionResult<NodeId> {
        self.left_chain(Self::unary, |token| match token {
            Token::Punct(Punct::Star) => Some(BinaryOp::Multiply),
            Token::Punct(Punct::Slash) => Some(BinaryOp::Divide),
            Token::Punct(Punct::Percent) => Some(BinaryOp::Remainder),
            _ => None,
        })
    }

    fn unary(&mut self) -> ExpressionResult<NodeId> {
        let op = match self.token {
            Token::Punct(Punct::Bang) => UnaryOp::Not,
            Token::Punct(Punct::Minus) => UnaryOp::Negate,
            _ => return self.postfix(),
        };
        let start = self.span;
        self.advance()?;
        let operand = self.nested(Self::unary)?;
        self.push(SyntaxNode::Unary { op, operand }, start)
    }
}
