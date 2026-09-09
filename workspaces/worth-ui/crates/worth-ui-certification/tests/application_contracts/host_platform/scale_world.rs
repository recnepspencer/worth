use std::collections::HashSet;

use worth_ui::facade::declaration::{
    ComponentChildPolicy, ComponentDescriptor, ComponentId, ComponentPropSchema,
    ComponentStateOwnership,
};
use worth_ui_dsl::{WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule};
use worth_ui_runtime::facade::mounted::{
    UiMountedFrameOutcome, UiMountedFrameRequest, UiPresentationDeadline,
};
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;

const MOUNTED_NODE_COUNT: usize = 4_096;
const PAINTED_NODE_COUNT: usize = 2_048;

pub(super) fn verify() -> usize {
    let recorder = worth_ui_host_headless::WorthUiHeadlessRecorder::with_viewport_extent(
        worth_ui_host_headless::UiHeadlessRecorderCapacity::new(1, 2, 16_384),
        worth_ui::facade::measurement_exchange::UiViewportExtentObservation {
            width: 160.0,
            height: 96.0,
        },
    );
    let (builder, module) = (0..4).fold(
        (
            super::world::application_builder(recorder.clone()),
            WorthUiRustAuthoredArtifactInputModule::new("app/main.wui"),
        ),
        |(builder, module), index| {
            let identity = super::world::component_identity(index);
            let token = super::world::token_identity(index);
            let descriptor = if index < 2 {
                super::world::component(&identity, index)
            } else {
                unpainted_component(&identity)
            };
            (
                builder
                    .register_theme_token(super::world::color_token(
                        &token,
                        super::world::color(index),
                    ))
                    .register_component(descriptor),
                module
                    .with_token(token, super::world::color(index))
                    .with_component_authored_identity(
                        identity,
                        format!("host-platform-scale-node-{index}"),
                    ),
            )
        },
    );
    let app = builder
        .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([module]))
        .freeze()
        .expect("scale application freezes");
    let mut session = app.launch().expect("scale application launches");
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            worth_ui_host_contract::UiHostSurfacePresentationMode::RecordOnly,
            crate::mounted_application_lifecycle::known_empty_surface_world::profile(1),
        )
        .unwrap();
    let nodes = {
        let graph = session.graph();
        graph
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
            .collect::<Vec<_>>()
    };
    assert_eq!(nodes.len(), 4);
    let mut instances = HashSet::with_capacity(MOUNTED_NODE_COUNT);
    for node in nodes {
        for _ in 0..MOUNTED_NODE_COUNT / 4 {
            instances.insert(session.mount_instance(node, surface).unwrap());
        }
    }
    assert_eq!(instances.len(), MOUNTED_NODE_COUNT);
    super::world::establish_allocations(&mut session, 2);
    let frame = session.execute_mounted_frame(
        UiMountedFrameRequest::all_bound_surfaces(),
        UiPresentationDeadline::at_tick(3_150),
        0,
        |_| {},
    );
    match frame {
        Ok(UiMountedFrameOutcome::Published(_)) => {}
        Ok(_) => panic!("scale frame did not publish"),
        Err(worth_ui::facade::app::WorthUiMountedFrameExecutionStop::PublicationLease(denial)) => {
            panic!("scale publication lease denied: {denial:?}")
        }
        Err(worth_ui::facade::app::WorthUiMountedFrameExecutionStop::HostMeasurement(denial)) => {
            panic!("scale host measurement denied: {denial:?}")
        }
        Err(
            worth_ui::facade::app::WorthUiMountedFrameExecutionStop::HostMeasurementTransition(
                denial,
            ),
        ) => panic!("scale host measurement transition denied: {denial:?}"),
        Err(worth_ui::facade::app::WorthUiMountedFrameExecutionStop::FrameworkTransition(_)) => {
            panic!("scale framework transition stopped")
        }
        Err(worth_ui::facade::app::WorthUiMountedFrameExecutionStop::OccurrenceGeometry(
            denial,
        )) => panic!("scale occurrence geometry denied: {denial:?}"),
        Err(worth_ui::facade::app::WorthUiMountedFrameExecutionStop::Preparation(denial)) => {
            panic!("scale frame preparation denied: {denial:?}")
        }
    }
    let mut transcripts = recorder.drain_transcripts().into_vec();
    assert_eq!(transcripts.len(), 1);
    let transcript = transcripts.pop().unwrap();
    assert_eq!(transcript.nodes().len(), MOUNTED_NODE_COUNT);
    assert_eq!(transcript.filled_rects().len(), PAINTED_NODE_COUNT);
    assert_eq!(transcript.paint_order().len(), PAINTED_NODE_COUNT);
    assert_eq!(
        transcript
            .filled_rects()
            .iter()
            .map(|mechanic| mechanic.command_identity())
            .collect::<HashSet<_>>()
            .len(),
        PAINTED_NODE_COUNT
    );
    assert!(transcript.semantic_text().is_empty());
    let shutdown = session.shutdown();
    assert!(shutdown.mounted_presentation().is_empty());
    transcript.nodes().len()
}

fn unpainted_component(identity: &str) -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(identity).unwrap(),
        ComponentPropSchema::named(format!("{identity}.props")),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
}
