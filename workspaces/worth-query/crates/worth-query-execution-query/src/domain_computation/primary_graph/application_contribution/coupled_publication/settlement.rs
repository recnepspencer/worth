//! R3 settlement: a done consumer of a reopened derivation keeps its
//! consumption when the derivation's replacement publishes exactly what it
//! read (Query half: route, role, entity and demand key; Native half: roles,
//! entities, generations, custody and payload). Otherwise it is reopened.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::synchronous_outputs::AppliedOutputParent;
use crate::domain_computation::primary_graph::provider::ParentRead;

impl<S: ApplicationSchema> CoupledApplicationAssembly<'_, '_, '_, S> {
    /// Settle `consumer`'s consumption of `previous` to `replacement`, the
    /// same route's later round. Returns whether it settled.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn settle_consumer(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        consumer: &AppliedOutputParent,
        previous: &AppliedOutputParent,
        replacement: &AppliedOutputParent,
    ) -> Result<bool, CoupledSourceReadStop> {
        let (Some(consumer_input), Some(previous_input), Some(replacement_input)) =
            (consumer.input(), previous.input(), replacement.input())
        else {
            return Ok(false);
        };
        let (Some(consumer_read), Some(replacement_read)) = (consumer.read(), replacement.read())
        else {
            return Ok(false);
        };
        let provider = &self.application.runtime.primary_provider;
        let settles = |settle: bool| {
            let writers = self.writers.borrow();
            let session = |read: ParentRead| {
                writers
                    .iter()
                    .find(|writer| writer.produces == read)
                    .and_then(|writer| match writer.stage.as_ref() {
                        Some(CoupledWriterStage::Inspected(writer)) => Some(writer.session()),
                        _ => None,
                    })
            };
            match (session(consumer_read), session(replacement_read)) {
                (Some(consumer), Some(replacement)) => {
                    provider.settle_consumed_output(consumer, replacement, settle)
                }
                _ => false,
            }
        };
        if !settles(false)
            || !candidate.settle_output_input(
                execution,
                consumer_input,
                previous.pin(),
                previous_input,
                replacement_input,
            )?
        {
            return Ok(false);
        }
        Ok(settles(true))
    }
}
