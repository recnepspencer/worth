use crate::facade::WorthUiApp;
use crate::runtime::tests::source_ingress_boundary_test_support::lower_file_submission;
use crate::runtime::{WorthUiSourceProvider, WorthUiWatcherEvent};

pub(crate) fn source_backed_declared_surface_component_app_with_host<Host>(host: Host) -> WorthUiApp
where
    Host: crate::facade::host::WorthUiHostAdapter + 'static,
{
    let snapshot = builder()
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("declared-surface component snapshot should prepare");
    let submission = lower_file_submission(
        WorthUiSourceProvider::in_memory("declared-surface-active-session-current").with_file(
            "app/main.wui",
            "surface workspace.surface.native {}\ncomponent workspace.component.active_session_current { region workspace.region.primary { sizing workspace.sizing.mosaic_support; } }",
        ),
        [WorthUiWatcherEvent::provider_revision(
            "declared-surface-active-session-current",
        )],
        snapshot.capabilities(),
    );
    builder()
        .with_candidate_submission(submission)
        .freeze()
        .map(|application| {
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                application,
                host,
            )
        })
        .expect("declared-surface component source application should prepare")
}

fn builder() -> crate::facade::entry::WorthUiApplicationBuilder {
    super::component_builder().register_surface(crate::capability::SurfaceDescriptor::new(
        crate::capability::SurfaceId::new("workspace.surface.native").unwrap(),
        crate::capability::SurfaceKind::primary_content(),
        crate::capability::ComponentId::new("workspace.component.active_session_current").unwrap(),
        crate::capability::SurfacePlacementClass::primary_region(),
        crate::capability::SurfaceStateClass::restorable(),
    ))
}
