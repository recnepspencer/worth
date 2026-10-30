use super::super::registration::BridgeAspectRegistration;

pub(crate) fn canonical_aspect_registration_order(
    left: &BridgeAspectRegistration,
    right: &BridgeAspectRegistration,
) -> std::cmp::Ordering {
    let left_scope = left.truth_scope();
    let right_scope = right.truth_scope();
    let left_aspect_exact = left_scope.aspect_selector().is_exact();
    let right_aspect_exact = right_scope.aspect_selector().is_exact();
    let left_target_exact = left_scope.target_selector().is_exact();
    let right_target_exact = right_scope.target_selector().is_exact();
    let left_entity_exact = left_scope.entity_selector().is_exact();
    let right_entity_exact = right_scope.entity_selector().is_exact();

    left_aspect_exact
        .cmp(&right_aspect_exact)
        .then_with(|| right_target_exact.cmp(&left_target_exact))
        .then_with(|| right_entity_exact.cmp(&left_entity_exact))
        .then_with(|| left.truth_surface_kind().cmp(&right.truth_surface_kind()))
        .then_with(|| left.truth_scope().cmp(right.truth_scope()))
        .then_with(|| {
            left.snapshot_read_contract()
                .canonical_basis()
                .cmp(right.snapshot_read_contract().canonical_basis())
        })
        .then_with(|| {
            left.subscription_slice_kind()
                .cmp(right.subscription_slice_kind())
        })
        .then_with(|| left.widening_policy().cmp(&right.widening_policy()))
        .then_with(|| {
            left.source_precision_policy()
                .cmp(&right.source_precision_policy())
        })
        .then_with(|| left.registration_id().cmp(right.registration_id()))
}
