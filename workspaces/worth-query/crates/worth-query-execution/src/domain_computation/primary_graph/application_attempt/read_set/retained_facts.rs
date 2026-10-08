use super::{
    admit_array, WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
    WorthQueryCompleteApplicationReadSet,
};
use crate::domain_computation::primary_graph::application_attempt::check_request_live;
use worth_execution::ExecutionAllocationPolicy;

impl<Schema, Operation, Input, Scope, Phase>
    WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>
{
    /// Retains ordinary append order under an explicitly selected allocation
    /// policy. Both old and new payload backing remain owned during the move.
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
        check_request_live(self.admission.publication_request(), operation)?;
        let count = self
            .facts
            .len()
            .checked_add(additional.len())
            .ok_or_else(|| {
                super::super::retained_decision_facts::StoreDenial::Representability
                    .into_attempt_denial(operation)
            })?;
        // Allocate final backing before moving the old owner. The sealed empty
        // replacement uses the same explicit policy; it is not a Default lane.
        self.admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        let mut replacement = worth_execution::ExecutionArrayBuilder::allocate(count, policy)
            .map_err(|denial| {
                WorthQueryApplicationAttemptDenial::allocation_denied(operation, denial)
            })?;
        let empty = admit_array(0, std::iter::empty(), policy, operation, || {
            self.admission
                .validate_current_authority()
                .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)
        })?;
        let old = std::mem::replace(&mut self.facts, empty);
        for fact in old.into_iter().chain(additional) {
            self.admission
                .validate_current_authority()
                .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
            replacement.push(fact).map_err(|denial| {
                WorthQueryApplicationAttemptDenial::allocation_denied(operation, denial)
            })?;
        }
        self.admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        self.facts = replacement.seal().map_err(|denial| {
            WorthQueryApplicationAttemptDenial::allocation_denied(operation, denial)
        })?;
        Ok(())
    }
}
