//! Pure admission: name resolution, bidirectional typing, literal folding,
//! and canonical linearization of a syntax tree into a typed program.
//!
//! Recursion follows the syntax tree, whose depth is checked against the
//! profile before checking starts, so hostile nesting denies before it can
//! exhaust the native stack.

mod bitwise;
mod calls;
mod casts;
mod collections;
mod constructors;
mod linearize;
mod names;
mod operators;

use std::collections::BTreeSet;

use super::denial::{
    ExpressionDenial, ExpressionDenialDetail, ExpressionOccurrence, ExpressionResource,
    ExpressionResult,
};
use super::functions::ExpressionFunctionCatalog;
use super::profile::{check_limit, AdmissionMeter, ExpressionProfile};
use super::program::{ExpressionProgram, Literal, Op, ProgramNode};
use super::syntax::ast::{NodeId, SyntaxNode, SyntaxTree};
use super::syntax::SourceOrigin;
use super::types::{resolve_type, ExpressionSchema, ExpressionType};

/// What names resolve against beyond binders.
#[derive(Clone, Copy)]
pub(crate) enum Scope<'a> {
    /// A top-level expression over the schema's operands.
    Operands,
    /// An installed function body over its declared parameters.
    Parameters(&'a [(Box<str>, ExpressionType)]),
}

pub(crate) struct AdmissionContext<'a> {
    pub(crate) schema: &'a ExpressionSchema,
    pub(crate) catalog: &'a ExpressionFunctionCatalog,
    pub(crate) profile: &'a ExpressionProfile,
}

/// A checked program with its operand slots and direct function closure.
pub(crate) struct CheckedProgram {
    pub(crate) program: ExpressionProgram,
    /// Slot order: canonical operand-name order over used operands, or
    /// declared parameter order for a function body.
    pub(crate) slots: Vec<(Box<str>, ExpressionType)>,
    /// Catalog indices of direct callees, in canonical closure order.
    pub(crate) functions: Vec<usize>,
    pub(crate) work: u64,
    /// Own nodes plus every call site's callee expansion.
    pub(crate) expanded_instructions: u64,
    pub(crate) call_depth: u32,
}

/// Checks `tree` against `context`; `expected` constrains the result type.
pub(crate) fn check_tree(
    tree: &SyntaxTree,
    context: &AdmissionContext<'_>,
    scope: Scope<'_>,
    expected: Option<&ExpressionType>,
) -> ExpressionResult<CheckedProgram> {
    let Some(root) = tree.root() else {
        return Err(ExpressionDenial::new(ExpressionDenialDetail::Syntax(
            super::denial::SyntaxDenial::UnexpectedEnd {
                expected: "an expression",
            },
        )));
    };
    let profile = context.profile;
    check_limit(profile, ExpressionResource::SyntaxNodes, tree.len() as u64)?;
    check_limit(
        profile,
        ExpressionResource::SyntaxDepth,
        u64::from(tree.depth(root)),
    )?;
    let mut checker = Checker {
        tree,
        context,
        scope,
        meter: AdmissionMeter::new(profile, ExpressionResource::AdmissionWork),
        binders: Vec::new(),
        nodes: Vec::new(),
        operands: BTreeSet::new(),
        functions: BTreeSet::new(),
    };
    let root = checker.check(root, expected)?;
    linearize::finish(checker, root)
}

pub(super) struct Checker<'a> {
    tree: &'a SyntaxTree,
    context: &'a AdmissionContext<'a>,
    scope: Scope<'a>,
    meter: AdmissionMeter,
    /// Innermost binder last.
    binders: Vec<(Box<str>, ExpressionType)>,
    /// Checked nodes in check order; linearization makes the order canonical.
    nodes: Vec<ProgramNode>,
    /// Schema operand indices, or parameter indices, the program reads.
    operands: BTreeSet<usize>,
    /// Catalog indices the program calls directly.
    functions: BTreeSet<usize>,
}

impl Checker<'_> {
    fn schema(&self) -> &ExpressionSchema {
        self.context.schema
    }

    fn occurrence(&self, id: NodeId) -> ExpressionOccurrence {
        match self.tree.span(id) {
            Some(span) => ExpressionOccurrence::Source(span),
            None => ExpressionOccurrence::Node(id.0),
        }
    }

    fn deny(&self, id: NodeId, detail: ExpressionDenialDetail) -> ExpressionDenial {
        ExpressionDenial::at(detail, self.occurrence(id))
    }

    fn ty(&self, index: u32) -> &ExpressionType {
        &self.nodes[index as usize].ty
    }

    fn emit(
        &mut self,
        id: NodeId,
        op: Op,
        ty: ExpressionType,
        children: Vec<u32>,
    ) -> ExpressionResult<u32> {
        self.meter.charge(1 + children.len() as u64)?;
        let origin = SourceOrigin::new(self.tree.span(id), id.0);
        self.nodes.push(ProgramNode {
            op,
            ty,
            children: children.into_boxed_slice(),
            origin,
        });
        Ok((self.nodes.len() - 1) as u32)
    }

    fn literal(
        &mut self,
        id: NodeId,
        literal: Literal,
        ty: ExpressionType,
    ) -> ExpressionResult<u32> {
        self.emit(id, Op::Literal(literal), ty, Vec::new())
    }

    /// Checks `id`, denying a result type other than `expected`.
    fn check(&mut self, id: NodeId, expected: Option<&ExpressionType>) -> ExpressionResult<u32> {
        let index = self
            .check_node(id, expected)
            .map_err(|denial| denial.with_occurrence(self.occurrence(id)))?;
        match expected {
            Some(expected) if self.ty(index) != expected => Err(self.mismatch(id, expected, index)),
            _ => Ok(index),
        }
    }

    fn mismatch(&self, id: NodeId, expected: &ExpressionType, found: u32) -> ExpressionDenial {
        self.deny(
            id,
            ExpressionDenialDetail::TypeMismatch {
                expected: expected.to_string(),
                found: self.ty(found).to_string(),
            },
        )
    }

    /// Whether `id` can be typed only from an expected type.
    fn needs_expected(&self, id: NodeId) -> bool {
        match self.tree.node(id) {
            SyntaxNode::List(items) => items.is_empty(),
            SyntaxNode::Map(entries) => entries.is_empty(),
            SyntaxNode::Conditional {
                then, otherwise, ..
            } => self.needs_expected(*then) && self.needs_expected(*otherwise),
            SyntaxNode::Let { body, .. } => self.needs_expected(*body),
            _ => false,
        }
    }

    fn check_node(
        &mut self,
        id: NodeId,
        expected: Option<&ExpressionType>,
    ) -> ExpressionResult<u32> {
        match self.tree.node(id) {
            SyntaxNode::Bool(value) => {
                self.literal(id, Literal::Bool(*value), ExpressionType::Bool)
            }
            SyntaxNode::Integer(value) => {
                let value = i64::try_from(*value).map_err(|_| {
                    self.deny(
                        id,
                        ExpressionDenialDetail::InvalidValue("Int64 literal out of range"),
                    )
                })?;
                self.literal(id, Literal::Integer(value.into()), ExpressionType::INT64)
            }
            SyntaxNode::Float(decimal) => {
                let value = decimal.to_f64().ok_or_else(|| {
                    self.deny(
                        id,
                        ExpressionDenialDetail::InvalidValue("float literal is not finite"),
                    )
                })?;
                self.literal(
                    id,
                    Literal::Float64(value.to_bits()),
                    ExpressionType::Float64,
                )
            }
            SyntaxNode::String(text) => {
                self.literal(id, Literal::String(text.clone()), ExpressionType::String)
            }
            SyntaxNode::Name(name) => self.name(id, name),
            SyntaxNode::Field { base, field } => self.field(id, *base, field),
            SyntaxNode::Comprehension {
                base,
                kind,
                binder,
                body,
            } => self.comprehension(id, *base, *kind, binder, *body),
            SyntaxNode::Unary { op, operand } => self.unary(id, *op, *operand),
            SyntaxNode::Binary { op, left, right } => self.binary(id, *op, *left, *right),
            SyntaxNode::Conditional {
                condition,
                then,
                otherwise,
            } => self.conditional(id, [*condition, *then, *otherwise], expected),
            SyntaxNode::Let {
                binder,
                value,
                body,
            } => self.let_binding(id, binder, *value, *body, expected),
            SyntaxNode::Call {
                function,
                type_arguments,
                arguments,
            } => self.call(id, function, type_arguments, arguments),
            SyntaxNode::None(syntax) => {
                let inner = resolve_type(self.schema(), syntax, id.0)?;
                self.literal(id, Literal::None, ExpressionType::option(inner))
            }
            SyntaxNode::List(items) => self.list(id, items, expected),
            SyntaxNode::Map(entries) => self.map(id, entries, expected),
            SyntaxNode::Record { type_name, fields } => self.record(id, type_name, fields),
        }
    }
}
