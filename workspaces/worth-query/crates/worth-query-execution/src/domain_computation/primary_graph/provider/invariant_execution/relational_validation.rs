//! The actual Relational validation, under the admitted request control.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    pub(super) fn validate_relational_candidate(
        &self,
        batch: worth_relational::facade::transactions::WorkerIntentBatch,
        branch: &worth_relational::facade::history::BranchId,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        application_touches: &worth_query_installation::facade::WorthQueryOperationTouchContract,
        aftermath_causality: Option<
            &crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
        >,
    ) -> Result<
        worth_relational::facade::mvcc::ValidatedRelationalProposal,
        WorthQueryInvariantExecutionFailure,
    > {
        #[cfg(not(test))]
        let _ = application_touches;
        let batch = if self.take_relational_invariant_violation() {
            batch.push(invariant_violation_probe())
        } else {
            batch
        };
        #[cfg(test)]
        let batch = if self.take_undeclared_application_touch() {
            batch.push(undeclared_application_touch_probe(
                &self.graph.layout,
                application_touches,
            )?)
        } else {
            batch
        };
        let basis = product.observation().basis().relational_basis();
        if basis.identity().branch_id() != branch {
            return Err(owner_failure());
        }
        let control = super::validation_control::ValidationRequestControl::new(request);
        let candidate = self.graph.with_runtime_mut(|runtime| {
            if let Some(pending) = aftermath_causality {
                let observed_parent = basis
                    .observation()
                    .commit_receipt()
                    .cloned()
                    .ok_or_else(aftermath_failure)?;
                if pending.parent() != &observed_parent {
                    return Err(aftermath_failure());
                }
            }
            let mut transaction = runtime
                .begin_branch_transaction_with_control(
                    basis,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                    control.relational().clone(),
                )
                .map_err(map_transaction_admission_failure)?;
            transaction
                .push_batch(batch)
                .map_err(map_transaction_staging_failure)?;
            #[cfg(test)]
            self.fault_port.candidate_staged_for_validation();
            Ok::<_, WorthQueryInvariantExecutionFailure>(transaction.validate(runtime))
        });
        let candidate = candidate?.map_err(map_validation_failure)?;
        validate_owner_evidence(candidate.invariant_evidence(), branch)?;
        Ok(candidate)
    }
}
