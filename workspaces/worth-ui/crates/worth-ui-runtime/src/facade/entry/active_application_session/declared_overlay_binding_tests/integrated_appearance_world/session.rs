use super::super::{appearance_publication_support as overlay, test_support};
use super::{authored, palette};
use crate::capability::*;
use crate::certification_support::ScriptedPresentationHost;
use crate::facade::WorthUiActiveApplicationSession;
use crate::mounting::UiPreparedMountedFrame;
use crate::mounting::{UiSurfaceBindingCoordinatePosture, UiSurfaceBindingProfile};
use worth_ui_host_contract::*;

#[path = "application_capabilities.rs"]
mod application_capabilities;
use application_capabilities::builder;

/// What a launched World declares about scrolling: the Scroll service policy
/// its wheel follows, and the primary region kind whose content scrolls.
pub(super) struct WorldScroll {
    pub(super) policy: crate::declaration::UiScrollPolicy,
    pub(super) region: MosaicRegionKindDescriptor,
}

impl Default for WorldScroll {
    fn default() -> Self {
        Self {
            policy: crate::declaration::UiScrollPolicy::nested_region(),
            region: crate::runtime::tests::source_ingress_boundary_test_support::source_backed_package_region(),
        }
    }
}

/// The capability shape one World launches with.
pub(super) struct WorldCapabilities {
    pub(super) seam: bool,
    pub(super) multi_region_owner: bool,
    pub(super) role: Option<worth_ui_dsl::UiAppearanceRoleDeclaration>,
    pub(super) scroll: WorldScroll,
}

impl WorldCapabilities {
    fn shared(seam: bool, multi_region_owner: bool) -> Self {
        Self {
            seam,
            multi_region_owner,
            role: None,
            scroll: WorldScroll::default(),
        }
    }
}

pub(super) struct World {
    pub(super) session: WorthUiActiveApplicationSession,
    pub(super) host: ScriptedPresentationHost,
    pub(super) surfaces: [UiSemanticSurfaceIdentity; 2],
    pub(super) instances: [UiMountedInstanceIdentity; 5],
    pub(super) graphs: [crate::graph::UiGraphNodeIdentity; 4],
}

impl World {
    pub(super) fn launch() -> Self {
        Self::launch_with_seam(false)
    }

    pub(super) fn launch_ap07(
        registration: worth_ui_query_binding::UiCollectionProjectionRegistration,
        source: String,
    ) -> Self {
        Self::launch_with_projection_budget(
            WorldCapabilities::shared(false, false),
            source,
            None,
            Some(registration),
            Default::default(),
        )
    }

    pub(super) fn launch_seam() -> Self {
        Self::launch_with_seam(true)
    }

    pub(super) fn launch_multi_region_seam() -> Self {
        Self::launch_with_source(true, true, authored::multi_region_seam_source())
    }

    fn launch_with_seam(seam: bool) -> Self {
        let source = if seam {
            authored::seam_source()
        } else {
            authored::source()
        };
        Self::launch_with_source(seam, false, source)
    }

    pub(super) fn launch_with_source(seam: bool, multi_region_owner: bool, source: String) -> Self {
        Self::launch_with_projection(seam, multi_region_owner, source, None)
    }

    pub(super) fn launch_with_projection(
        seam: bool,
        multi_region_owner: bool,
        source: String,
        scalar: Option<(
            ComponentDescriptor,
            worth_ui_query_binding::UiScalarProjectionRegistration,
        )>,
    ) -> Self {
        Self::launch_with_projection_budget(
            WorldCapabilities::shared(seam, multi_region_owner),
            source,
            scalar,
            None,
            Default::default(),
        )
    }

    pub(super) fn launch_with_retention_budget(
        budget: crate::mounting::UiMountedFrameRetentionBudget,
    ) -> Self {
        Self::launch_with_projection_budget(
            WorldCapabilities::shared(false, false),
            authored::source(),
            None,
            None,
            budget,
        )
    }

    /// A World whose Scroll service and primary region are the caller's, with
    /// or without a Motion service to publish settles to.
    pub(super) fn launch_with_scroll(scroll: WorldScroll) -> Self {
        Self::launch_with_projection_budget(
            WorldCapabilities {
                scroll,
                ..WorldCapabilities::shared(false, false)
            },
            authored::source(),
            None,
            None,
            Default::default(),
        )
    }

    pub(super) fn launch_with_appearance_role(
        source: String,
        role: worth_ui_dsl::UiAppearanceRoleDeclaration,
    ) -> Self {
        Self::launch_with_projection_budget(
            WorldCapabilities {
                role: Some(role),
                ..WorldCapabilities::shared(false, false)
            },
            source,
            None,
            None,
            Default::default(),
        )
    }

    fn launch_with_projection_budget(
        capabilities: WorldCapabilities,
        source: String,
        scalar: Option<(
            ComponentDescriptor,
            worth_ui_query_binding::UiScalarProjectionRegistration,
        )>,
        collection: Option<worth_ui_query_binding::UiCollectionProjectionRegistration>,
        budget: crate::mounting::UiMountedFrameRetentionBudget,
    ) -> Self {
        let configured = || {
            let builder = builder(&capabilities).with_mounted_frame_retention_budget(budget);
            let builder = match &scalar {
                Some((component, registration)) => builder
                    .register_component(component.clone())
                    .register_scalar_projection(registration.clone())
                    .unwrap(),
                None => builder,
            };
            match &collection {
                Some(registration) => {
                    super::hostile_protocol::configure_builder(builder, registration.clone())
                }
                None => builder,
            }
        };
        let snapshot = configured().freeze()
            .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host).unwrap();
        let submission =
            crate::runtime::tests::source_ingress_boundary_test_support::lower_file_submission(
                crate::runtime::WorthUiSourceProvider::in_memory("integrated-overlay")
                    .with_file("app/main.wui", &source),
                [crate::runtime::WorthUiWatcherEvent::provider_revision(
                    "integrated-overlay",
                )],
                snapshot.capabilities(),
            );
        let host = ScriptedPresentationHost::native_display();
        host.set_capabilities(worth_ui_host_native::appearance_capability_report());
        let observer = host.clone();
        let mut session = configured().with_candidate_submission(submission).freeze().map(|app| {
            let mut app = crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(app, host);
            let installation = worth_ui_query_binding::WorthUiPresentationAsyncHostPlan::prepare()
                .unwrap()
                .install_for_certification()
                .unwrap();
            app.install_presentation_async(installation).unwrap();
            app
        }).unwrap().launch().unwrap();
        let bindings = session
            .application
            .authored_overlay_material()
            .overlay_declaration_bindings();
        let declarations = ["workspace.surface.overlay", "workspace.surface.secondary"]
            .map(|name| bindings.surface_named(name).unwrap());
        let regions = ["workspace.surface.overlay", "workspace.surface.secondary"].map(|surface| {
            bindings
                .region_named(surface, "workspace.region.primary")
                .unwrap()
        });
        let surfaces = declarations.map(|declaration| {
            let surface = session
                .create_declared_semantic_surface(declaration)
                .unwrap();
            session
                .register_host_surface(
                    surface,
                    UiHostSurfacePresentationMode::NativeDisplay,
                    UiSurfaceBindingProfile::new(
                        1_000,
                        UiSurfaceBindingCoordinatePosture::LogicalPoints,
                        1,
                    )
                    .unwrap(),
                )
                .unwrap();
            surface
        });
        let graphs = authored::COMPONENTS.map(|component| {
            let (graph, semantic) = {
                let graph = session.graph();
                graph
                    .node_identities()
                    .find_map(|identity| {
                        let lookup = graph.lookup().graph_node(identity)?;
                        let semantic = lookup
                            .value()
                            .declaration_identity()
                            .authored_semantic_name()
                            .to_owned();
                        (semantic == format!("component:{component}"))
                            .then_some((identity, semantic))
                    })
                    .unwrap()
            };
            session
                .register_application_semantic_text(semantic.into_boxed_str(), graph)
                .unwrap();
            graph
        });
        let instances = [(0, 0), (1, 0), (2, 0), (0, 1), (3, 0)].map(|(index, surface_index)| {
            let surface = surfaces[surface_index];
            let node = session.mounted_graph_node(graphs[index]).unwrap();
            session.mount_instance(node, surface).unwrap()
        });
        let semantic_text = authored::COMPONENTS
            .iter()
            .map(|component| {
                crate::native_platform::UiNativeComponentSemanticTextChange::new(
                    format!("component:{component}"),
                    "AB",
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        session
            .admit_application_semantic_text(&semantic_text)
            .unwrap();
        overlay::establish_allocation(&mut session);
        super::geometry::install(&mut session, surfaces, instances, regions);
        overlay::close_source_with(&mut session, &source);
        Self {
            session,
            host: observer,
            surfaces,
            instances,
            graphs,
        }
    }

    pub(super) fn prepare(&mut self) -> UiPreparedMountedFrame {
        let request = self.session.mounted_frame_request();
        self.prepare_request(request)
    }

    pub(super) fn prepare_surface(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
    ) -> UiPreparedMountedFrame {
        self.prepare_request(crate::mounting::UiMountedFrameRequest::exact_surfaces(
            vec![surface],
        ))
    }

    pub(super) fn prepare_surface_with_current_portals(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
    ) -> UiPreparedMountedFrame {
        let (revision, overlays) = self.session.portal.as_ref().map_or_else(
            || (0, Vec::new()),
            |portal| {
                (
                    portal.revision(),
                    portal.current_mounted_projection_inputs(),
                )
            },
        );
        assert!(
            !overlays.is_empty(),
            "the current Portal set remains mounted"
        );
        let request = crate::mounting::UiMountedFrameRequest::exact_surfaces(vec![surface])
            .with_portal_overlays(revision, overlays);
        self.prepare_request(request)
    }

    fn prepare_request(
        &mut self,
        request: crate::mounting::UiMountedFrameRequest,
    ) -> UiPreparedMountedFrame {
        self.session
            .prepare_mounted_frame_with_application_presentation(request, |_| {})
            .unwrap_or_else(|stop| match stop {
                crate::facade::entry::WorthUiMountedFrameExecutionStop::Preparation(denial) => {
                    panic!("shared authored frame: {denial:?}")
                }
                _ => panic!("shared authored frame stops before preparation"),
            })
    }
}
