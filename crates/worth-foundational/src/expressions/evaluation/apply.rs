//! Applying a node to its evaluated children.
//!
//! Strict operators consume their operands whole. Field access passes the
//! origin through, so a later use records only the field it touched, and list
//! and record constructors keep their parts' origins, so taking one part back
//! out reads nothing else.

use std::sync::Arc;

use crate::expressions::denial::ExpressionResult;
use crate::expressions::program::Op;
use crate::expressions::syntax::ast::{BinaryOp, UnaryOp};

use super::jobs::Job;
use super::machine::{Applied, Held, Machine};
use super::maps::MapSort;
use super::numbers;
use super::paths::Origin;
use super::value::ExpressionValue;

/// Admission fixed every node's arity.
pub(super) fn take<const N: usize>(arguments: Vec<Held>) -> [Held; N] {
    arguments
        .try_into()
        .unwrap_or_else(|_| unreachable!("admission fixes each operation's arity"))
}

/// The origin of a constructed value; parts that are all computed need no
/// tracking.
pub(super) fn parts(origins: Vec<Origin>) -> Origin {
    if origins
        .iter()
        .all(|origin| matches!(origin, Origin::Computed))
    {
        Origin::Computed
    } else {
        Origin::Parts(Arc::from(origins))
    }
}

impl Machine {
    pub(super) fn apply(&mut self, node: u32, arguments: Vec<Held>) -> ExpressionResult<Applied> {
        let compiled = self.compiled.clone();
        let index = self.activations.last().map_or(0, |call| call.routine);
        let routine = &compiled.routines[index as usize];
        let program_node = &routine.program.nodes()[node as usize];
        let ty = &program_node.ty;
        let value = match &program_node.op {
            Op::Unary(op) => {
                let [operand] = take(arguments);
                self.runtime.consume(&operand)?;
                match op {
                    UnaryOp::Not => ExpressionValue::bool(operand.value.as_bool() != Some(true)),
                    UnaryOp::Negate => numbers::negate(ty, &operand.value)?,
                }
            }
            Op::Binary(op) => {
                let [left, right] = take(arguments);
                self.runtime.consume(&left)?;
                self.runtime.consume(&right)?;
                if matches!(
                    op,
                    BinaryOp::Less
                        | BinaryOp::LessEqual
                        | BinaryOp::Greater
                        | BinaryOp::GreaterEqual
                        | BinaryOp::Equal
                        | BinaryOp::NotEqual
                ) {
                    return Ok(Applied::Job(Job::compare(*op, left.value, right.value)));
                }
                numbers::arithmetic(*op, ty, &left.value, &right.value)?
            }
            Op::Field(field) => return self.field(arguments, *field).map(Applied::Ready),
            Op::List | Op::Record => {
                let items = arguments.len() as u64;
                self.runtime.meter.allocate(16 + 8 * items)?;
                let (values, origins): (Vec<_>, Vec<_>) = arguments
                    .into_iter()
                    .map(|held| (held.value, held.origin))
                    .unzip();
                let value = if program_node.op == Op::List {
                    ExpressionValue::list(values)
                } else {
                    ExpressionValue::record(values)
                };
                return Ok(Applied::Ready(Held {
                    value,
                    origin: parts(origins),
                }));
            }
            Op::Map => {
                // Sorting reads every key whole; values keep their origins.
                for key in arguments.iter().step_by(2) {
                    self.runtime.consume(key)?;
                }
                return Ok(Applied::Job(Job::Sort(Box::new(MapSort::new(arguments)))));
            }
            Op::Call(callee) => {
                return Ok(Applied::Call {
                    routine: routine.calls[*callee as usize],
                    arguments: arguments.into_boxed_slice(),
                });
            }
            Op::Builtin(builtin) => {
                let types: Vec<_> = program_node
                    .children
                    .iter()
                    .map(|child| &routine.program.nodes()[*child as usize].ty)
                    .collect();
                return self.builtin(*builtin, ty, &types, arguments);
            }
            Op::Literal(_)
            | Op::Operand(_)
            | Op::Local(_)
            | Op::Conditional
            | Op::Let
            | Op::Comprehension(_) => unreachable!("evaluated without an apply step"),
        };
        Ok(Applied::Ready(Held::computed(value)))
    }

    fn field(&mut self, arguments: Vec<Held>, field: u32) -> ExpressionResult<Held> {
        let [record] = take(arguments);
        self.runtime.meter.read();
        let items = record
            .value
            .items()
            .expect("admission reads fields of records");
        let value = items[field as usize].clone();
        let origin = match &record.origin {
            Origin::Path(path) => {
                let field = self
                    .runtime
                    .recorder
                    .field(*path, field, &mut self.runtime.meter)?;
                Origin::Path(field)
            }
            Origin::Parts(parts) => parts[field as usize].clone(),
            Origin::Computed => Origin::Computed,
        };
        Ok(Held { value, origin })
    }
}
