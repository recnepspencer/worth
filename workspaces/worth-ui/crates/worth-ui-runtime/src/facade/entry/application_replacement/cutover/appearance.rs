use super::{WorthUiApplicationCutoverDenial, WorthUiPendingApplicationCutover};

pub(super) fn prepare_successor_theme(
    session: &crate::facade::WorthUiActiveApplicationSession,
    pending: &WorthUiPendingApplicationCutover,
    successor: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
) -> Result<
    crate::runtime::presentation_state::UiPreparedAppearanceGenerationSuccession,
    WorthUiApplicationCutoverDenial,
> {
    let candidate_index = pending.next_app.prepared_authority().consumed_fact_index();
    let has_candidate_consumers = candidate_index.has_appearance_consumers();
    let bindings = session
        .presentation
        .appearance_theme_state()
        .map(|state| state.active_bindings().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    if !has_candidate_consumers {
        return session
            .prepare_appearance_replacement_succession(successor, None, None, None)
            .map_err(WorthUiApplicationCutoverDenial::AppearanceThemeSuccession);
    }

    let themes = pending.next_app.capabilities().appearance_themes().ok_or(
        WorthUiApplicationCutoverDenial::AppearanceThemeAdmission(
            crate::runtime::appearance::UiThemeCapabilityReceiptDenial::MissingBundle,
        ),
    )?;
    let host_profile = session
        .host_session
        .capability_report()
        .appearance_profile()
        .ok_or(WorthUiApplicationCutoverDenial::AppearanceThemeAdmission(
            crate::runtime::appearance::UiThemeCapabilityReceiptDenial::MissingHostProfile,
        ))?;
    let admission =
        crate::runtime::appearance::UiThemeCapabilityAdmission::from_frozen_capabilities(
            themes,
            themes.initial_definition_identity(),
            pending.next_app.capabilities().appearance_roles(),
            host_profile,
        )
        .and_then(|admission| {
            admission.prepare(
                candidate_index.appearance_required_role_identities(),
                successor.clone(),
            )
        })
        .map_err(WorthUiApplicationCutoverDenial::AppearanceThemeAdmission)?;
    let rebinding = if bindings.is_empty() {
        None
    } else {
        Some(
            crate::runtime::appearance::prepare_theme_generation_rebinding(
                themes,
                pending.next_app.capabilities().appearance_roles(),
                host_profile,
                &session.active_generation_identity(),
                successor.clone(),
                bindings.iter(),
            )
            .map_err(WorthUiApplicationCutoverDenial::AppearanceThemeAdmission)?,
        )
    };
    session
        .prepare_appearance_replacement_succession(
            successor,
            Some(admission),
            rebinding.as_ref(),
            Some(themes),
        )
        .map_err(WorthUiApplicationCutoverDenial::AppearanceThemeSuccession)
}

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
}
