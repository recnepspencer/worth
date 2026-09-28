//! Lowering admitted meaning into an immutable evaluation plan: one routine
//! per distinct program, literals converted once, static costs precomputed,
//! and calls resolved to routine indices. Shared function bodies lower once,
//! however many callers reach them.

use std::collections::HashMap;
use std::sync::Arc;

use crate::expressions::admitted::AdmittedExpression;
use crate::expressions::canonical::ExpressionProgramIdentity;
use crate::expressions::functions::InstalledExpressionFunction;
use crate::expressions::profile::ExpressionProfile;
use crate::expressions::program::{ExpressionProgram, Literal, Op};
use crate::expressions::types::{ExpressionSchema, ExpressionType};

use super::cost::static_cost;
use super::value::{ExpressionValue, Repr};

#[derive(Debug)]
pub(super) struct Routine {
    pub(super) program: ExpressionProgram,
    /// Each literal node's value; `None` for other nodes.
    pub(super) literals: Box<[Option<ExpressionValue>]>,
    /// Extra work each node charges before it runs.
    pub(super) costs: Box<[u64]>,
    /// Routine index of each closure entry.
    pub(super) calls: Box<[u32]>,
}

impl Routine {
    pub(super) fn root(&self) -> u32 {
        self.program.nodes().len() as u32 - 1
    }
}

/// An immutable, discardable plan. Routine 0 is the expression itself.
#[derive(Debug)]
pub(super) struct Compiled {
    pub(super) routines: Box<[Routine]>,
    pub(super) slots: Box<[(Box<str>, ExpressionType)]>,
    pub(super) declarations: ExpressionSchema,
    pub(super) profile: ExpressionProfile,
    pub(super) identity: Arc<ExpressionProgramIdentity>,
}

/// Assigns routine indices to functions in first-reached order.
#[derive(Default)]
struct Functions {
    index: HashMap<usize, u32>,
    /// Holds every reached function alive, so no address is reused.
    order: Vec<Arc<InstalledExpressionFunction>>,
}

impl Functions {
    fn calls<'a>(
        &mut self,
        closure: impl Iterator<Item = &'a Arc<InstalledExpressionFunction>>,
    ) -> Box<[u32]> {
        closure
            .map(|function| {
                let key = Arc::as_ptr(function) as usize;
                let next = self.order.len() as u32 + 1;
                *self.index.entry(key).or_insert_with(|| {
                    self.order.push(function.clone());
                    next
                })
            })
            .collect()
    }
}

pub(super) fn compile(admitted: &AdmittedExpression) -> Compiled {
    let mut functions = Functions::default();
    let calls = functions.calls(admitted.functions());
    let mut routines = vec![routine(admitted.program(), calls)];
    while routines.len() <= functions.order.len() {
        let function = functions.order[routines.len() - 1].clone();
        let calls = functions.calls(function.functions());
        routines.push(routine(function.program(), calls));
    }
    Compiled {
        routines: routines.into_boxed_slice(),
        slots: admitted
            .slots()
            .map(|(name, ty)| (Box::from(name), ty.clone()))
            .collect(),
        declarations: admitted.declarations().clone(),
        profile: *admitted.profile(),
        identity: Arc::new(admitted.identity().clone()),
    }
}

fn routine(program: &ExpressionProgram, calls: Box<[u32]>) -> Routine {
    let nodes = program.nodes();
    Routine {
        literals: nodes
            .iter()
            .map(|node| match &node.op {
                Op::Literal(value) => Some(literal(value, &node.ty)),
                _ => None,
            })
            .collect(),
        costs: nodes
            .iter()
            .map(|node| static_cost(program, node))
            .collect(),
        calls,
        program: program.clone(),
    }
}

fn width(ty: &ExpressionType) -> u32 {
    match ty {
        ExpressionType::Bits(width) | ExpressionType::Logic4(width) => *width,
        _ => unreachable!("bus literals have bus types"),
    }
}

fn literal(literal: &Literal, ty: &ExpressionType) -> ExpressionValue {
    ExpressionValue(match literal {
        Literal::Bool(value) => Repr::Bool(*value),
        Literal::Integer(value) => Repr::Integer(*value),
        Literal::Float32(bits) => Repr::Float32(*bits),
        Literal::Float64(bits) => Repr::Float64(*bits),
        Literal::Decimal { coefficient, scale } => Repr::Decimal {
            coefficient: *coefficient,
            scale: *scale,
        },
        Literal::String(text) => Repr::String(Arc::from(&**text)),
        Literal::Bytes(bytes) => Repr::Bytes(Arc::from(&**bytes)),
        Literal::Enum(index) => Repr::Enum(*index),
        Literal::Quantity(bits) => Repr::Quantity(*bits),
        Literal::Bits(limbs) => Repr::Bits {
            width: width(ty),
            limbs: Arc::from(&**limbs),
        },
        Literal::Logic4 { value, unknown } => Repr::Logic4 {
            width: width(ty),
            value: Arc::from(&**value),
            unknown: Arc::from(&**unknown),
        },
        Literal::None => Repr::None,
    })
}
