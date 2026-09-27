//! Typed source extraction for one narrowed allocation invalidation.

pub(super) fn query_source_of(
    target: &crate::runtime::UiAllocationInvalidationTarget,
) -> Option<(
    &crate::capability::ViewBindingId,
    &worth_ui_query_binding::WorthUiSettledSnapshotFact,
)> {
    match target {
        crate::runtime::UiAllocationInvalidationTarget::SettledQueryFact {
            view_binding_id,
            fact,
            ..
        }
        | crate::runtime::UiAllocationInvalidationTarget::ScrollOwnedContentExtent {
            view_binding_id,
            fact,
            ..
        } => Some((view_binding_id, fact)),
        crate::runtime::UiAllocationInvalidationTarget::Graph(_)
        | crate::runtime::UiAllocationInvalidationTarget::ResizePreview { .. }
        | crate::runtime::UiAllocationInvalidationTarget::HostMeasurement { .. }
        | crate::runtime::UiAllocationInvalidationTarget::PortalAnchor { .. }
        | crate::runtime::UiAllocationInvalidationTarget::ScrollOwnedExtent { .. }
        | crate::runtime::UiAllocationInvalidationTarget::DurableResize { .. } => None,
    }
}

pub(super) fn scroll_bindings_of(
    target: &crate::runtime::UiAllocationInvalidationTarget,
) -> &[crate::runtime::UiAdmittedScrollInvalidationBinding] {
    match target {
        crate::runtime::UiAllocationInvalidationTarget::ScrollOwnedContentExtent {
            bindings,
            ..
        }
        | crate::runtime::UiAllocationInvalidationTarget::ScrollOwnedExtent { bindings, .. } => {
            bindings
        }
        crate::runtime::UiAllocationInvalidationTarget::Graph(_)
        | crate::runtime::UiAllocationInvalidationTarget::ResizePreview { .. }
        | crate::runtime::UiAllocationInvalidationTarget::SettledQueryFact { .. }
        | crate::runtime::UiAllocationInvalidationTarget::HostMeasurement { .. }
        | crate::runtime::UiAllocationInvalidationTarget::PortalAnchor { .. }
        | crate::runtime::UiAllocationInvalidationTarget::DurableResize { .. } => &[],
    }
}

pub(super) fn target_sets_of(
    target: &crate::runtime::UiAllocationInvalidationTarget,
) -> Vec<&crate::graph::UiAdmittedAllocationInvalidationTargetSet> {
    let bindings = scroll_bindings_of(target);
    if !bindings.is_empty() {
        return bindings.iter().map(|binding| binding.target()).collect();
    }
    vec![targets_of(target)]
}

fn targets_of(
    target: &crate::runtime::UiAllocationInvalidationTarget,
) -> &crate::graph::UiAdmittedAllocationInvalidationTargetSet {
    match target {
        crate::runtime::UiAllocationInvalidationTarget::Graph(target)
        | crate::runtime::UiAllocationInvalidationTarget::ResizePreview { target, .. }
        | crate::runtime::UiAllocationInvalidationTarget::SettledQueryFact { target, .. }
        | crate::runtime::UiAllocationInvalidationTarget::HostMeasurement { target, .. }
        | crate::runtime::UiAllocationInvalidationTarget::DurableResize { target, .. } => target,
        crate::runtime::UiAllocationInvalidationTarget::PortalAnchor { movement } => {
            movement.target()
        }
        crate::runtime::UiAllocationInvalidationTarget::ScrollOwnedContentExtent { .. }
        | crate::runtime::UiAllocationInvalidationTarget::ScrollOwnedExtent { .. } => {
            unreachable!("scroll bindings expose their own target sets")
        }
    }
}
