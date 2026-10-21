//! Reuse admitted source inputs, retaining each complete retirement batch on the request.
use super::{
    ApplicationMutationBinding, ApplicationSchema, DecisionReader, HandlerExecutionDenial,
};
use crate::domain_computation::primary_graph::{
    operation_body::{
        BodyFieldExpectation, PreparedBodyInputs, PreparedBodyRetirements, PreparedRetirementBatch,
    },
    HandlerResult, WorthQueryAdvancementDenial, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryManagedComputationResourceDenial,
    WorthQueryOperationBodyDenial,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram;
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, '_, '_, S, B> {
    pub(in crate::domain_computation::primary_graph) fn body_retirements(
        &mut self,
        body: &ApplicationOperationBodyProgram,
        indices: &[usize],
        inputs: Option<&PreparedBodyInputs>,
    ) -> HandlerResult<PreparedBodyRetirements<S>, WorthQueryOperationBodyDenial> {
        match self.prepare_retirements(body, indices, inputs) {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn prepare_retirements(
        &mut self,
        body: &ApplicationOperationBodyProgram,
        indices: &[usize],
        inputs: Option<&PreparedBodyInputs>,
    ) -> Result<PreparedBodyRetirements<S>, HandlerExecutionDenial> {
        if body.retirements().is_empty() {
            return Ok(PreparedBodyRetirements {
                batches: Vec::new(),
                _memory: None,
            });
        }
        let bytes = body
            .retirements()
            .len()
            .checked_mul(size_of::<PreparedRetirementBatch<S>>())
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(overflow)?;
        let memory = self.execution.reserve(bytes).map_err(resource)?;
        let mut batches = Vec::with_capacity(body.retirements().len());
        for (index, (rule, input_index)) in body.retirements().iter().zip(indices).enumerate() {
            let value = inputs
                .and_then(|inputs| inputs.values.get(*input_index))
                .and_then(Option::as_ref)
                .expect("installed retirement source is an admitted required scalar write");
            if self.body_scope_field(&rule.trigger, BodyFieldExpectation::Equals(value))? {
                continue;
            }
            self.context_use.set(self.context_use.get().scope());
            let execution = self.execution;
            let batch = execution
                .decision_stage(|context| {
                    self.reader
                        .body_retirement(self.scope, rule, index, execution, context)
                })
                .map_err(HandlerExecutionDenial::new)??;
            batches.push(batch);
        }
        reject_overlap(self.execution, &batches)?;
        Ok(PreparedBodyRetirements {
            batches,
            _memory: Some(memory),
        })
    }
}
fn reject_overlap<S>(
    execution:&crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution<'_>,
    batches: &[PreparedRetirementBatch<S>],
) -> Result<(), HandlerExecutionDenial> {
    let count = batches
        .iter()
        .try_fold(0usize, |sum, batch| sum.checked_add(batch.targets.len()))
        .ok_or_else(overflow)?;
    let bytes = count
        .checked_mul(size_of::<worth_relational::facade::identity::EntityId>())
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(overflow)?;
    let _memory = execution.reserve(bytes).map_err(resource)?;
    execution.decision_stage(|context|{
        let work=u64::try_from(count).ok().and_then(|n|n.checked_mul(u64::from(usize::BITS-count.leading_zeros())+1)).ok_or_else(overflow)?;
        context.checkpoint(work).map_err(|cause|HandlerExecutionDenial::new(crate::domain_computation::primary_graph::application_contribution::advancement_denial(worth_execution::WorkCeilingDenial::Stopped(cause))))?;
        let mut selected=Vec::with_capacity(count);
        selected.extend(batches.iter().flat_map(|batch|batch.targets.iter().map(|(_,target)|target.entity_id)));
        selected.sort_unstable();
        if selected.windows(2).any(|pair|pair[0]==pair[1]){return Err(HandlerExecutionDenial::new(WorthQueryApplicationAttemptDenial::new(WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep,"overlapping owned selection retirements")));}
        Ok(())
    }).map_err(HandlerExecutionDenial::new)?
}
fn resource(cause: WorthQueryManagedComputationResourceDenial) -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
}
fn overflow() -> HandlerExecutionDenial {
    resource(WorthQueryManagedComputationResourceDenial::CapacityOverflow)
}
