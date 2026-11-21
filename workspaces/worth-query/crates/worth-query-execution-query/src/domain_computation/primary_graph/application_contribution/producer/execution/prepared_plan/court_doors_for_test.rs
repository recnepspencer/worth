//! Sealed test doors over this module's private members.
use super::*;

impl<'owner, Value: 'owner, Stop: 'owner> OwnerPlan<'owner, Value, Stop> {
    #[doc(hidden)]
    pub fn immediate_for_test(outcome: Result<Value, Stop>) -> Self {
        Self::immediate(outcome)
    }
}

impl<'owner, Value: 'owner, Stop: 'owner> OwnerPlan<'owner, Value, Stop> {
    #[doc(hidden)]
    pub fn new_for_test(
        requests: Vec<PreparedCompute>,
        complete: impl FnOnce(Vec<ComputedSlot>) -> Result<Value, Stop> + 'owner,
    ) -> Self {
        Self::new(requests, complete)
    }
}

impl<'owner, Value: 'owner, Stop: 'owner> OwnerPlan<'owner, Value, Stop> {
    #[doc(hidden)]
    pub fn with_inputs_for_test(
        self,
        inputs: crate::domain_computation::primary_graph::application_output_demand::PreparedRequiredInputs,
    ) -> Self {
        Self::with_inputs(self, inputs)
    }
}
