//! Checked source ownership retained across recorded idempotency resolution.
use super::*;

/// A checked source identity with its actual observed-source owner.
/// No fact payload, allocation policy, or lease is stored in this phase.
pub struct WorthQueryPendingSourceExpectation<Query> {
    pub(super) admission_identity: WorthQueryOperationAdmissionIdentity,
    pub(super) selected_product: crate::basis::WorthQueryProductBranchReadIdentity,
    pub(super) expected_query_identifier: &'static str,
    pub(super) bound: WorthQueryBoundSourceExpectation,
    pub(super) source: WorthQueryObservedSource<Query>,
}

impl<Query> WorthQueryPendingSourceExpectation<Query> {
    /// The same checked extension used for fresh execution and recorded retries.
    pub const fn bound_source(&self) -> WorthQueryBoundSourceExpectation {
        self.bound
    }

    /// Materialize only after recorded resolution has selected fresh execution.
    pub fn consume_into<Schema, Operation, Input, Scope>(
        self,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        phase: &crate::domain_computation::primary_graph::request_execution::WorthQueryAdvancementPhase<'_>,
        admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<(), WorthQuerySourceExpectationDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        runtime
            .readmit_mutable_source_admission(admission, self.expected_query_identifier)?
            .consume_pending(self, phase)
    }
}
