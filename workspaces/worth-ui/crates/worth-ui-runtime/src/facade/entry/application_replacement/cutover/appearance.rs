use super::{WorthUiApplicationCutoverDenial, WorthUiPendingApplicationCutover};

pub(super) fn validate_candidate_owner_installation(
    pending: &WorthUiPendingApplicationCutover,
    service_policy: &crate::declaration::UiNormalizedServicePolicyPlan,
) -> Result<(), WorthUiApplicationCutoverDenial> {
    let demand = pending
        .next_app
        .prepared_authority()
        .consumed_fact_index()
        .appearance_axis_demand();
    for (axis, owner_installed) in [
        (
            worth_ui_dsl::UiAppearanceStateAxis::Focus,
            service_policy.focus().is_some(),
        ),
        (
            worth_ui_dsl::UiAppearanceStateAxis::Selection,
            service_policy.selection().is_some(),
        ),
    ] {
        if demand.contains(axis) && !owner_installed {
            return Err(WorthUiApplicationCutoverDenial::AppearanceOwnerUnavailable(
                axis,
            ));
        }
    }
    Ok(())
}

pub(super) fn reconcile_successor_owners(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
) {
    let demand = session
        .application
        .prepared_authority()
        .consumed_fact_index()
        .appearance_axis_demand();
    session.interaction.reconcile_appearance_demand(
        demand.contains(worth_ui_dsl::UiAppearanceStateAxis::Hover),
        demand.contains(worth_ui_dsl::UiAppearanceStateAxis::Pressed),
    );
    session.intent_admission.reconcile_operability_appearance(
        demand.contains(worth_ui_dsl::UiAppearanceStateAxis::Operability),
    );
    session
        .intent_application_facts
        .reconcile_validation_appearance(
            demand.contains(worth_ui_dsl::UiAppearanceStateAxis::Validation),
        );
    session.appearance_inspection.reset_for_new_generation();
    session.appearance_owner_snapshot = None;
}
