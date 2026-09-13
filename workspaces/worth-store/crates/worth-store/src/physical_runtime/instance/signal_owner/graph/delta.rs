use worth_signal::facade::ChangedRegion;

use crate::physical_runtime::work::{PhysicalSignalAspectBindingDigest, PhysicalWorkAspectDelta};

use super::super::PhysicalSignalDeltaApplicationFailure;

impl super::PhysicalSignalGraph {
    pub(super) fn apply_delta(
        &mut self,
        route_slot: usize,
        route: PhysicalSignalAspectBindingDigest,
        delta: &PhysicalWorkAspectDelta,
    ) -> Result<(), PhysicalSignalDeltaApplicationFailure> {
        let binding = self
            .bindings
            .binding_for_slot(route_slot)
            .filter(|binding| binding.digest() == route && binding.digest() == delta.binding())
            .ok_or(PhysicalSignalDeltaApplicationFailure::BindingNotInstalled)?;
        if !delta.is_installed_by(binding) {
            return Err(PhysicalSignalDeltaApplicationFailure::BindingCapabilityMismatch);
        }
        let source = self
            .topology
            .source_for_slot(route_slot)
            .ok_or(PhysicalSignalDeltaApplicationFailure::BindingNotInstalled)?;
        let aspect = binding.signal_aspect();
        let region = binding.partition().map(|partition| ChangedRegion {
            partition: partition.partition.clone(),
            detail: partition.detail.clone(),
        });
        let expected_basis = self
            .runtime
            .observe_signal_branch_basis(self.runtime.current_branch())
            .map_err(|_| PhysicalSignalDeltaApplicationFailure::SignalMutationRejected)?;

        self.locality.invalidate_scope(route, delta.scope());
        self.context.version = self
            .context
            .version
            .checked_add(1)
            .ok_or(PhysicalSignalDeltaApplicationFailure::VersionExhausted)?;
        self.runtime
            .advance_signal_branch(
                &mut self.context,
                &expected_basis,
                |transaction| match region.as_ref() {
                    Some(region) => transaction.mark_changed_with_regions(
                        source,
                        aspect,
                        std::slice::from_ref(region),
                    ),
                    None => transaction.mark_changed(source, aspect),
                },
            )
            .map_err(|_| PhysicalSignalDeltaApplicationFailure::SignalMutationRejected)?;
        self.settle_dependency(source)
            .map_err(|_| PhysicalSignalDeltaApplicationFailure::SignalEvaluationRejected)
    }
}
