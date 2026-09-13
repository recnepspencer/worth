use crate::facade::{WorthUi, WorthUiActiveApplicationSession, WorthUiApp};
use crate::runtime::tests::source_ingress_boundary_test_support::{
    lower_file_submission, source_backed_package_component, source_backed_package_region,
    source_backed_package_sizing,
};
use crate::runtime::{
    WorthUiSourceProvider, WorthUiWatchedCandidateSubmission, WorthUiWatcherEvent,
};

#[path = "active_application_session_test_support/declared_surface.rs"]
mod declared_surface;
pub(crate) use declared_surface::source_backed_declared_surface_component_app_with_host;

pub(crate) use super::active_application_candidate_catalog_test_support::{
    admit_candidate_catalog, admit_candidate_complete_catalog,
    admit_first_candidate_catalog_row_with_viewport_width,
};

pub(crate) fn source_backed_component_session() -> WorthUiActiveApplicationSession {
    source_backed_component_app()
        .launch()
        .expect("component source application should launch")
}

pub(crate) fn source_backed_scaled_component_session(
    unrelated_component_count: usize,
) -> WorthUiActiveApplicationSession {
    source_backed_scaled_component_app(unrelated_component_count)
        .launch()
        .expect("scaled component source application should launch")
}

pub(crate) fn source_backed_component_app() -> WorthUiApp {
    source_backed_component_app_from_builder(component_builder(), |application| {
        crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host(
            application,
        )
    })
}

pub(crate) fn source_backed_component_app_with_host<Host>(host: Host) -> WorthUiApp
where
    Host: crate::facade::host::WorthUiHostAdapter + 'static,
{
    source_backed_component_app_from_builder(component_builder(), move |application| {
        crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
            application,
            host,
        )
    })
}

pub(crate) fn source_backed_focusable_component_app_with_host<Host>(host: Host) -> WorthUiApp
where
    Host: crate::facade::host::WorthUiHostAdapter + 'static,
{
    let snapshot = component_builder_with_focus()
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("focusable component snapshot should prepare");
    component_builder_with_focus()
        .with_candidate_submission(focus_component_submission(
            "focusable-active-session-current",
            "workspace.component.active_session_current",
            snapshot.capabilities(),
        ))
        .freeze()
        .map(|application| {
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                application,
                host,
            )
        })
        .expect("focusable component source application should prepare")
}

pub(crate) fn source_backed_component_app_with_host_and_viewport_allocation<Host>(
    host: Host,
) -> WorthUiApp
where
    Host: crate::facade::host::WorthUiHostAdapter + 'static,
{
    let snapshot = component_builder_with_viewport_allocation()
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("viewport component snapshot should prepare");
    component_builder_with_viewport_allocation()
        .with_candidate_submission(component_submission(
            "viewport-active-session-current",
            "workspace.component.active_session_current",
            snapshot.capabilities(),
        ))
        .freeze()
        .map(|application| {
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                application,
                host,
            )
        })
        .expect("viewport component source application should prepare")
}

pub(crate) fn source_backed_component_app_with_host_and_scalar_projection<Host>(
    host: Host,
    registration: worth_ui_query_binding::UiScalarProjectionRegistration,
) -> WorthUiApp
where
    Host: crate::facade::host::WorthUiHostAdapter + 'static,
{
    let builder = component_builder()
        .register_scalar_projection(registration)
        .expect("test projection registration should match its installed Query view");
    source_backed_component_app_from_builder(builder, move |application| {
        crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
            application,
            host,
        )
    })
}

fn source_backed_component_app_from_builder(
    builder: crate::facade::entry::WorthUiCertificationApplicationBuilder,
    activate: impl FnOnce(crate::facade::entry::WorthUiHostNeutralApp) -> WorthUiApp,
) -> WorthUiApp {
    let snapshot = component_builder()
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("component snapshot should prepare");
    builder
        .with_candidate_submission(component_submission(
            "active-session-current",
            "workspace.component.active_session_current",
            snapshot.capabilities(),
        ))
        .freeze()
        .map(activate)
        .expect("component source application should prepare")
}

pub(crate) fn component_candidate_submission(
    session: &WorthUiActiveApplicationSession,
    source_name: &str,
    component_id: &str,
) -> WorthUiWatchedCandidateSubmission {
    component_submission(source_name, component_id, session.capabilities())
}

pub(crate) fn component_candidate_submission_with_removal_token(
    session: &WorthUiActiveApplicationSession,
    source_name: &str,
    component_id: &str,
) -> WorthUiWatchedCandidateSubmission {
    lower_file_submission(
        WorthUiSourceProvider::in_memory(source_name).with_file(
            "app/main.wui",
            format!(
                "{}\ntoken theme.removal_only = \"theme.removal_only\";",
                component_declaration(component_id)
            ),
        ),
        [WorthUiWatcherEvent::provider_revision(source_name)],
        session.capabilities(),
    )
}

pub(crate) fn scaled_component_candidate_submission(
    session: &WorthUiActiveApplicationSession,
    source_name: &str,
    unrelated_component_count: usize,
) -> WorthUiWatchedCandidateSubmission {
    scaled_component_submission(
        source_name,
        "workspace.component.scale_target_candidate",
        unrelated_component_count,
        session.capabilities(),
    )
}

pub(crate) fn component_builder() -> crate::facade::entry::WorthUiApplicationBuilder {
    component_builder_with_allocation(false)
}

fn component_builder_with_viewport_allocation() -> crate::facade::entry::WorthUiApplicationBuilder {
    component_builder_with_allocation(true)
}

fn component_builder_with_focus() -> crate::facade::entry::WorthUiApplicationBuilder {
    let (_, _, world_profile) =
        crate::evidence::measurement::projection::fact_test_support::display_field_projection_context(
            "focusable-active-application-session",
        );
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .with_graph_world_profile(world_profile)
        .register_component(
            source_backed_package_component("workspace.component.active_session_current")
                .with_focus(crate::capability::ComponentFocusSupport::focusable()),
        )
        .register_component(
            source_backed_package_component("workspace.component.active_session_candidate")
                .with_focus(crate::capability::ComponentFocusSupport::focusable()),
        )
        .register_theme_token(crate::capability::ThemeTokenDescriptor::define(
            crate::capability::ThemeTokenId::new("theme.removal_only")
                .expect("removal-only fixture token id is valid"),
            crate::capability::ThemeTokenFamily::text(),
            crate::capability::ThemeTokenSource::application(),
            crate::capability::ThemeTokenValue::color(
                crate::capability::UiThemeColor::parse("#101820")
                    .expect("removal-only fixture token color is valid"),
            ),
        ))
        .register_mosaic_region_kind(source_backed_package_region())
        .register_mosaic_sizing_contract(source_backed_package_sizing())
        .register_mosaic_state_slot(component_runtime_state_slot())
}

fn component_builder_with_allocation(
    viewport_allocation: bool,
) -> crate::facade::entry::WorthUiApplicationBuilder {
    let (_, _, world_profile) =
        crate::evidence::measurement::projection::fact_test_support::display_field_projection_context(
            "active-application-session",
        );
    let allocation = crate::capability::ComponentAllocationMeasurementContract::fill_viewport();
    let active = source_backed_package_component("workspace.component.active_session_current");
    let candidate = source_backed_package_component("workspace.component.active_session_candidate");
    let active = if viewport_allocation {
        active.with_allocation_measurement_contract(allocation)
    } else {
        active
    };
    let candidate = if viewport_allocation {
        candidate.with_allocation_measurement_contract(allocation)
    } else {
        candidate
    };
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .with_graph_world_profile(world_profile)
        .register_component(active)
        .register_component(candidate)
        .register_theme_token(crate::capability::ThemeTokenDescriptor::define(
            crate::capability::ThemeTokenId::new("theme.removal_only")
                .expect("removal-only fixture token id is valid"),
            crate::capability::ThemeTokenFamily::text(),
            crate::capability::ThemeTokenSource::application(),
            crate::capability::ThemeTokenValue::color(
                crate::capability::UiThemeColor::parse("#101820")
                    .expect("removal-only fixture token color is valid"),
            ),
        ))
        .register_mosaic_region_kind(source_backed_package_region())
        .register_mosaic_sizing_contract(source_backed_package_sizing())
        .register_mosaic_state_slot(component_runtime_state_slot())
}

fn component_runtime_state_slot() -> crate::capability::MosaicStateSlotDescriptor {
    crate::capability::MosaicStateSlotDescriptor::new(
        crate::capability::MosaicStateSlotId::new("workspace.state.active_component")
            .expect("active component state-slot id is valid"),
        crate::capability::MosaicStateSlotKind::active_stack_item(),
    )
    .with_owner_identity(
        crate::capability::MosaicStateOwnerIdentity::mosaic_region_kind(
            crate::capability::MosaicRegionKindId::new("workspace.region.primary")
                .expect("active component state owner is valid"),
        ),
    )
    .with_persistence_policy(
        crate::capability::MosaicStatePersistencePolicy::restore_across_hot_reload(),
    )
    .with_replacement_rule(
        crate::capability::MosaicStateReplacementRule::preserve_when_owner_matches(),
    )
    .with_truth_posture(crate::capability::MosaicStateTruthPosture::ui_runtime_state())
}

fn source_backed_scaled_component_app(unrelated_component_count: usize) -> WorthUiApp {
    let builder = scaled_component_builder(unrelated_component_count);
    let snapshot = scaled_component_builder(unrelated_component_count)
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("scaled component snapshot should prepare");
    builder
        .with_candidate_submission(scaled_component_submission(
            "scaled-active-session-current",
            "workspace.component.scale_target_active",
            unrelated_component_count,
            snapshot.capabilities(),
        ))
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("scaled component source application should prepare")
}

fn scaled_component_builder(
    unrelated_component_count: usize,
) -> crate::facade::entry::WorthUiCertificationApplicationBuilder {
    let (_, _, world_profile) =
        crate::evidence::measurement::projection::fact_test_support::display_field_projection_context(
            "scaled-active-application-session",
        );
    let mut builder = WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .with_graph_world_profile(world_profile)
        .register_component(source_backed_package_component(
            "workspace.component.scale_target_active",
        ))
        .register_component(source_backed_package_component(
            "workspace.component.scale_target_candidate",
        ))
        .register_mosaic_region_kind(source_backed_package_region())
        .register_mosaic_sizing_contract(source_backed_package_sizing());
    for index in 0..unrelated_component_count {
        builder = builder.register_theme_token(scaled_unrelated_token(index));
    }
    builder
}

fn component_submission(
    source_name: &str,
    component_id: &str,
    capabilities: &crate::capability::CapabilitySnapshot,
) -> WorthUiWatchedCandidateSubmission {
    lower_file_submission(
        WorthUiSourceProvider::in_memory(source_name).with_file(
            "app/main.wui",
            format!(
                "component {component_id} {{ region workspace.region.primary {{ sizing workspace.sizing.mosaic_support; }} }}"
            ),
        ),
        [WorthUiWatcherEvent::provider_revision(source_name)],
        capabilities,
    )
}

fn focus_component_submission(
    source_name: &str,
    component_id: &str,
    capabilities: &crate::capability::CapabilitySnapshot,
) -> WorthUiWatchedCandidateSubmission {
    lower_file_submission(
        WorthUiSourceProvider::in_memory(source_name).with_file(
            "app/main.wui",
            format!(
                "component {component_id} {{ region workspace.region.primary {{ sizing workspace.sizing.mosaic_support; }} }}\nfocus workspace.focus.main {{ scope workbench; restore; reveal; }}"
            ),
        ),
        [WorthUiWatcherEvent::provider_revision(source_name)],
        capabilities,
    )
}

fn scaled_component_submission(
    source_name: &str,
    target_component_id: &str,
    unrelated_component_count: usize,
    capabilities: &crate::capability::CapabilitySnapshot,
) -> WorthUiWatchedCandidateSubmission {
    let mut declarations = vec![component_declaration(target_component_id)];
    declarations.extend(
        (0..unrelated_component_count)
            .map(scaled_unrelated_token_id)
            .map(|identity| format!("token {identity} = \"{identity}\";")),
    );
    lower_file_submission(
        WorthUiSourceProvider::in_memory(source_name)
            .with_file("app/main.wui", declarations.join("\n")),
        [WorthUiWatcherEvent::provider_revision(source_name)],
        capabilities,
    )
}

fn component_declaration(component_id: &str) -> String {
    format!(
        "component {component_id} {{ region workspace.region.primary {{ sizing workspace.sizing.mosaic_support; }} }}"
    )
}

fn scaled_unrelated_token_id(index: usize) -> String {
    format!("theme.scale.unrelated_{index}")
}

fn scaled_unrelated_token(index: usize) -> crate::capability::ThemeTokenDescriptor {
    crate::capability::ThemeTokenDescriptor::define(
        crate::capability::ThemeTokenId::new(scaled_unrelated_token_id(index))
            .expect("scaled token id is valid"),
        crate::capability::ThemeTokenFamily::text(),
        crate::capability::ThemeTokenSource::application(),
        crate::capability::ThemeTokenValue::color(
            crate::capability::UiThemeColor::parse("#101820").expect("scaled token color is valid"),
        ),
    )
}
