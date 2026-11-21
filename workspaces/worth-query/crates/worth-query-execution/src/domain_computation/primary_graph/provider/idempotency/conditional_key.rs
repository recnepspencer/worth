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

/// World's state for one original key on its selected product branch.
pub(in crate::domain_computation::primary_graph) enum WorthQueryConditionalKeyFate {
    /// The original occurrence is published on the product branch.
    Published(super::WorthQueryRecoveredApplicationCommitReceipt),
    /// The original is recovered unpublished custody under this handle.
    RecoveredUnpublished(ProductUnpublishedRecoveryHandle),
    /// This live process still holds unpublished or pending publication
    /// custody for the key on the selected branch incarnation.
    LiveUnpublishedOrPending,
    /// World released an original that kept its owner effect.
    Released,
    /// No original exists: no committed open, or an empty one World released.
    Absent,
    /// Another root intent holds the key.
    OtherIntent,
}

impl WorthQueryPrimaryGraphProvider {
    /// Whether this live process still holds unpublished or pending publication
    /// custody for `key` on `product`. Checked before concluding `Absent` in a
    /// live process so an in-flight/unreleased original is never erased.
    pub(in crate::domain_computation::primary_graph) fn has_live_unpublished_or_pending_conditional_key(
        &self,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        key: [u8; 32],
    ) -> bool {
        let affinity =
            WorthQueryProductIdempotencyAffinity::from_observation(product.observation());
        if self.has_live_unpublished_key(&affinity, key) {
            return true;
        }
        matches!(
            self.inspect_pending_application_idempotency(affinity.incarnation()),
            Some(Some(recorded)) if *recorded.key_identity() == key
        ) || matches!(
            self.inspect_pending_application_idempotency(affinity.incarnation()),
            Some(None)
        )
    }

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
        if self.has_live_unpublished_or_pending_conditional_key(product, key) {
            return Ok(WorthQueryConditionalKeyFate::LiveUnpublishedOrPending);
        }
        let layout = self.graph.layout.provider_idempotency().clone();
        self.graph.with_runtime(execution, |runtime| {
            let selected = super::observe_selected_idempotency_branch(product)?;
            let basis = selected.basis().relational_basis();
            let original = self.read_current_idempotency_at_basis(
                execution,
                runtime,
                &layout,
                basis,
                |snapshot| {
                    original_at_snapshot(
                        execution, self, runtime, snapshot, &layout, product, binding,
                    )
                },
            )?;
            Ok(match original {
                None => WorthQueryConditionalKeyFate::Absent,
                Some(receipt) if recorded_root(receipt.recorded_intent())? == intent => {
                    WorthQueryConditionalKeyFate::Published(receipt)
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
