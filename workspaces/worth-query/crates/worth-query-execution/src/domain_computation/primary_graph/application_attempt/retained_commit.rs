use crate::domain_computation::primary_graph::application_attempt::ApplicationCommitOps;
use crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost;
use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchema;

use {super::WorthQueryApplicationCommitOutcome, super::WorthQueryApplicationCommitReceipt, super::WorthQueryApplicationEffectProgram, crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding};
use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;

/// The fresh performed carrier exists only for a publication that reserved it.
/// An idempotent replay remains descriptive and cannot create a new lease.
pub enum WorthQueryApplicationRetainedCommitOutcome {
    Committed {
        receipt: WorthQueryApplicationCommitReceipt,
        retained: Arc<WorthQueryApplicationReadObservation>,
    },
    Other(WorthQueryApplicationCommitOutcome),
}

pub trait RetainedCommitOps<Schema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    fn compare_and_commit_application_retained_in_advancement<Operation, Input, Scope>(
        &self,

        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationRetainedCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let outcome = self.compare_and_commit_application_in_advancement(
            phase,
            program.with_client_observation(),
            idempotency,
            allocation_policy,
        );
        self.retained_commit_outcome(outcome)
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> RetainedCommitOps<Schema> for HostRuntime where
    Schema: ApplicationSchema, {}

pub(in crate::domain_computation::primary_graph) trait RetainedCommitScopedOps<Schema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn retained_commit_outcome(
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
            self.application_runtime().application_query_host(),
            crate::basis::WorthQueryProductObservationLease::new(observation),
        );
        WorthQueryApplicationRetainedCommitOutcome::Committed { receipt, retained }
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> RetainedCommitScopedOps<Schema> for HostRuntime where
    Schema: ApplicationSchema, {}
