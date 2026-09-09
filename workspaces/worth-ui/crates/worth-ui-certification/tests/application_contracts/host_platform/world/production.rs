use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentChildPolicy, ComponentDescriptor, ComponentId,
    ComponentPropSchema, ComponentStateOwnership, ComponentStaticPaintContract,
    ComponentStaticPaintOrder, ComponentViewportInset, ThemeColorValue, ThemeTokenDescriptor,
    ThemeTokenFamily, ThemeTokenId, ThemeTokenSource, ThemeTokenValue,
};
use worth_ui::facade::measurement_exchange::{
    UiMeasurementEvidenceFamily, UiViewportExtentRequest,
};
use worth_ui_dsl::{WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule};
use worth_ui_runtime::facade::entry::UiMountedAllocationMeasurementRequest;
use worth_ui_runtime::facade::host::{
    UiHostMeasurementAssumptionProfile, UiHostMeasurementNeed,
    UiHostMeasurementNormalizationContext,
};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedFrameOutcome, UiMountedFrameRequest,
    UiPresentationDeadline,
};
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiMountedAllocationCertificationExt,
    WorthUiMountedIdentityCertificationExt,
};

const RECTANGLE_COUNT: usize = 2_048;
const BLUE: &str = "#2f81f7";
const YELLOW: &str = "#f2cc60";

pub(in crate::host_platform) struct ProducedMaximumOverlap {
    pub session: worth_ui::facade::app::WorthUiActiveApplicationSession,
    pub initial: worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
    pub deltas: Box<[ProducedMaximumDelta]>,
    pub unchanged: ProducedUnchanged,
    pub restorations: Box<[ProducedMaximumDelta]>,
    pub authored_instances: Box<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
    pub semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
}

pub(in crate::host_platform) struct ProducedMaximumDelta {
    pub changed_rows: usize,
    pub transcript: worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
    pub authored_instances: Box<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
    pub delta_rows_carried: u64,
    pub draw_mutations: u64,
    pub order_mutations: u64,
    pub damage_regions: u64,
}

pub(in crate::host_platform) struct ProducedUnchanged {
    pub cost: worth_ui_host_contract::UiHostPresentationCostReport,
    pub native_work_count: usize,
}

struct MountedMaximumRows {
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    rows: Vec<MountedMaximumRow>,
}

struct MountedMaximumRow {
    node: worth_ui_runtime::facade::mounted::UiMountedGraphNodeHandle,
    instance: worth_ui_host_contract::UiMountedInstanceIdentity,
}

pub(crate) fn produce_maximum_overlap(
    recorder: worth_ui_host_headless::WorthUiHeadlessRecorder,
) -> ProducedMaximumOverlap {
    let app = build_application(recorder.clone());
    let mut session = app.launch().expect("maximum-overlap application launches");
    let mut mounted = mount_maximum_overlap(&mut session);
    establish_allocations(&mut session, RECTANGLE_COUNT);
    execute_frame(&mut session, 10);
    let initial = one_transcript(&recorder, "maximum-overlap initial transcript");
    let unchanged = produce_unchanged(&mut session, &recorder, 11);
    let authored_instances = mounted
        .rows
        .iter()
        .map(|row| row.instance)
        .collect::<Vec<_>>();
    let mut deltas = Vec::new();
    let mut restorations = Vec::new();
    for (index, count) in [1, RECTANGLE_COUNT / 2, RECTANGLE_COUNT]
        .into_iter()
        .enumerate()
    {
        deltas.push(produce_removal_delta(
            &mut session,
            &recorder,
            &mounted,
            count,
            20 + index as u64 * 2,
        ));
        if count != RECTANGLE_COUNT {
            restore_rows(&mut session, &mut mounted, count);
            restorations.push(produce_restoration(
                &mut session,
                &recorder,
                &mounted,
                count,
                21 + index as u64 * 2,
            ));
        }
    }
    ProducedMaximumOverlap {
        session,
        initial,
        deltas: deltas.into_boxed_slice(),
        unchanged,
        restorations: restorations.into_boxed_slice(),
        authored_instances: authored_instances.into_boxed_slice(),
        semantic_surface: mounted.surface,
    }
}

fn produce_unchanged(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    recorder: &worth_ui_host_headless::WorthUiHeadlessRecorder,
    tick: u64,
) -> ProducedUnchanged {
    let cost = execute_frame(session, tick);
    let native_work_count = recorder.drain_transcripts().len();
    ProducedUnchanged {
        cost,
        native_work_count,
    }
}

fn produce_restoration(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    recorder: &worth_ui_host_headless::WorthUiHeadlessRecorder,
    mounted: &MountedMaximumRows,
    count: usize,
    tick: u64,
) -> ProducedMaximumDelta {
    let cost = execute_frame(session, tick);
    ProducedMaximumDelta {
        changed_rows: count,
        transcript: one_transcript(recorder, "maximum-overlap restoration transcript"),
        authored_instances: mounted
            .rows
            .iter()
            .map(|row| row.instance)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        delta_rows_carried: cost.delta_rows_carried(),
        draw_mutations: cost.draw_list_mutations(),
        order_mutations: cost.order_mutations(),
        damage_regions: cost.logical_damage_regions(),
    }
}

fn produce_removal_delta(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    recorder: &worth_ui_host_headless::WorthUiHeadlessRecorder,
    mounted: &MountedMaximumRows,
    count: usize,
    tick: u64,
) -> ProducedMaximumDelta {
    for row in mounted.rows.iter().take(count) {
        session.unmount_instance(row.instance).unwrap();
    }
    let cost = execute_frame(session, tick);
    ProducedMaximumDelta {
        changed_rows: count,
        transcript: one_transcript(recorder, "maximum-overlap removal transcript"),
        authored_instances: mounted
            .rows
            .iter()
            .skip(count)
            .map(|row| row.instance)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        delta_rows_carried: cost.delta_rows_carried(),
        draw_mutations: cost.draw_list_mutations(),
        order_mutations: cost.order_mutations(),
        damage_regions: cost.logical_damage_regions(),
    }
}

fn restore_rows(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    mounted: &mut MountedMaximumRows,
    count: usize,
) {
    for row in mounted.rows.iter_mut().take(count) {
        row.instance = session.mount_instance(row.node, mounted.surface).unwrap();
    }
}

fn one_transcript(
    recorder: &worth_ui_host_headless::WorthUiHeadlessRecorder,
    context: &str,
) -> worth_ui_host_headless::UiHeadlessMountedFrameTranscript {
    let transcripts = recorder.drain_transcripts().into_vec();
    assert_eq!(transcripts.len(), 1, "{context}");
    transcripts.into_iter().next().unwrap()
}

fn build_application(
    recorder: worth_ui_host_headless::WorthUiHeadlessRecorder,
) -> worth_ui::facade::app::WorthUiApp {
    let (builder, module) = (0..RECTANGLE_COUNT).fold(
        (
            application_builder(recorder),
            WorthUiRustAuthoredArtifactInputModule::new("app/main.wui"),
        ),
        |(builder, module), index| {
            let identity = component_identity(index);
            (
                builder
                    .register_theme_token(color_token(&token_identity(index), color(index)))
                    .register_component(component(&identity, index)),
                module
                    .with_token(token_identity(index), color(index))
                    .with_component_authored_identity(
                        identity,
                        format!("host-platform-maximum-{index:04}"),
                    ),
            )
        },
    );
    builder
        .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([module]))
        .freeze()
        .expect("maximum-overlap application freezes")
}

fn mount_maximum_overlap(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> MountedMaximumRows {
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            super::super::super::mounted_application_lifecycle::known_empty_surface_world::profile(
                1,
            ),
        )
        .unwrap();
    let graph = session.graph();
    let nodes = graph
        .node_identities()
        .filter_map(|identity| session.mounted_graph_node(identity).ok())
        .filter(|handle| {
            graph
                .lookup()
                .graph_node(handle.graph_node_identity())
                .is_some_and(|lookup| {
                    lookup
                        .value()
                        .declaration_identity()
                        .authored_semantic_name()
                        .starts_with("component:host.platform.maximum.rect_")
                })
        })
        .collect::<Vec<_>>();
    assert_eq!(nodes.len(), RECTANGLE_COUNT);
    let rows = nodes
        .into_iter()
        .map(|node| MountedMaximumRow {
            instance: session.mount_instance(node, surface).unwrap(),
            node,
        })
        .collect();
    MountedMaximumRows { surface, rows }
}

pub(in crate::host_platform) fn execute_frame(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    tick: u64,
) -> worth_ui_host_contract::UiHostPresentationCostReport {
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    let outcome = match session.execute_mounted_frame(
        UiMountedFrameRequest::all_bound_surfaces(),
        UiPresentationDeadline::at_tick(tick),
        0,
        |_| {},
    ) {
        Ok(outcome) => outcome,
        Err(_) => panic!("maximum-overlap frame stopped"),
    };
    match outcome {
        UiMountedFrameOutcome::Published(publication) => publication.cost_report().adapter(),
        _ => panic!("maximum-overlap frame did not publish"),
    }
}

pub(in crate::host_platform) fn application_builder(
    recorder: worth_ui_host_headless::WorthUiHeadlessRecorder,
) -> worth_ui_certification::scenario::application_authority_closure::FixedCertificationApplicationBuilder{
    let builder = worth_ui::facade::app::WorthUi::app()
        .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse());
    worth_ui_certification::scenario::application_authority_closure::FixedCertificationApplicationBuilder::new(
        builder,
        recorder,
    )
}

pub(in crate::host_platform) fn component(identity: &str, index: usize) -> ComponentDescriptor {
    let allocation = if index == 1 {
        ComponentAllocationMeasurementContract::viewport_inset(ComponentViewportInset::symmetric(
            48, 24,
        ))
    } else {
        ComponentAllocationMeasurementContract::fill_viewport()
    };
    ComponentDescriptor::new(
        ComponentId::new(identity).unwrap(),
        ComponentPropSchema::named(format!("{identity}.props")),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_static_paint(
        ComponentStaticPaintContract::opaque_fill(
            ThemeTokenId::new(token_identity(index)).unwrap(),
            ComponentStaticPaintOrder::back_to_front(index as u32),
        ),
        allocation,
    )
}

pub(in crate::host_platform) fn color_token(identity: &str, value: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        ThemeTokenId::new(identity).unwrap(),
        ThemeTokenFamily::surface(),
        ThemeTokenSource::application(),
        ThemeTokenValue::color(ThemeColorValue::hex(value).unwrap()),
    )
}

pub(in crate::host_platform) fn component_identity(index: usize) -> String {
    format!("host.platform.maximum.rect_{index:04}")
}

pub(in crate::host_platform) fn token_identity(index: usize) -> String {
    format!("host.platform.maximum.color_{index:04}")
}

pub(in crate::host_platform) fn color(index: usize) -> &'static str {
    if index == 1 {
        YELLOW
    } else {
        BLUE
    }
}

pub(in crate::host_platform) fn establish_allocations(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    expected_nodes: usize,
) {
    let capability = session.host_measurement_capability();
    let assumptions = UiHostMeasurementAssumptionProfile::from_capability_report(
        capability.capability_report(),
        1,
        2,
        3,
        4,
    );
    let request = UiMountedAllocationMeasurementRequest::new(
        UiMeasurementEvidenceFamily::ViewportExtent,
        UiHostMeasurementNeed::ViewportExtent(UiViewportExtentRequest),
        UiHostMeasurementNormalizationContext::viewport_logical_exact(assumptions),
    );
    let receipt = session
        .establish_mounted_allocation_catalog(1, [request])
        .expect("maximum-overlap allocation catalog");
    assert_eq!(receipt.committed().receipts().len(), expected_nodes);
}
