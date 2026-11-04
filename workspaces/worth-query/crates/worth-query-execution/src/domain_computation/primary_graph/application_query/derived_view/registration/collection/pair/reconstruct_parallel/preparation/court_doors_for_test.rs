//! Sealed test doors over this module's private members.
use super::*;

impl<'a, S: ApplicationSchema, FQ, FP, FR, SQ, SP, SR, A, I, C>
    WorthQueryDerivedPairReadPlans<'a, S, FQ, FP, FR, SQ, SP, SR, A, I, C>
{
    #[doc(hidden)]
    pub fn admit_for_test(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        root: EntityId,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<
        (
            WorthQueryApplicationAuthorizationWorkEvidence,
            WorthQueryApplicationAuthorizationWorkEvidence,
        ),
        Denial,
    > {
        Self::admit(self, execution, application, root, context)
    }
}

impl<'a, S: ApplicationSchema, FQ, FP, FR, SQ, SP, SR, A, I, C>
    WorthQueryDerivedPairReadPlans<'a, S, FQ, FP, FR, SQ, SP, SR, A, I, C>
{
    #[doc(hidden)]
    pub fn worker_for_test(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        root: EntityId,
    ) -> Result<PreparedPair<'_>, Denial> {
        Self::worker(self, execution, root)
    }
}

impl<'a, S: ApplicationSchema, FQ, FP, FR, SQ, SP, SR, A, I, C>
    WorthQueryDerivedPairReadPlans<'a, S, FQ, FP, FR, SQ, SP, SR, A, I, C>
{
    #[doc(hidden)]
    pub fn worker_with_pins_for_test(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        root: EntityId,
        pins: &mut crate::domain_computation::primary_graph::application_query::one_shot::pair_plans::WorthQueryPairReadPins,
    ) -> Result<PreparedPair<'_>, Denial> {
        Self::worker_with_pins(self, execution, root, pins)
    }
}
