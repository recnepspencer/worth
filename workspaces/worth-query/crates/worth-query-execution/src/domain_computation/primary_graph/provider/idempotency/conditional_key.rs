//! What World holds for an indeterminate conditional decision's original key.
//!
//! The decision retained only its key and root intent: the admission slots
//! the commit bound were lost with the answer. A recorded original belongs to
//! the decision exactly when its key and recorded root intent are the row's.
use super::snapshot_resolution::original_at_snapshot;
use super::{
    WorthQueryApplicationIdempotencyBinding, WorthQueryPrimaryGraphProvider,
    WorthQueryProductIdempotencyAffinity, WorthQueryProviderIdempotencyResolutionDenial as Denial,
};
use worth_runtime_world::facade::ProductUnpublishedRecoveryHandle;

/// World's state for one original key, read before service.
pub(in crate::domain_computation::primary_graph) enum WorthQueryConditionalKeyFate {
    /// The original occurrence is published on the product branch.
    Published,
    /// The original is recovered unpublished custody under this handle.
    RecoveredUnpublished(ProductUnpublishedRecoveryHandle),
    /// World released an original that kept its owner effect.
    Released,
    /// No original exists: no committed open, or an empty one World released.
    Absent,
    /// Another root intent holds the key.
    OtherIntent,
}

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn conditional_key_fate(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        key: [u8; 32],
        intent: [u8; 32],
    ) -> Result<WorthQueryConditionalKeyFate, Denial> {
        let binding = WorthQueryApplicationIdempotencyBinding::new(key, intent);
        let affinity =
            WorthQueryProductIdempotencyAffinity::from_observation(product.observation());
        if let Some(fate) = self.recovered_key_fate(&affinity, binding)? {
            return Ok(fate);
        }
        let layout = self.graph.layout.provider_idempotency().clone();
        self.graph.with_runtime(execution, |runtime| {
            let selected = product
                .observation_port()
                .observe_product_branch_occurrence(product.observation().lifecycle_incarnation())
                .map_err(|_| Denial::ForeignAffinity)?;
            let basis = selected.basis().relational_basis();
            self.graph
                .carry_primary_index_basis_for_basis(execution, runtime, basis)
                .map_err(super::idempotency_index_currency_denial)?;
            runtime
                .index_authority()
                .ensure_field_index_for_basis(
                    layout.key_index_id,
                    basis,
                    crate::domain_computation::primary_graph::index_maintenance_budget::ordinary_index_maintenance_budget(),
                    execution,
                )
                .map_err(|denial| match denial.kind {
                    worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind::Execution(cause) => {
                        super::snapshot_resolution::idempotency_read_stop(cause)
                    }
                    _ => Denial::Unavailable,
                })?;
            let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(runtime, basis)
                .map_err(|denial| super::idempotency_snapshot_denial(denial.into()))?;
            let original =
                original_at_snapshot(execution, self, runtime, &snapshot, &layout, product, binding);
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            Ok(match original? {
                None => WorthQueryConditionalKeyFate::Absent,
                Some(receipt) if recorded_root(receipt.recorded_intent())? == intent => {
                    WorthQueryConditionalKeyFate::Published
                }
                Some(_) => WorthQueryConditionalKeyFate::OtherIntent,
            })
        })
    }
}

/// The root intent identity every recorded intent grammar opens with.
pub(in crate::domain_computation::primary_graph::provider) fn recorded_root(
    recorded: &str,
) -> Result<[u8; 32], Denial> {
    Ok(WorthQueryApplicationIdempotencyBinding::observe_recorded_intent(recorded)?.root)
}
