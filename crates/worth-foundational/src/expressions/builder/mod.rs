//! The typed Rust expression builder.
//!
//! A builder produces the same draft a source parse produces, without source
//! text. Term kinds reject operator and operand combinations Rust can see;
//! names, fields, and overloads still resolve at admission. Invalid names and
//! foreign terms are recorded and reported by [`ExpressionBuilder::finish`],
//! so building stays a chain of plain calls.

mod calls;
mod operators;

use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

use super::denial::{
    ExpressionDenial, ExpressionDenialDetail, ExpressionResource, ExpressionResult, SyntaxDenial,
};
use super::draft::ExpressionDraft;
use super::profile::{check_limit, ExpressionProfile};
use super::syntax::ast::{NodeId, QualifiedName, SyntaxNode, SyntaxTree};
use super::syntax::literal::DecimalText;
use super::types::is_identifier;

pub use calls::ExpressionTypeArgument;

static NEXT_BUILDER: AtomicU64 = AtomicU64::new(0);

mod sealed {
    pub trait Sealed {}
}

/// What Rust knows about a term before admission.
pub trait TermKind: sealed::Sealed + Copy {}

/// A Bool-valued term.
#[derive(Debug, Clone, Copy)]
pub enum Boolean {}
/// An integer, float, decimal, or quantity term.
#[derive(Debug, Clone, Copy)]
pub enum Numeric {}
/// A String term.
#[derive(Debug, Clone, Copy)]
pub enum Textual {}
/// A term whose type only admission knows: names, fields, and calls.
#[derive(Debug, Clone, Copy)]
pub enum Dynamic {}

impl sealed::Sealed for Boolean {}
impl sealed::Sealed for Numeric {}
impl sealed::Sealed for Textual {}
impl sealed::Sealed for Dynamic {}
impl TermKind for Boolean {}
impl TermKind for Numeric {}
impl TermKind for Textual {}
impl TermKind for Dynamic {}

/// Kinds with a language ordering.
pub trait Ordered: TermKind {}
impl Ordered for Numeric {}
impl Ordered for Textual {}
impl Ordered for Dynamic {}

/// A handle to one term in the builder that made it.
#[derive(Debug, Clone, Copy)]
pub struct Term<K: TermKind> {
    builder: u64,
    node: NodeId,
    kind: PhantomData<K>,
}

impl<K: TermKind> Term<K> {
    /// Forgets the static kind.
    pub fn dynamic(self) -> Term<Dynamic> {
        self.retype()
    }

    fn retype<J: TermKind>(self) -> Term<J> {
        Term {
            builder: self.builder,
            node: self.node,
            kind: PhantomData,
        }
    }
}

impl Term<Dynamic> {
    /// Claims a Bool; admission checks the claim.
    pub fn boolean(self) -> Term<Boolean> {
        self.retype()
    }

    /// Claims a numeric type; admission checks the claim.
    pub fn numeric(self) -> Term<Numeric> {
        self.retype()
    }

    /// Claims a String; admission checks the claim.
    pub fn textual(self) -> Term<Textual> {
        self.retype()
    }
}

/// Builds one expression draft. Terms may be reused; `finish` expands them
/// into a tree exactly as the equivalent source would parse.
#[derive(Debug)]
pub struct ExpressionBuilder {
    id: u64,
    tree: SyntaxTree,
    error: Option<ExpressionDenial>,
}

impl Default for ExpressionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionBuilder {
    pub fn new() -> Self {
        Self {
            id: NEXT_BUILDER.fetch_add(1, Ordering::Relaxed),
            tree: SyntaxTree::new(false),
            error: None,
        }
    }

    pub fn boolean(&mut self, value: bool) -> Term<Boolean> {
        self.push(SyntaxNode::Bool(value))
    }

    /// An Int64 literal; a negative value is a negated literal, as in source.
    pub fn int(&mut self, value: i64) -> Term<Numeric> {
        let magnitude = self.push(SyntaxNode::Integer(u128::from(value.unsigned_abs())));
        if value < 0 {
            self.negate(magnitude)
        } else {
            magnitude
        }
    }

    /// A Float64 literal with the exact decimal of the shortest rendering
    /// that round-trips `value`, so source `2.5` and `float(2.5)` agree.
    pub fn float(&mut self, value: f64) -> Term<Numeric> {
        let Some(decimal) = DecimalText::from_f64(value.abs()) else {
            self.fail(ExpressionDenialDetail::InvalidValue(
                "float literal is not finite",
            ));
            return self.push(SyntaxNode::Bool(false));
        };
        let magnitude = self.push(SyntaxNode::Float(decimal));
        if value.is_sign_negative() && value != 0.0 {
            self.negate(magnitude)
        } else {
            magnitude
        }
    }

    pub fn text(&mut self, value: &str) -> Term<Textual> {
        self.push(SyntaxNode::String(value.into()))
    }

    /// An operand, binder, parameter, or `Enum::Variant` reference.
    pub fn name(&mut self, name: &str) -> Term<Dynamic> {
        let name = self.qualified(name);
        self.push(SyntaxNode::Name(name))
    }

    pub fn field<K: TermKind>(&mut self, base: Term<K>, field: &str) -> Term<Dynamic> {
        let base = self.own(base);
        let field = self.identifier(field);
        self.push(SyntaxNode::Field { base, field })
    }

    /// Expands the terms reachable from `root` into a tree within
    /// engineering node and depth ceilings; admission re-checks the draft
    /// against the profile it admits under.
    pub fn finish<K: TermKind>(self, root: Term<K>) -> ExpressionResult<ExpressionDraft> {
        let profile = &ExpressionProfile::engineering();
        if let Some(error) = self.error {
            return Err(error);
        }
        if root.builder != self.id {
            return Err(foreign_term());
        }
        let mut tree = SyntaxTree::new(false);
        // Post-order expansion: (node, children already emitted).
        let mut stack = vec![(root.node, 0_usize)];
        let mut emitted: Vec<NodeId> = Vec::new();
        while let Some(&(node, next)) = stack.last() {
            let children = self.tree.node(node).children();
            if let Some(child) = children.get(next) {
                let top = stack.len() - 1;
                stack[top].1 += 1;
                stack.push((*child, 0));
                continue;
            }
            stack.pop();
            let mut fresh = emitted
                .split_off(emitted.len() - children.len())
                .into_iter();
            let expanded = self
                .tree
                .node(node)
                .map_children(|_| fresh.next().expect("one per child"));
            check_limit(
                profile,
                ExpressionResource::SyntaxNodes,
                tree.len() as u64 + 1,
            )?;
            let (id, depth) = tree.push(expanded, None);
            check_limit(profile, ExpressionResource::SyntaxDepth, u64::from(depth))?;
            emitted.push(id);
        }
        Ok(ExpressionDraft::from_tree(tree))
    }

    fn push<K: TermKind>(&mut self, node: SyntaxNode) -> Term<K> {
        let (node, _) = self.tree.push(node, None);
        Term {
            builder: self.id,
            node,
            kind: PhantomData,
        }
    }

    /// The node behind `term`, recording an error for a foreign term.
    fn own<K: TermKind>(&mut self, term: Term<K>) -> NodeId {
        if term.builder == self.id {
            term.node
        } else {
            self.error.get_or_insert_with(foreign_term);
            // A local node keeps the arena well formed until `finish` denies.
            self.push::<Dynamic>(SyntaxNode::Bool(false)).node
        }
    }

    fn fail(&mut self, detail: ExpressionDenialDetail) {
        self.error
            .get_or_insert_with(|| ExpressionDenial::new(detail));
    }

    fn identifier(&mut self, text: &str) -> Box<str> {
        if !is_identifier(text) {
            self.fail(ExpressionDenialDetail::Syntax(
                SyntaxDenial::InvalidIdentifier,
            ));
        }
        text.into()
    }

    fn qualified(&mut self, text: &str) -> QualifiedName {
        QualifiedName(
            text.split("::")
                .map(|segment| self.identifier(segment))
                .collect(),
        )
    }
}

fn foreign_term() -> ExpressionDenial {
    ExpressionDenial::new(ExpressionDenialDetail::UnsupportedFeature(
        "a term from another builder was used",
    ))
}
