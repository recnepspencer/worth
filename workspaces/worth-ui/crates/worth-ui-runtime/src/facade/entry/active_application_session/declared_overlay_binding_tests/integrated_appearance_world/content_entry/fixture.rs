use crate::capability::*;
use crate::certification_support::ScriptedPresentationHost;
use crate::facade::WorthUiActiveApplicationSession;
use crate::mounting::*;
use crate::runtime::tests::source_ingress_boundary_test_support as source;
use worth_ui_host_contract::*;

#[path = "assertions.rs"]
mod assertions;
#[path = "reconstruction.rs"]
mod reconstruction;

const COMPONENT: &str = "workspace.component.publication_text";
const SOURCE: &str = "component workspace.component.publication_text { appearance { role text.content } }\nappearance role text.content applies_to workspace.component.publication_text { foreground use token(overlay.content.foreground) }";

pub(super) struct TextWorld {
    pub session: WorthUiActiveApplicationSession,
    pub host: ScriptedPresentationHost,
    pub surfaces: [UiSemanticSurfaceIdentity; 2],
    pub graph: crate::graph::UiGraphNodeIdentity,
    pub occurrences: Vec<(UiMountedInstanceIdentity, usize, [f32; 4])>,
    accepted: [Option<(
        UiHostObservationPresentationBasis,
        UiMountedPresentationAttemptIdentity,
        std::rc::Rc<UiMountedProjectionFrame>,
    )>; 2],
}

impl TextWorld {
    pub fn launch() -> Self {
        let snapshot = builder().freeze().map(
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host,
        ).unwrap();
        let submission = source::lower_file_submission(
            crate::runtime::WorthUiSourceProvider::in_memory("text-publication")
                .with_file("app/main.wui", SOURCE),
            [crate::runtime::WorthUiWatcherEvent::provider_revision(
                "text-publication",
            )],
            snapshot.capabilities(),
        );
        let host = ScriptedPresentationHost::native_display();
        host.set_capabilities(worth_ui_host_native::appearance_capability_report());
        let observer = host.clone();
        let mut session = builder()
            .with_candidate_submission(submission)
            .freeze()
            .map(|app| {
                let mut app = crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                    app, host,
                );
                let installation = worth_ui_query_binding::WorthUiPresentationAsyncHostPlan::prepare()
                    .unwrap().install_for_certification().unwrap();
                app.install_presentation_async(installation).unwrap();
                app
            })
            .unwrap()
            .launch()
            .unwrap();
        let graph = session
            .graph()
            .node_identities()
            .find(|identity| {
                session
                    .graph()
                    .lookup()
                    .graph_node(*identity)
                    .unwrap()
                    .value()
                    .declaration_identity()
                    .authored_semantic_name()
                    == format!("component:{COMPONENT}")
            })
            .unwrap();
        session
            .register_application_semantic_text(
                format!("component:{COMPONENT}").into_boxed_str(),
                graph,
            )
            .unwrap();
        let surfaces = [0, 1].map(|_| {
            let surface = session.create_semantic_surface().unwrap();
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
        let mut world = Self {
            session,
            host: observer,
            surfaces,
            graph,
            occurrences: Vec::new(),
            accepted: [None, None],
        };
        world.add_occurrence(0, [20.0, 30.0, 160.0, 40.0]);
        world.add_occurrence(1, [50.0, 60.0, 120.0, 30.0]);
        super::super::super::appearance_publication_support::establish_allocation(
            &mut world.session,
        );
        world.admit(0, "AB");
        super::super::super::appearance_publication_support::close_source_with(
            &mut world.session,
            SOURCE,
        );
        world
    }

    pub fn admit(&mut self, revision: u64, text: &str) {
        self.session
            .admit_application_semantic_text(&[
                crate::native_platform::UiNativeComponentSemanticTextChange::successor(
                    format!("component:{COMPONENT}"),
                    revision,
                    text,
                )
                .unwrap(),
            ])
            .unwrap();
    }

    pub fn add_occurrence(
        &mut self,
        surface_index: usize,
        bounds: [f32; 4],
    ) -> UiMountedInstanceIdentity {
        let node = self.session.mounted_graph_node(self.graph).unwrap();
        let instance = self
            .session
            .mount_instance(node, self.surfaces[surface_index])
            .unwrap();
        self.occurrences.push((instance, surface_index, bounds));
        self.install_geometry(surface_index);
        instance
    }

    fn install_geometry(&mut self, index: usize) {
        let surface = self.surfaces[index];
        let geometry: Vec<_> = self
            .occurrences
            .iter()
            .filter(|(_, member, _)| *member == index)
            .map(|(instance, _, bounds)| {
                UiMountedOccurrenceGeometry::surface(*instance, rectangle(*bounds))
            })
            .collect();
        let revision = self
            .session
            .mounted
            .next_occurrence_geometry_revision_for_test(surface);
        let mut layout = self.session.begin_mounted_layout();
        let basis = layout.basis(surface).unwrap();
        layout
            .complete_surface_geometry(UiMountedSurfaceGeometryBatch::new(
                basis,
                revision,
                rectangle([0.0, 0.0, 1280.0, 720.0]),
                geometry,
            ))
            .unwrap();
    }

    pub fn replace_occurrence(&mut self, position: usize) -> UiMountedInstanceIdentity {
        let (retired, surface, bounds) = self.occurrences.remove(position);
        self.session.unmount_instance(retired).unwrap();
        self.add_occurrence(surface, bounds)
    }

    pub fn launch_scaled(count: usize) -> Self {
        let mut world = Self::launch();
        let node = world.session.mounted_graph_node(world.graph).unwrap();
        for index in 0..count {
            let bounds = [
                (index % 24) as f32 * 50.0,
                (index / 24) as f32 * 25.0,
                40.0,
                20.0,
            ];
            if index == 0 {
                world.occurrences[0].2 = bounds;
            } else {
                let instance = world
                    .session
                    .mount_instance(node, world.surfaces[0])
                    .unwrap();
                world.occurrences.push((instance, 0, bounds));
            }
        }
        world.install_geometry(0);
        world
    }

    pub fn execute(&mut self, indices: &[usize], tick: u64, accept: bool) -> UiMountCostReport {
        for _ in indices {
            if accept {
                self.host.push_native_display_presented();
            } else {
                self.host.push_rejected();
            }
        }
        self.execute_scripted(indices, tick, accept)
    }

    pub fn execute_scripted(
        &mut self,
        indices: &[usize],
        tick: u64,
        accept: bool,
    ) -> UiMountCostReport {
        let request = UiMountedFrameRequest::exact_surfaces(
            indices.iter().map(|i| self.surfaces[*i]).collect(),
        );
        let outcome = self
            .session
            .execute_mounted_frame(
                request,
                UiPresentationDeadline::at_tick(u64::MAX),
                tick,
                |_| {},
            )
            .unwrap_or_else(|stop| match stop {
                crate::facade::entry::WorthUiMountedFrameExecutionStop::Preparation(denial) => {
                    panic!("text preparation: {denial:?}")
                }
                _ => panic!("text preparation stopped before host admission"),
            });
        let cost = outcome
            .cost_report()
            .expect("publication attempts report their performed work");
        match outcome {
            UiMountedFrameOutcome::Published(receipt) if accept => {
                assert_eq!(
                    self.session.current_mounted_publication().unwrap().frame(),
                    receipt.frame()
                );
                for index in indices {
                    assert_eq!(
                        self.session
                            .mounted
                            .current_presentation_for_surface(self.surfaces[*index])
                            .unwrap()
                            .frame(),
                        receipt.frame()
                    );
                    self.accepted[*index] = Some((
                        self.session
                            .mounted
                            .current_presentation_for_surface(self.surfaces[*index])
                            .unwrap(),
                        receipt.attempt(),
                        self.session
                            .mounted
                            .current_projection_rc_for_test()
                            .unwrap(),
                    ));
                }
            }
            UiMountedFrameOutcome::RejectedBeforeEffects(_) if !accept => {}
            UiMountedFrameOutcome::RejectedBeforeEffects(denial) => {
                panic!("text host rejection at {tick}: {:?}", denial.rejections())
            }
            UiMountedFrameOutcome::AdmissionDenied(denial) => {
                panic!("text publication: {:?}", denial.denial())
            }
            UiMountedFrameOutcome::PresentationIndeterminate(frame) => {
                panic!("text presentation at {tick}: {:?}", frame.report())
            }
            other => panic!(
                "unexpected text publication at {tick}: {:?}",
                std::mem::discriminant(&other)
            ),
        }
        cost
    }
}

fn rectangle([x, y, width, height]: [f32; 4]) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap()
}

fn builder() -> crate::facade::entry::WorthUiApplicationBuilder {
    use worth_ui_dsl::*;
    let role = UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new("text.content").unwrap())
        .applies_to_component(UiDslComponentReference::new(COMPONENT).unwrap())
        .cover(
            UiAppearanceAspect::Foreground,
            UiAppearancePartitionAuthoring::new([]).with_cell(
                UiAppearanceCell::when([]).uses_slot(
                    UiThemeSlotIdentity::new("overlay.content.foreground").unwrap(),
                    UiThemeValueKind::Color,
                ),
            ),
        )
        .unwrap()
        .build()
        .unwrap();
    let component = source::source_backed_package_component(COMPONENT)
        .with_allocation_measurement_contract(
            ComponentAllocationMeasurementContract::fill_viewport(),
        )
        .with_semantic_text(super::super::authored::text_contract())
        .with_appearance_aspect_contract(role.aspect_contract().clone())
        .unwrap();
    let (_, _, profile) = crate::evidence::measurement::projection::fact_test_support::display_field_projection_context("text-publication");
    crate::facade::WorthUi::app().with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .with_graph_world_profile(profile).register_component(component)
        .register_theme_token(crate::runtime::tests::appearance_component_session_test_support::appearance_theme_token(
            ThemeTokenId::new(super::super::palette::TEXT_TOKEN).unwrap()))
        .register_appearance_role(role).unwrap()
        .register_appearance_theme_bundle(super::super::palette::theme()).unwrap()
}
