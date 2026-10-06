//! Pure, budgeted evaluation of admitted expressions.
//!
//! An admitted expression compiles once into an immutable plan. Each
//! evaluation binds a sealed input snapshot, runs in slices of at most
//! [`MAX_SLICE_QUANTUM`] semantic work units, and ends with a value or a typed
//! denial, the reads it consumed, and its cost. A suspended evaluation is a
//! plain machine value: resuming it continues the same meter, and where
//! slices end never changes the result, the reads, or the charged cost.

mod apply;
mod builtins;
mod buses;
mod compare;
mod compile;
mod comprehension;
mod cost;
mod inputs;
mod jobs;
mod machine;
mod maps;
mod meter;
mod numbers;
mod paths;
mod scan;
mod value;

use std::sync::Arc;
use std::task::Poll;

use super::admitted::AdmittedExpression;
use super::canonical::ExpressionProgramIdentity;
use super::denial::{ExpressionDenial, ExpressionDenialDetail, ExpressionResult};
use super::profile::ExpressionProfile;
use super::types::ExpressionType;

use compile::Compiled;
use machine::Machine;
use meter::{EvaluationMeter, COST_CONTRACT};

pub use inputs::{ExpressionInputs, ExpressionInputsBuilder};
pub use meter::{ExpressionCost, MAX_SLICE_QUANTUM};
pub use paths::{ExpressionConsumption, ExpressionPathStep, ExpressionRead, ExpressionReadKind};
pub use value::ExpressionValue;

/// An admitted expression lowered for evaluation. Cheap to clone and share.
#[derive(Debug, Clone)]
pub struct CompiledExpression(Arc<Compiled>);

impl AdmittedExpression {
    /// Lowers this expression and its function closure for evaluation.
    pub fn compile(&self) -> CompiledExpression {
        CompiledExpression(Arc::new(compile::compile(self)))
    }
}

impl CompiledExpression {
    pub fn identity(&self) -> &ExpressionProgramIdentity {
        &self.0.identity
    }

    pub fn result_type(&self) -> &ExpressionType {
        self.0.routines[0].program.result_type()
    }

    /// The operands the expression may read, in canonical name order.
    pub fn slots(&self) -> impl ExactSizeIterator<Item = (&str, &ExpressionType)> {
        self.0.slots.iter().map(|(name, ty)| (&**name, ty))
    }

    /// Binds `inputs` and returns an evaluation that has done no work yet.
    ///
    /// The effective limits are the tighter of the admitted profile and
    /// `profile`. Inputs over other type declarations, or an operand whose
    /// declared type differs from its slot, deny; an operand left unbound
    /// denies only if evaluation reads it.
    pub fn start(
        &self,
        inputs: &ExpressionInputs,
        profile: &ExpressionProfile,
    ) -> ExpressionResult<ExpressionContinuation> {
        let compiled = &self.0;
        let schema = inputs.schema();
        if !compiled.declarations.same_declarations(schema) {
            return Err(ExpressionDenial::new(
                ExpressionDenialDetail::TypeMismatch {
                    expected: "inputs over the admitted type declarations".to_string(),
                    found: "inputs over other type declarations".to_string(),
                },
            ));
        }
        let mut operands = Vec::with_capacity(compiled.slots.len());
        for (name, ty) in compiled.slots.iter() {
            match schema.operand(name) {
                Some(declared) if declared != ty => {
                    return Err(ExpressionDenial::new(
                        ExpressionDenialDetail::TypeMismatch {
                            expected: ty.to_string(),
                            found: declared.to_string(),
                        },
                    ));
                }
                _ => operands.push(inputs.value(name).cloned()),
            }
        }
        let meter = EvaluationMeter::new(&compiled.profile.meet(profile));
        let machine = Machine::new(compiled.clone(), operands.into(), meter);
        Ok(ExpressionContinuation {
            machine: Box::new(machine),
        })
    }

    /// Evaluates to completion in slices of [`MAX_SLICE_QUANTUM`] units. A
    /// binding denial is a denied evaluation that did no work.
    pub fn evaluate(
        &self,
        inputs: &ExpressionInputs,
        profile: &ExpressionProfile,
    ) -> ExpressionEvaluation {
        let mut continuation = match self.start(inputs, profile) {
            Ok(continuation) => continuation,
            Err(denial) => {
                return ExpressionEvaluation {
                    result: Err(denial),
                    consumption: ExpressionConsumption::default(),
                    cost: ExpressionCost::default(),
                    identity: self.0.identity.clone(),
                };
            }
        };
        loop {
            match continuation.step(MAX_SLICE_QUANTUM) {
                ExpressionStep::Complete(evaluation) => return evaluation,
                ExpressionStep::Suspended(next) => continuation = next,
            }
        }
    }
}

/// A suspended evaluation: sealed pure machine state and its cumulative
/// meter. It carries no authority; owners decide whether to resume it.
#[derive(Debug)]
pub struct ExpressionContinuation {
    machine: Box<Machine>,
}

/// The outcome of one slice.
#[derive(Debug)]
pub enum ExpressionStep {
    Complete(ExpressionEvaluation),
    Suspended(ExpressionContinuation),
}

impl ExpressionContinuation {
    /// Runs one slice of at most `quantum` semantic work units, clamped to
    /// `1..=MAX_SLICE_QUANTUM`.
    pub fn step(mut self, quantum: u64) -> ExpressionStep {
        match self.machine.run(quantum) {
            Poll::Pending => ExpressionStep::Suspended(self),
            Poll::Ready(result) => {
                let machine = &self.machine;
                ExpressionStep::Complete(ExpressionEvaluation {
                    result,
                    consumption: machine.runtime.recorder.finish(&machine.compiled.slots),
                    cost: machine.runtime.meter.cost(),
                    identity: machine.compiled.identity.clone(),
                })
            }
        }
    }

    /// Cost so far, across every slice run.
    pub fn cost(&self) -> ExpressionCost {
        self.machine.runtime.meter.cost()
    }
}

/// A completed evaluation: its value or denial, the operand reads it
/// depended on, its cost, and the program it evaluated.
#[derive(Debug, Clone)]
pub struct ExpressionEvaluation {
    result: ExpressionResult<ExpressionValue>,
    consumption: ExpressionConsumption,
    cost: ExpressionCost,
    identity: Arc<ExpressionProgramIdentity>,
}

impl ExpressionEvaluation {
    pub fn result(&self) -> Result<&ExpressionValue, &ExpressionDenial> {
        self.result.as_ref()
    }

    pub fn into_result(self) -> ExpressionResult<ExpressionValue> {
        self.result
    }

    /// Reads the outcome depended on. A denial depends on the reads made
    /// before it, so a denied evaluation reports them too.
    pub fn consumption(&self) -> &ExpressionConsumption {
        &self.consumption
    }

    pub fn cost(&self) -> &ExpressionCost {
        &self.cost
    }

    /// The versioned cost contract `cost` was charged under.
    pub fn cost_contract(&self) -> &'static str {
        COST_CONTRACT
    }

    pub fn identity(&self) -> &ExpressionProgramIdentity {
        &self.identity
    }
}
