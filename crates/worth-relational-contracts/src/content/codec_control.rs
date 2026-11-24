pub use super::denial::RelationalContentDenial as Denial;
use crate::execution::request_stage::checkpoint;
use crate::validation::custom_rule::work_meter::native_rule_debit::{
    CustomInvariantNativeCheckpointDenial as Stop, NativeRuleDebit,
};
use crate::validation::data::structural_walk_denial::StructuralWalkDenialKind;
use worth_execution::{ExecutionRequest, MapKernelContext};
#[cfg(test)]
mod tests;

// Only the SDK's generic nested-stop sentinel may yield to a classified inner
// refusal. A concrete original-request stop is authoritative at final handoff.
pub fn final_denial(inner: Option<Denial>, outer: Denial) -> Denial {
    match (inner, outer) {
        (
            Some(inner),
            Denial::Execution(
                crate::execution::denial_cause::RelationalExecutionDenialCause::NestedStopped,
            ),
        ) => inner,
        (_, outer) => outer,
    }
}

/// Borrowed original controls; codecs cannot replace this account or its first stop.
pub struct RelationalContentControl<'borrow, 'scope, 'lease> {
    context: &'borrow mut MapKernelContext<'scope, 'lease>,
    first_stop: Option<Denial>,
    source: Option<(ExecutionRequest<'borrow, 'borrow>, NativeRuleDebit<'borrow>)>,
    source_stop: Option<StructuralWalkDenialKind>,
}

impl<'borrow, 'scope, 'lease> RelationalContentControl<'borrow, 'scope, 'lease> {
    pub fn checkpoint(&mut self, units: u64) -> Result<(), Denial> {
        if let Some(cause) = &self.first_stop {
            return Err(*cause);
        }
        let result = if let Some((execution, source)) = &self.source {
            match source.checkpoint(*execution, self.context, units) {
                Ok(()) => Ok(()),
                Err(Stop::Request(cause)) => Err(cause),
                Err(Stop::Source(cause)) => {
                    self.source_stop = Some(cause);
                    self.first_stop = Some(Denial::EncodingInconsistency);
                    return Err(Denial::EncodingInconsistency);
                }
            }
        } else {
            checkpoint(self.context, units)
        };
        match result {
            Ok(()) => Ok(()),
            Err(cause) => {
                let cause = Denial::Execution(cause.into());
                self.first_stop = Some(cause);
                Err(cause)
            }
        }
    }
}

/// Operations used by the Relational owners.
pub trait RelationalContentControlOwnerPort<'borrow, 'scope, 'lease>: Sized {
    fn new(context: &'borrow mut MapKernelContext<'scope, 'lease>) -> Self;
    fn attach_source(
        &mut self,
        execution: ExecutionRequest<'borrow, 'borrow>,
        source: NativeRuleDebit<'borrow>,
    );
    fn source_stop(&self) -> Option<StructuralWalkDenialKind>;
}

impl<'borrow, 'scope, 'lease> RelationalContentControlOwnerPort<'borrow, 'scope, 'lease>
    for RelationalContentControl<'borrow, 'scope, 'lease>
{
    fn new(context: &'borrow mut MapKernelContext<'scope, 'lease>) -> Self {
        Self {
            context,
            first_stop: None,
            source: None,
            source_stop: None,
        }
    }

    fn attach_source(
        &mut self,
        execution: ExecutionRequest<'borrow, 'borrow>,
        source: NativeRuleDebit<'borrow>,
    ) {
        self.source = Some((execution, source));
    }

    fn source_stop(&self) -> Option<StructuralWalkDenialKind> {
        self.source_stop
    }
}
