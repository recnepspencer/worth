//! The explicit-stack evaluation machine.
//!
//! Frames, values, binders, and call activations live on heap stacks, so
//! evaluation depth never becomes native recursion. Each frame runs one
//! charged step: a step that cannot pay pushes its frame back, which leaves
//! the machine a complete, resumable value at every suspension point.

use std::sync::Arc;
use std::task::Poll;

use crate::expressions::denial::{
    ExpressionDenial, ExpressionDenialDetail, ExpressionOccurrence, ExpressionResult,
};
use crate::expressions::program::{Op, ProgramNode};
use crate::expressions::syntax::ast::BinaryOp;

use super::compile::{Compiled, Routine};
use super::comprehension::Iteration;
use super::jobs::Job;
use super::meter::EvaluationMeter;
use super::paths::{ExpressionReadKind, Origin, Recorder};
use super::value::ExpressionValue;

/// A value and where it came from.
#[derive(Debug, Clone)]
pub(super) struct Held {
    pub(super) value: ExpressionValue,
    pub(super) origin: Origin,
}

impl Held {
    pub(super) fn computed(value: ExpressionValue) -> Self {
        Self {
            value,
            origin: Origin::Computed,
        }
    }
}

/// The meter and recorder every step charges.
#[derive(Debug)]
pub(super) struct Runtime {
    pub(super) meter: EvaluationMeter,
    pub(super) recorder: Recorder,
}

impl Runtime {
    /// Records that the whole of `held` was used.
    pub(super) fn consume(&mut self, held: &Held) -> ExpressionResult<()> {
        self.recorder
            .consume(&held.value, &held.origin, &mut self.meter)
    }

    /// Records a `kind` read of `held` when it is an operand path.
    pub(super) fn read(&mut self, held: &Held, kind: ExpressionReadKind) -> ExpressionResult<()> {
        self.recorder
            .read_origin(&held.origin, kind, &held.value, &mut self.meter)
    }
}

#[derive(Debug)]
pub(super) enum Frame {
    /// Visit a node.
    Eval(u32),
    /// Every child is evaluated; apply the node.
    Apply(u32),
    /// `&&`, `||`, or `??` after the left operand.
    Lazy(u32),
    /// A conditional after its condition.
    Choose(u32),
    /// A `let` after its value.
    Bind(u32),
    Unbind,
    /// A comprehension after its base, then between iterations.
    Loop(u32, Option<Box<Iteration>>),
    Job(u32, Box<Job>),
    Return,
}

impl Frame {
    fn node(&self) -> Option<u32> {
        match self {
            Self::Eval(node)
            | Self::Apply(node)
            | Self::Lazy(node)
            | Self::Choose(node)
            | Self::Bind(node)
            | Self::Loop(node, _)
            | Self::Job(node, _) => Some(*node),
            Self::Unbind | Self::Return => None,
        }
    }
}

/// One installed function call in progress.
#[derive(Debug)]
pub(super) struct Activation {
    pub(super) routine: u32,
    pub(super) arguments: Box<[Held]>,
    /// The calling node in the caller's routine.
    pub(super) site: u32,
}

/// What applying a node produced.
pub(super) enum Applied {
    Ready(Held),
    Job(Job),
    Call {
        routine: u32,
        arguments: Box<[Held]>,
    },
}

#[derive(Debug)]
pub(super) struct Machine {
    pub(super) compiled: Arc<Compiled>,
    pub(super) operands: Box<[Option<ExpressionValue>]>,
    pub(super) frames: Vec<Frame>,
    pub(super) values: Vec<Held>,
    pub(super) locals: Vec<Held>,
    pub(super) activations: Vec<Activation>,
    pub(super) runtime: Runtime,
}

impl Machine {
    pub(super) fn new(
        compiled: Arc<Compiled>,
        operands: Box<[Option<ExpressionValue>]>,
        meter: EvaluationMeter,
    ) -> Self {
        let root = compiled.routines[0].root();
        Self {
            compiled,
            operands,
            frames: vec![Frame::Eval(root)],
            values: Vec::new(),
            locals: Vec::new(),
            activations: Vec::new(),
            runtime: Runtime {
                meter,
                recorder: Recorder::default(),
            },
        }
    }

    /// Runs one slice of at most `quantum` units: the result when evaluation
    /// completes, or `Pending` with the machine ready to resume.
    pub(super) fn run(&mut self, quantum: u64) -> Poll<ExpressionResult<ExpressionValue>> {
        self.runtime.meter.open_slice(quantum);
        while let Some(frame) = self.frames.pop() {
            let node = frame.node();
            match self.execute(frame) {
                Ok(step) if step.is_pending() => return Poll::Pending,
                Ok(_) => {}
                Err(denial) => {
                    let occurrence = node.map(|node| self.occurrence(node));
                    return Poll::Ready(Err(match occurrence {
                        Some(occurrence) => denial.with_occurrence(occurrence),
                        None => denial,
                    }));
                }
            }
        }
        Poll::Ready(self.finish())
    }

    fn finish(&mut self) -> ExpressionResult<ExpressionValue> {
        let result = self
            .values
            .pop()
            .expect("a complete program leaves its result");
        self.runtime.consume(&result)?;
        self.runtime.meter.output(result.value.logical_bytes())?;
        Ok(result.value)
    }

    /// Where a denial at `node` of the current routine occurred: inside an
    /// installed function, the outermost call site in the expression.
    fn occurrence(&self, node: u32) -> ExpressionOccurrence {
        let node = self.activations.first().map_or(node, |call| call.site);
        let origin = self.compiled.routines[0].program.nodes()[node as usize].origin;
        origin.span().map_or(
            ExpressionOccurrence::Node(origin.syntax_node()),
            ExpressionOccurrence::Source,
        )
    }

    pub(super) fn routine(&self) -> &Routine {
        let index = self.activations.last().map_or(0, |call| call.routine);
        &self.compiled.routines[index as usize]
    }

    pub(super) fn node(&self, node: u32) -> &ProgramNode {
        &self.routine().program.nodes()[node as usize]
    }

    fn execute(&mut self, frame: Frame) -> ExpressionResult<Poll<()>> {
        match frame {
            Frame::Eval(node) => self.eval(node),
            Frame::Apply(node) => self.apply_node(node),
            Frame::Lazy(node) => self.lazy(node),
            Frame::Choose(node) => {
                let condition = self.pop();
                self.runtime.consume(&condition)?;
                let chosen = if condition.value.as_bool() == Some(true) {
                    1
                } else {
                    2
                };
                let branch = self.node(node).children[chosen];
                self.frames.push(Frame::Eval(branch));
                Ok(Poll::Ready(()))
            }
            Frame::Bind(node) => {
                let value = self.pop();
                self.locals.push(value);
                let body = self.node(node).children[1];
                self.frames.extend([Frame::Unbind, Frame::Eval(body)]);
                Ok(Poll::Ready(()))
            }
            Frame::Unbind => {
                self.locals.pop();
                Ok(Poll::Ready(()))
            }
            Frame::Loop(node, iteration) => self.iterate(node, iteration),
            Frame::Job(node, mut job) => match job.step(&mut self.runtime)? {
                Poll::Ready(held) => {
                    self.values.push(held);
                    Ok(Poll::Ready(()))
                }
                Poll::Pending => {
                    self.frames.push(Frame::Job(node, job));
                    Ok(Poll::Pending)
                }
            },
            Frame::Return => {
                self.activations.pop();
                Ok(Poll::Ready(()))
            }
        }
    }

    pub(super) fn pop(&mut self) -> Held {
        self.values
            .pop()
            .expect("evaluated operands precede their use")
    }

    fn eval(&mut self, node: u32) -> ExpressionResult<Poll<()>> {
        if self.runtime.meter.work(1)?.is_pending() {
            self.frames.push(Frame::Eval(node));
            return Ok(Poll::Pending);
        }
        let compiled = self.compiled.clone();
        let index = self.activations.last().map_or(0, |call| call.routine);
        let routine = &compiled.routines[index as usize];
        let program_node = &routine.program.nodes()[node as usize];
        let children = &program_node.children;
        match &program_node.op {
            Op::Literal(_) => {
                let value = routine.literals[node as usize].clone();
                let value = value.expect("literal nodes are converted at compile");
                self.values.push(Held::computed(value));
            }
            Op::Operand(slot) => {
                let held = self.operand(*slot)?;
                self.runtime.meter.read();
                self.values.push(held);
            }
            Op::Local(index) => {
                let held = self.locals[self.locals.len() - 1 - *index as usize].clone();
                self.runtime.meter.read();
                self.values.push(held);
            }
            Op::Conditional => self
                .frames
                .extend([Frame::Choose(node), Frame::Eval(children[0])]),
            Op::Binary(BinaryOp::And | BinaryOp::Or | BinaryOp::Coalesce) => {
                self.frames
                    .extend([Frame::Lazy(node), Frame::Eval(children[0])]);
            }
            Op::Let => self
                .frames
                .extend([Frame::Bind(node), Frame::Eval(children[0])]),
            Op::Comprehension(_) => {
                self.frames
                    .extend([Frame::Loop(node, None), Frame::Eval(children[0])]);
            }
            _ => {
                self.frames.push(Frame::Apply(node));
                self.frames
                    .extend(children.iter().rev().map(|child| Frame::Eval(*child)));
            }
        }
        Ok(Poll::Ready(()))
    }

    /// Operand `slot`: a root operand path, or a function argument.
    fn operand(&mut self, slot: u32) -> ExpressionResult<Held> {
        if let Some(call) = self.activations.last() {
            return Ok(call.arguments[slot as usize].clone());
        }
        let Some(value) = self.operands[slot as usize].clone() else {
            let name = self.compiled.slots[slot as usize].0.to_string();
            return Err(ExpressionDenial::new(
                ExpressionDenialDetail::MissingOperand(name),
            ));
        };
        let path = self
            .runtime
            .recorder
            .operand(slot, &mut self.runtime.meter)?;
        Ok(Held {
            value,
            origin: Origin::Path(path),
        })
    }

    fn apply_node(&mut self, node: u32) -> ExpressionResult<Poll<()>> {
        let extra = self.routine().costs[node as usize];
        if extra > 0 && self.runtime.meter.work(extra)?.is_pending() {
            self.frames.push(Frame::Apply(node));
            return Ok(Poll::Pending);
        }
        let arity = self.node(node).children.len();
        let arguments = self.values.split_off(self.values.len() - arity);
        match self.apply(node, arguments)? {
            Applied::Ready(held) => self.values.push(held),
            Applied::Job(job) => self.frames.push(Frame::Job(node, Box::new(job))),
            Applied::Call { routine, arguments } => {
                self.runtime.meter.call();
                let root = self.compiled.routines[routine as usize].root();
                self.activations.push(Activation {
                    routine,
                    arguments,
                    site: node,
                });
                self.frames.extend([Frame::Return, Frame::Eval(root)]);
            }
        }
        Ok(Poll::Ready(()))
    }

    /// `&&` and `||` decide on a consumed left operand; `??` reads only its
    /// left operand's presence. A deciding right operand is the result.
    fn lazy(&mut self, node: u32) -> ExpressionResult<Poll<()>> {
        let left = self.pop();
        let program_node = self.node(node);
        let right = program_node.children[1];
        let Op::Binary(op) = program_node.op else {
            unreachable!("lazy frames belong to lazy operators");
        };
        let optional_result = program_node.ty == self.node(program_node.children[0]).ty;
        if op == BinaryOp::Coalesce {
            self.runtime.read(&left, ExpressionReadKind::Presence)?;
            match left.value.as_option() {
                Some(Some(inner)) if !optional_result => {
                    let origin = match &left.origin {
                        Origin::Parts(parts) => parts[0].clone(),
                        origin => origin.clone(),
                    };
                    self.values.push(Held {
                        value: inner.clone(),
                        origin,
                    });
                }
                Some(Some(_)) => self.values.push(left),
                _ => self.frames.push(Frame::Eval(right)),
            }
            return Ok(Poll::Ready(()));
        }
        self.runtime.consume(&left)?;
        let decided = left.value.as_bool() == Some(op == BinaryOp::Or);
        if decided {
            self.values.push(Held::computed(left.value));
        } else {
            self.frames.push(Frame::Eval(right));
        }
        Ok(Poll::Ready(()))
    }
}
