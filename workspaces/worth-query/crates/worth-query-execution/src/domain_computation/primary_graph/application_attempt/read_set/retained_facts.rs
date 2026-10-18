use super::{
    admit_completed_facts, WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
    WorthQueryCompleteApplicationReadSet,
};
use crate::domain_computation::primary_graph::{
    application_attempt::check_request_live,
    application_attempt::retained_decision_facts::StorageControl,
};
use worth_execution::ExecutionAllocationPolicy;

impl<Schema, Operation, Input, Scope, Phase>
    WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>
{
    /// Retains ordinary append order and original backing. Old and new roots
    /// coexist until admission, liveness and authority checks all succeed.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn append_completed_facts<
        Facts,
    >(
        &mut self,
        additional: Facts,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Facts: IntoIterator<Item = WorthQueryApplicationObservedFact>,
        Facts::IntoIter: ExactSizeIterator,
    {
        let additional = additional.into_iter();
        if additional.len() == 0 {
            return Ok(());
        }
        let operation = self.admission.operation();
        let request = self.admission.publication_request();
        check_request_live(request, operation)?;
        let check = || {
            self.admission
                .validate_current_authority()
                .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)
        };
        let tail = admit_completed_facts(
            additional.len(),
            additional,
            policy,
            request,
            operation,
            check,
        )?;
        let joined = self
            .facts
            .append_completed(tail, StorageControl::new(policy, Some(request)))
            .map_err(|cause| cause.into_attempt_denial(operation))?;
        check()?;
        self.facts = joined;
        Ok(())
    }
}
