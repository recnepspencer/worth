//! Operators, conditionals, `let`, and comprehensions on the builder.

use crate::expressions::syntax::ast::{BinaryOp, ComprehensionKind, SyntaxNode, UnaryOp};

use super::{Boolean, Dynamic, ExpressionBuilder, Numeric, Ordered, Term, TermKind};

impl ExpressionBuilder {
    pub fn not(&mut self, operand: Term<Boolean>) -> Term<Boolean> {
        self.unary(UnaryOp::Not, operand)
    }

    pub fn negate(&mut self, operand: Term<Numeric>) -> Term<Numeric> {
        self.unary(UnaryOp::Negate, operand)
    }

    pub fn add(&mut self, left: Term<Numeric>, right: Term<Numeric>) -> Term<Numeric> {
        self.binary(BinaryOp::Add, left, right)
    }

    pub fn subtract(&mut self, left: Term<Numeric>, right: Term<Numeric>) -> Term<Numeric> {
        self.binary(BinaryOp::Subtract, left, right)
    }

    pub fn multiply(&mut self, left: Term<Numeric>, right: Term<Numeric>) -> Term<Numeric> {
        self.binary(BinaryOp::Multiply, left, right)
    }

    pub fn divide(&mut self, left: Term<Numeric>, right: Term<Numeric>) -> Term<Numeric> {
        self.binary(BinaryOp::Divide, left, right)
    }

    pub fn remainder(&mut self, left: Term<Numeric>, right: Term<Numeric>) -> Term<Numeric> {
        self.binary(BinaryOp::Remainder, left, right)
    }

    pub fn less<K: Ordered>(&mut self, left: Term<K>, right: Term<K>) -> Term<Boolean> {
        self.binary(BinaryOp::Less, left, right)
    }

    pub fn less_equal<K: Ordered>(&mut self, left: Term<K>, right: Term<K>) -> Term<Boolean> {
        self.binary(BinaryOp::LessEqual, left, right)
    }

    pub fn greater<K: Ordered>(&mut self, left: Term<K>, right: Term<K>) -> Term<Boolean> {
        self.binary(BinaryOp::Greater, left, right)
    }

    pub fn greater_equal<K: Ordered>(&mut self, left: Term<K>, right: Term<K>) -> Term<Boolean> {
        self.binary(BinaryOp::GreaterEqual, left, right)
    }

    pub fn equal<K: TermKind>(&mut self, left: Term<K>, right: Term<K>) -> Term<Boolean> {
        self.binary(BinaryOp::Equal, left, right)
    }

    pub fn not_equal<K: TermKind>(&mut self, left: Term<K>, right: Term<K>) -> Term<Boolean> {
        self.binary(BinaryOp::NotEqual, left, right)
    }

    /// Lazy: `right` evaluates only when `left` is true.
    pub fn and(&mut self, left: Term<Boolean>, right: Term<Boolean>) -> Term<Boolean> {
        self.binary(BinaryOp::And, left, right)
    }

    /// Lazy: `right` evaluates only when `left` is false.
    pub fn or(&mut self, left: Term<Boolean>, right: Term<Boolean>) -> Term<Boolean> {
        self.binary(BinaryOp::Or, left, right)
    }

    /// `option ?? fallback`: lazy in `fallback`.
    pub fn coalesce<K: TermKind>(&mut self, option: Term<Dynamic>, fallback: Term<K>) -> Term<K> {
        self.binary(BinaryOp::Coalesce, option, fallback)
    }

    pub fn conditional<K: TermKind>(
        &mut self,
        condition: Term<Boolean>,
        then: Term<K>,
        otherwise: Term<K>,
    ) -> Term<K> {
        let condition = self.own(condition);
        let then = self.own(then);
        let otherwise = self.own(otherwise);
        self.push(SyntaxNode::Conditional {
            condition,
            then,
            otherwise,
        })
    }

    /// `let binder = value; body`, where `body` receives the binder reference.
    pub fn let_in<J: TermKind, K: TermKind>(
        &mut self,
        binder: &str,
        value: Term<J>,
        body: impl FnOnce(&mut Self, Term<Dynamic>) -> Term<K>,
    ) -> Term<K> {
        let value = self.own(value);
        let binder = self.identifier(binder);
        let reference = self.name(&binder);
        let body = body(self, reference);
        let body = self.own(body);
        self.push(SyntaxNode::Let {
            binder,
            value,
            body,
        })
    }

    /// `base.map(binder, body)`.
    pub fn map<J: TermKind, K: TermKind>(
        &mut self,
        base: Term<J>,
        binder: &str,
        body: impl FnOnce(&mut Self, Term<Dynamic>) -> Term<K>,
    ) -> Term<Dynamic> {
        self.comprehension(ComprehensionKind::Map, base, binder, body)
    }

    /// `base.filter(binder, predicate)`.
    pub fn filter<J: TermKind>(
        &mut self,
        base: Term<J>,
        binder: &str,
        predicate: impl FnOnce(&mut Self, Term<Dynamic>) -> Term<Boolean>,
    ) -> Term<Dynamic> {
        self.comprehension(ComprehensionKind::Filter, base, binder, predicate)
    }

    /// `base.all(binder, predicate)`: true for an empty list.
    pub fn all<J: TermKind>(
        &mut self,
        base: Term<J>,
        binder: &str,
        predicate: impl FnOnce(&mut Self, Term<Dynamic>) -> Term<Boolean>,
    ) -> Term<Boolean> {
        self.comprehension(ComprehensionKind::All, base, binder, predicate)
            .retype()
    }

    /// `base.any(binder, predicate)`: false for an empty list.
    pub fn any<J: TermKind>(
        &mut self,
        base: Term<J>,
        binder: &str,
        predicate: impl FnOnce(&mut Self, Term<Dynamic>) -> Term<Boolean>,
    ) -> Term<Boolean> {
        self.comprehension(ComprehensionKind::Any, base, binder, predicate)
            .retype()
    }

    fn comprehension<J: TermKind, K: TermKind>(
        &mut self,
        kind: ComprehensionKind,
        base: Term<J>,
        binder: &str,
        body: impl FnOnce(&mut Self, Term<Dynamic>) -> Term<K>,
    ) -> Term<Dynamic> {
        let base = self.own(base);
        let binder = self.identifier(binder);
        let reference = self.name(&binder);
        let body = body(self, reference);
        let body = self.own(body);
        self.push(SyntaxNode::Comprehension {
            base,
            kind,
            binder,
            body,
        })
    }

    fn unary<K: TermKind>(&mut self, op: UnaryOp, operand: Term<K>) -> Term<K> {
        let operand = self.own(operand);
        self.push(SyntaxNode::Unary { op, operand })
    }

    fn binary<J: TermKind, K: TermKind, R: TermKind>(
        &mut self,
        op: BinaryOp,
        left: Term<J>,
        right: Term<K>,
    ) -> Term<R> {
        let left = self.own(left);
        let right = self.own(right);
        self.push(SyntaxNode::Binary { op, left, right })
    }
}
