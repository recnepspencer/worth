//! Fresh command admission validates the original performed settlement before native recovery.
use super::WorthQueryApplicationSettlementRecoveryError;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryAdvancementDenial,
    WorthQueryAdvancementPhase, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationSettlementDeferred, WorthQueryOperationAuthorizationDenial,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_query_installation::facade::ApplicationSchema;

/// A settlement refusal preserves its original performed native handle.
#[derive(Debug)]
pub enum WorthQueryCommandSettlementDenial {
    Execution(WorthQueryAdvancementDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
    ForeignAdmission,
    BindingMismatch,
    Recovery(WorthQueryApplicationSettlementRecoveryError),
}
impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn recover_deferred_application_settlement_in_advancement<O, I, Scope>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &WorthQueryProgramPublicationAccess,
        deferred: &WorthQueryApplicationSettlementDeferred,
        admission: &WorthQueryAdmittedApplicationOperation<S, O, I, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        worth_relational::facade::history::RelationalCommitReceipt,
        WorthQueryCommandSettlementDenial,
    >
    where
        I: Clone + Send + Sync + 'static,
    {
        use WorthQueryCommandSettlementDenial as D;
        let execution = phase
            .checked_execution_request_for(&self.product_runtime)
            .map_err(D::Execution)?;
        admission
            .validate_current_authority()
            .map_err(D::Authorization)?;
        if !admission.belongs_to(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
        ) {
            return Err(D::ForeignAdmission);
        }
        let bound = idempotency
            .bind_operation(admission.operation_definition_identity())
            .bind_operation_scope(admission.operation_scope_binding())
            .bind_preconditions(admission.mutation_preconditions().identity())
            .bind_governed_input(admission.governed_input_identity())
            .bind_governed_proposal(admission.governed_proposal_identity());
        let product = admission
            .graph_work()
            .mutation_product()
            .ok_or(D::ForeignAdmission)?
            .publication_binding();
        let affinity = crate::domain_computation::primary_graph::provider::WorthQueryProductIdempotencyAffinity::from_observation(product.observation());
        if bound != deferred.idempotency_binding() || &affinity != deferred.product_affinity() {
            return Err(D::BindingMismatch);
        }
        self.recover_deferred_application_settlement(&execution, deferred)
            .map_err(D::Recovery)
    }
}
