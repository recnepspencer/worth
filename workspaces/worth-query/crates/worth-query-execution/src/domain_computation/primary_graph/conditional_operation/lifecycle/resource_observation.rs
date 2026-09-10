use std::sync::Arc;

use super::{WorthQueryConditionalRetainedResourceCounts, WorthQueryInstalledTemporalOperation};

pub(super) fn retained_resource_counts<Binding, Reconstruction, Execution, Clock, Input>(
    operation: &WorthQueryInstalledTemporalOperation<
        Binding,
        Reconstruction,
        Execution,
        Clock,
        Input,
    >,
) -> WorthQueryConditionalRetainedResourceCounts {
    let mut counts = super::super::lifecycle_inventory::retained_resource_counts(
        &operation.retained_wakes,
        operation.reconstructed_intents.len(),
    );
    counts.direct_deliveries += usize::from(operation.pending_direct_delivery.is_some());
    for binding in operation.inactive_bindings.values() {
        counts.add(super::super::lifecycle_inventory::retained_resource_counts(
            &binding.retained_wakes,
            binding.reconstructed_intents.len(),
        ));
        counts.direct_deliveries += usize::from(binding.pending_direct_delivery.is_some());
    }
    counts
}

pub(super) fn lifecycle_resources<Binding, Reconstruction, Execution, Clock, Input>(
    operation: &WorthQueryInstalledTemporalOperation<
        Binding,
        Reconstruction,
        Execution,
        Clock,
        Input,
    >,
) -> super::super::lifecycle_inventory::WorthQueryConditionalOperationLiveness {
    let mut resources = super::super::lifecycle_inventory::WorthQueryConditionalOperationLiveness {
        binding: Arc::downgrade(&operation.lifecycle_token),
        lease: Arc::downgrade(&operation.definition.clock_lease),
        wakes: weak_wakes(&operation.retained_wakes),
        intents: operation
            .reconstructed_intents
            .values()
            .map(|intent| Arc::downgrade(&intent.lifecycle_token))
            .collect(),
        attempts: weak_attempts(&operation.retained_wakes),
    };
    for binding in operation.inactive_bindings.values() {
        resources.wakes.extend(weak_wakes(&binding.retained_wakes));
        resources.intents.extend(
            binding
                .reconstructed_intents
                .values()
                .map(|intent| Arc::downgrade(&intent.lifecycle_token)),
        );
        resources
            .attempts
            .extend(weak_attempts(&binding.retained_wakes));
    }
    resources
}

fn weak_wakes(
    wakes: &[super::super::signal_decision_reentry::WorthQueryRetainedConditionalWake],
) -> Vec<std::sync::Weak<()>> {
    wakes
        .iter()
        .map(|wake| Arc::downgrade(&wake.lifecycle_token))
        .collect()
}

fn weak_attempts(
    wakes: &[super::super::signal_decision_reentry::WorthQueryRetainedConditionalWake],
) -> Vec<std::sync::Weak<()>> {
    wakes
        .iter()
        .filter(|wake| wake.application_attempted)
        .map(|wake| Arc::downgrade(&wake.lifecycle_token))
        .collect()
}
