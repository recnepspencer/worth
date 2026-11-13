//! Each typed stage governs inspection with its own genuine admission.
use super::capture::TypedStage;
use super::envelope::{Denial, Resolution};
use super::*;
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> TypedStage<S, B>
where
    B::Input: Clone + Send + Sync,
    B::IdempotencyKey: Send + Sync,
    B::Result: Send,
    B::Denial: Send,
{
    pub(super) fn envelope_product(
        &self,
        context: &AuthoredEditStageContext<'_, '_, '_, S>,
    ) -> Result<crate::basis::WorthQueryProductBranchLease, Denial> {
        let admitted = (self.admit)(context).map_err(|_| Denial::foreign_admission())?;
        admitted
            .operation
            .graph_work()
            .mutation_product()
            .map(|product| product.retained_clone())
            .ok_or_else(Denial::foreign_admission)
    }
    pub(super) fn inspect_envelope(
        &self,
        context: &AuthoredEditStageContext<'_, '_, '_, S>,
        coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
        intent: worth_query_declaration::facade::application_operation::ApplicationEditIntentIdentity,
        next: &mut dyn FnMut(WorthQueryApplicationIdempotencyBinding) -> Result<Resolution, Denial>,
    ) -> Result<Resolution, Denial> {
        let admitted = (self.admit)(context).map_err(|_| Denial::foreign_admission())?;
        if admitted.operation.scope_entity_id() != self.source {
            return Err(Denial::foreign_admission());
        }
        if admitted
            .operation
            .graph_work()
            .mutation_product()
            .is_none_or(|product| product.observation() != &self.before)
        {
            return Err(Denial::foreign_admission());
        }
        let identities = self
            .key
            .stage_with_encoded_input::<S, B>(&self.input, self.ordinal)
            .map_err(|_| Denial::foreign_admission())?;
        let binding = WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities)
            .bind_edit_envelope(&intent)
            .bind_admission(&admitted.operation);
        let execution = context
            .phase
            .checked_execution_request_for(context.application.product_runtime())
            .map_err(|_| Denial::foreign_admission())?;
        let proof = context
            .application
            .authorize_idempotency_inspection(&execution, &admitted.operation, coordination)
            .map_err(Denial::from_authorization)?;
        proof
            .govern((), |()| next(binding))
            .map_err(|(_, cause)| Denial::from_authorization(cause))?
    }
}
