use super::super::{appearance_publication_support as overlay, test_support};
use super::{authored, palette};
use crate::capability::*;
use crate::certification_support::{ScriptedPresentationHost, ScriptedSurfaceCompletion};
use crate::facade::WorthUiActiveApplicationSession;
use crate::mounting::{UiMountedFrameOutcome, UiPreparedMountedFrame};
use crate::mounting::{UiSurfaceBindingCoordinatePosture, UiSurfaceBindingProfile};
use worth_ui_host_contract::*;

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

    fn launch_with_source(seam: bool, multi_region_owner: bool, source: String) -> Self {
        let snapshot = builder(seam, multi_region_owner).freeze()
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
        host.set_capabilities(worth_ui_host_native::staged_appearance_capability_report());
        let observer = host.clone();
        let mut session = builder(seam, multi_region_owner).with_candidate_submission(submission).freeze().map(|app| {
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

    pub(super) fn publish(&mut self, frame: UiPreparedMountedFrame, now: u64, initial: bool) {
        for _ in frame.surfaces() {
            if initial {
                self.host.push_native_display_presented();
            } else {
                self.host.push_native_display_settled_without_effects();
            }
        }
        let outcome = self.session.present_prepared_mounted_frame_internal(
            frame,
            UiPresentationDeadline::at_tick(u64::MAX),
            now,
        );
        match outcome {
            UiMountedFrameOutcome::Published(_) => {}
            UiMountedFrameOutcome::AdmissionDenied(denial) => {
                panic!("shared publication: {:?}", denial.denial())
            }
            UiMountedFrameOutcome::RejectedBeforeEffects(denial) => {
                panic!("shared publication: {:?}", denial.rejections())
            }
            other => panic!("shared publication: {:?}", std::mem::discriminant(&other)),
        }
    }

    pub(super) fn publish_in_flight(&mut self, frame: UiPreparedMountedFrame, now: u64) {
        assert_eq!(
            frame.surfaces().len(),
            1,
            "shared in-flight proof is one surface"
        );
        let predecessor = self.session.current_mounted_publication().unwrap().frame();
        let appearance = self
            .session
            .mounted
            .current_unpublished_appearance()
            .unwrap()
            .cloned();
        self.host.push_in_flight(
            vec![ScriptedSurfaceCompletion::Presented(
                UiMountedSurfacePresentationCompletion::new(
                    UiHostSurfacePresentationMode::NativeDisplay,
                    UiHostPresentationEpoch::issued_by_host(now + 1_000),
                    UiMountedCompletedEffects::new(Vec::new()),
                    UiHostPresentationCostReport::default(),
                ),
            )],
            UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
        );
        let outcome = self.session.present_prepared_mounted_frame_internal(
            frame,
            UiPresentationDeadline::at_tick(u64::MAX),
            now,
        );
        let pending = match outcome {
            UiMountedFrameOutcome::InFlight(pending) => pending,
            UiMountedFrameOutcome::AdmissionDenied(denial) => {
                panic!("shared in-flight admission: {:?}", denial.denial())
            }
            UiMountedFrameOutcome::RejectedBeforeEffects(denial) => {
                panic!("shared in-flight rejection: {:?}", denial.rejections())
            }
            other => panic!(
                "shared authored publication must enter the legal in-flight posture: {:?}",
                std::mem::discriminant(&other)
            ),
        };
        assert_eq!(
            self.session.current_mounted_publication().unwrap().frame(),
            predecessor,
            "the accepted predecessor stays authoritative while host work is pending"
        );
        assert_eq!(
            self.session
                .mounted
                .current_unpublished_appearance()
                .unwrap(),
            appearance.as_ref(),
            "pending text and overlay work cannot replace accepted appearance"
        );
        let completed = self.session.complete_mounted_presentation(pending, now + 1);
        match completed {
            UiMountedFrameOutcome::Published(_) => {}
            UiMountedFrameOutcome::PresentationIndeterminate(frame) => {
                panic!("shared in-flight completion: {:?}", frame.report())
            }
            other => panic!(
                "shared in-flight completion: {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }
}

fn builder(
    seam: bool,
    multi_region_owner: bool,
) -> crate::facade::entry::WorthUiCertificationApplicationBuilder {
    let mut builder = test_support::authored_overlay_builder_with_component(authored::component(0))
        .with_focus_policy_defaults(crate::declaration::UiFocusPolicy::workbench())
        .with_motion_policy_defaults(crate::declaration::UiMotionPolicy::system_respecting())
        .with_scroll_policy_defaults(crate::declaration::UiScrollPolicy::nested_region())
        .register_surface(SurfaceDescriptor::new(SurfaceId::new("workspace.surface.secondary").unwrap(),
            SurfaceKind::overlay_content(), ComponentId::new(authored::COMPONENTS[0]).unwrap(),
            SurfacePlacementClass::overlay_layer(), SurfaceStateClass::restorable()))
        .register_theme_token(crate::runtime::tests::appearance_component_session_test_support::appearance_theme_token(ThemeTokenId::new(palette::TEXT_TOKEN).unwrap()));
    if seam {
        let primary = MosaicRegionKindId::new("workspace.region.primary").unwrap();
        let secondary = MosaicRegionKindId::new("workspace.region.secondary").unwrap();
        let edge = MosaicSharedEdge::new(primary.clone(), secondary.clone()).unwrap();
        let contract = MosaicSeamPaintContract::admit(
            [primary.clone(), secondary.clone()],
            [edge.clone()],
            [MosaicSeamPaintOwner::new(edge, primary.clone()).unwrap()],
            [
                MosaicExteriorCorner::new(primary.clone(), MosaicExteriorCornerPosture::TopLeft),
                MosaicExteriorCorner::new(primary, MosaicExteriorCornerPosture::BottomLeft),
                MosaicExteriorCorner::new(secondary.clone(), MosaicExteriorCornerPosture::TopRight),
                MosaicExteriorCorner::new(secondary, MosaicExteriorCornerPosture::BottomRight),
            ],
        )
        .unwrap();
        let secondary_region = if multi_region_owner {
            authored::secondary_region_allowing_escape()
        } else {
            authored::secondary_region()
        };
        builder = builder
            .register_mosaic_region_kind(secondary_region)
            .register_mosaic_seam_paint_contract(contract)
            .unwrap();
    }
    for index in 0..authored::COMPONENTS.len() {
        if index != 0 {
            builder = builder.register_component(authored::component(index));
        }
        builder = builder
            .register_appearance_role(authored::role(index))
            .unwrap();
    }
    builder
        .register_appearance_role(overlay::backdrop_role())
        .unwrap()
        .register_appearance_theme_bundle(palette::theme())
        .unwrap()
}
