//! Sealed computes travel; producer completion and publication stay on the owner.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) mod canonical_apply;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::computation::prepared_request::{
    ComputedSlot, PreparedCompute,
};

pub(in crate::domain_computation::primary_graph::application_contribution::producer) type ProducerExecutionPlan<
    'owner,
> = OwnerPlan<'owner, ProducerExecutionOutcome, ProducerExecutionStop>;

type OwnerCallback<'owner, Value, Stop> = Box<
    dyn FnOnce(Vec<ComputedSlot>, &mut InvalidationEditAdmission) -> Result<Value, Stop> + 'owner,
>;

pub(in crate::domain_computation::primary_graph) struct OwnerPlan<'owner, Value, Stop> {
    requests: Vec<PreparedCompute>,
    inputs: Option<
        crate::domain_computation::primary_graph::application_output_demand::PreparedRequiredInputs,
    >,
    complete: OwnerCallback<'owner, Value, Stop>,
}

/// The consuming cursor owns completion before successor custody. Discarding
/// a suffix closes its running claim before releasing the successor interest.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct OwnerCompletion<
    'owner,
    Value,
    Stop,
    Custody,
> {
    slots: Vec<ComputedSlot>,
    complete: Option<OwnerCallback<'owner, Value, Stop>>,
    custody: Custody,
}
impl<Value, Stop, Custody> OwnerCompletion<'_, Value, Stop, Custody> {
    fn with_slots(mut self, slots: Vec<ComputedSlot>) -> Self {
        assert!(self.slots.is_empty(), "completion receives its slots once");
        self.slots = slots;
        self
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn finish(
        mut self,
        admission: &mut InvalidationEditAdmission,
    ) -> (Result<Value, Stop>, Custody) {
        // Keep custody in self while completion runs: on an unwind the running
        // claim in the callback closes before self releases its interest.
        let complete = self.complete.take().expect("completion is consumed once");
        let outcome = complete(std::mem::take(&mut self.slots), admission);
        (outcome, self.custody)
    }
}

impl<'owner, Value: 'owner, Stop: 'owner> OwnerPlan<'owner, Value, Stop> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn immediate(
        outcome: Result<Value, Stop>,
    ) -> Self {
        Self::new(Vec::new(), move |slots, _| {
            assert!(
                slots.is_empty(),
                "an immediate owner plan has no compute slots"
            );
            outcome
        })
    }
    pub(in crate::domain_computation::primary_graph) fn new(
        requests: Vec<PreparedCompute>,
        complete: impl FnOnce(Vec<ComputedSlot>, &mut InvalidationEditAdmission) -> Result<Value, Stop>
            + 'owner,
    ) -> Self {
        Self {
            requests,
            inputs: None,
            complete: Box::new(complete),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn prepared_inputs(&self) -> Option<&crate::domain_computation::primary_graph::application_output_demand::PreparedRequiredInputs>{
        self.inputs.as_ref()
    }
    pub(in crate::domain_computation::primary_graph) fn has_compute(&self) -> bool {
        !self.requests.is_empty()
    }

    pub(in crate::domain_computation::primary_graph) fn with_inputs(
        mut self,
        inputs: crate::domain_computation::primary_graph::application_output_demand::PreparedRequiredInputs,
    ) -> Self {
        self.inputs = Some(inputs);
        self
    }

    /// Align owner completion with its successor custody until canonical apply.
    /// Discarding a plan closes its invocation before releasing its replacement.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn complete_with<
        Next: 'owner,
        NextStop: 'owner,
        Custody: 'owner,
    >(
        self,
        custody: Custody,
        apply: impl FnOnce(
                OwnerCompletion<'owner, Value, Stop, Custody>,
                &mut InvalidationEditAdmission,
            ) -> Result<Next, NextStop>
            + 'owner,
    ) -> OwnerPlan<'owner, Next, NextStop> {
        let Self {
            requests,
            inputs,
            complete,
        } = self;
        let completion = OwnerCompletion {
            slots: Vec::new(),
            complete: Some(complete),
            custody,
        };
        let mut plan = OwnerPlan::new(requests, move |slots, admission| {
            apply(completion.with_slots(slots), admission)
        });
        plan.inputs = inputs;
        plan
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn map_result<
        Next: 'owner,
        NextStop: 'owner,
    >(
        self,
        finish: impl FnOnce(Result<Value, Stop>, &mut InvalidationEditAdmission) -> Result<Next, NextStop>
            + 'owner,
    ) -> OwnerPlan<'owner, Next, NextStop> {
        let Self {
            requests,
            inputs,
            complete,
        } = self;
        let mut plan = OwnerPlan::new(requests, move |slots, admission| {
            finish(complete(slots, admission), admission)
        });
        plan.inputs = inputs;
        plan
    }
}

#[cfg(test)]
mod completion_custody;
