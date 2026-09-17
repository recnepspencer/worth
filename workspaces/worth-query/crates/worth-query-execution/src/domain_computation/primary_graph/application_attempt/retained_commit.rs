use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationReadObservation, WorthQueryPrimaryGraphApplicationRuntime,
};

/// The fresh performed carrier exists only for a publication that reserved it.
/// An idempotent replay remains descriptive and cannot create a new lease.
pub enum WorthQueryApplicationRetainedCommitOutcome {
    Committed {
        receipt: WorthQueryApplicationCommitReceipt,
        retained: Arc<WorthQueryApplicationReadObservation>,
    },
    Other(WorthQueryApplicationCommitOutcome),
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn compare_and_commit_application_retained<Operation, Input, Scope>(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationRetainedCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let outcome =
            self.compare_and_commit_application(program.with_client_observation(), idempotency);
        self.retained_commit_outcome(outcome)
    }

    pub(in crate::domain_computation::primary_graph) fn retained_commit_outcome(
        &self,
        outcome: WorthQueryApplicationCommitOutcome,
    ) -> WorthQueryApplicationRetainedCommitOutcome {
        let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
            return WorthQueryApplicationRetainedCommitOutcome::Other(outcome);
        };
        let publication = receipt.committed_product_publication();
        let observation = publication
            .take_client_observation()
            .expect("opt-in publication reserved its exact successor observation before effects");
        assert_eq!(observation.branch_identity(), publication.product_branch());
        assert_eq!(
            observation.selected_commit(),
            publication.composite_commit()
        );
        let retained = WorthQueryApplicationReadObservation::from_product(
            self,
            crate::basis::WorthQueryProductObservationLease::new(observation),
        );
        WorthQueryApplicationRetainedCommitOutcome::Committed { receipt, retained }
    }
}
