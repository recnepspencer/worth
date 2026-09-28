//! `map`, `filter`, `all`, and `any` over a list, one charged iteration per
//! step. Each element binds as `Local(0)` with its own path, so the body's
//! reads name the element they touched. A full scan records the list's
//! membership and order; a short-circuited one records only the prefix it
//! examined.

use std::sync::Arc;
use std::task::Poll;

use crate::expressions::denial::ExpressionResult;
use crate::expressions::program::Op;
use crate::expressions::syntax::ast::ComprehensionKind;

use super::apply::parts;
use super::machine::{Frame, Held, Machine};
use super::paths::{ExpressionReadKind, Origin};
use super::value::{Composite, ExpressionValue, Repr};

#[derive(Debug)]
pub(super) struct Iteration {
    kind: ComprehensionKind,
    base: Held,
    list: Arc<Composite>,
    /// Elements bound so far.
    index: usize,
    /// Whether the body of element `index - 1` is being evaluated.
    awaiting: bool,
    output: Vec<ExpressionValue>,
    origins: Vec<Origin>,
}

impl Machine {
    pub(super) fn iterate(
        &mut self,
        node: u32,
        iteration: Option<Box<Iteration>>,
    ) -> ExpressionResult<Poll<()>> {
        let mut iteration = match iteration {
            Some(iteration) => iteration,
            None => self.begin(node),
        };
        if iteration.awaiting {
            iteration.awaiting = false;
            let element = self.locals.pop().expect("the element is bound");
            let result = self.pop();
            if iteration.kind == ComprehensionKind::Map {
                self.runtime.meter.scratch(8)?;
                iteration.output.push(result.value);
                iteration.origins.push(result.origin);
            } else {
                self.runtime.consume(&result)?;
                let accepted = result.value.as_bool() == Some(true);
                match iteration.kind {
                    ComprehensionKind::Filter if accepted => {
                        self.runtime.meter.scratch(8)?;
                        iteration.output.push(element.value);
                        iteration.origins.push(element.origin);
                    }
                    ComprehensionKind::All if !accepted => return self.decide(&iteration, false),
                    ComprehensionKind::Any if accepted => return self.decide(&iteration, true),
                    _ => {}
                }
            }
        }
        if iteration.index == iteration.list.items.len() {
            return self.complete(*iteration);
        }
        if self.runtime.meter.work(1)?.is_pending() {
            self.frames.push(Frame::Loop(node, Some(iteration)));
            return Ok(Poll::Pending);
        }
        self.runtime.meter.visit(1)?;
        self.runtime.meter.read();
        let index = iteration.index;
        let origin = match &iteration.base.origin {
            Origin::Path(path) => {
                let element =
                    self.runtime
                        .recorder
                        .element(*path, index as u64, &mut self.runtime.meter)?;
                Origin::Path(element)
            }
            Origin::Parts(parts) => parts[index].clone(),
            Origin::Computed => Origin::Computed,
        };
        self.locals.push(Held {
            value: iteration.list.items[index].clone(),
            origin,
        });
        iteration.index += 1;
        iteration.awaiting = true;
        let body = self.node(node).children[1];
        self.frames
            .extend([Frame::Loop(node, Some(iteration)), Frame::Eval(body)]);
        Ok(Poll::Ready(()))
    }

    fn begin(&mut self, node: u32) -> Box<Iteration> {
        let Op::Comprehension(kind) = self.node(node).op else {
            unreachable!("loop frames belong to comprehensions");
        };
        let base = self.pop();
        let Repr::List(list) = &base.value.0 else {
            unreachable!("admission iterates only lists");
        };
        let list = list.clone();
        Box::new(Iteration {
            kind,
            base,
            list,
            index: 0,
            awaiting: false,
            output: Vec::new(),
            origins: Vec::new(),
        })
    }

    /// `all` or `any` decided by element `index - 1`.
    fn decide(&mut self, iteration: &Iteration, value: bool) -> ExpressionResult<Poll<()>> {
        let prefix = ExpressionReadKind::Prefix(iteration.index as u64);
        self.runtime.read(&iteration.base, prefix)?;
        self.values
            .push(Held::computed(ExpressionValue::bool(value)));
        Ok(Poll::Ready(()))
    }

    fn complete(&mut self, iteration: Iteration) -> ExpressionResult<Poll<()>> {
        self.runtime
            .read(&iteration.base, ExpressionReadKind::Length)?;
        let held = match iteration.kind {
            ComprehensionKind::All => Held::computed(ExpressionValue::bool(true)),
            ComprehensionKind::Any => Held::computed(ExpressionValue::bool(false)),
            ComprehensionKind::Map | ComprehensionKind::Filter => {
                self.runtime.meter.allocate(16)?;
                Held {
                    value: ExpressionValue::list(iteration.output),
                    origin: parts(iteration.origins),
                }
            }
        };
        self.values.push(held);
        Ok(Poll::Ready(()))
    }
}
