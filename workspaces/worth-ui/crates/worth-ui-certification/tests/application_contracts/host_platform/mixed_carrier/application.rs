use worth_ui::facade::declaration::{ComponentSemanticTextContract, ThemeTokenId};
use worth_ui_dsl::{
    WorthUiArtifactInputBodyAtom, WorthUiProjectionCollectionPolicy,
    WorthUiProjectionCollectionSelection, WorthUiProjectionLifecycle,
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule,
};
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;

use super::{MixedCarrierFixtureProfile, MountedMixedRows};
use crate::host_platform::world;

const COLLECTION_SOURCE: &str = "host.platform.mixed.collection";
const SCALAR_SOURCE: &str = "host.platform.mixed.scalar";
const INSTALLED_PROJECTION: &str = "platform.pulse.status";
const INSTALLED_SCALAR_PROJECTION: &str = "host.platform.mixed.scalar.view";

pub(super) fn build(
    profile: MixedCarrierFixtureProfile,
    collection: worth_ui_query_binding::UiCollectionProjectionRegistration,
    scalar: worth_ui_query_binding::UiScalarProjectionRegistration,
    recorder: worth_ui_host_headless::WorthUiHeadlessRecorder,
) -> worth_ui::facade::app::WorthUiApp {
    let (builder, module) = (0..profile.rectangle_component_count).fold(
        (
            world::appearance::register(world::application_builder(recorder)),
            world::appearance::declare_roles(WorthUiRustAuthoredArtifactInputModule::new(
                "app/main.wui",
            )),
        ),
        |(builder, module), index| {
            let component = world::component_identity(index);
            let token = world::token_identity(index);
            let descriptor = if index <= 1 {
                world::component(&component, index).with_semantic_text(
                    ComponentSemanticTextContract::body_default(
                        ThemeTokenId::new(token.clone()).unwrap(),
                        2_048,
                    ),
                )
            } else {
                world::component(&component, index)
            };
            let module = world::appearance::attach(
                authored_component(module, &component, &token, index),
                &component,
                index,
            );
            (
                builder
                    .register_theme_token(world::color_token(&token, world::color(index)))
                    .register_component(descriptor),
                module,
            )
        },
    );
    let module = module
        .try_with_query_collection_text(
            COLLECTION_SOURCE,
            INSTALLED_PROJECTION,
            "identity.id",
            WorthUiProjectionCollectionSelection::new(
                ["status"],
                WorthUiProjectionLifecycle::Live,
                WorthUiProjectionCollectionPolicy::new(false, false),
            ),
        )
        .expect("mixed carrier collection query compiles")
        .try_with_query_scalar_text(
            SCALAR_SOURCE,
            INSTALLED_SCALAR_PROJECTION,
            "status",
            WorthUiProjectionLifecycle::Live,
        )
        .expect("mixed carrier scalar query compiles");
    builder
        .register_collection_projection(collection)
        .expect("mixed carrier collection registers")
        .register_scalar_projection(scalar)
        .expect("mixed carrier scalar projection registers")
        .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([module]))
        .freeze()
        .expect("mixed carrier application freezes")
}

fn authored_component(
    module: WorthUiRustAuthoredArtifactInputModule,
    component: &str,
    token: &str,
    index: usize,
) -> WorthUiRustAuthoredArtifactInputModule {
    let module = module.with_token(token, world::color(index));
    if index <= 1 {
        let source = if index == 0 {
            COLLECTION_SOURCE
        } else {
            SCALAR_SOURCE
        };
        module.with_component_body_atoms_and_authored_identity(
            component,
            format!("host-platform-mixed-{index:04}"),
            vec![
                WorthUiArtifactInputBodyAtom::Identifier("content".to_owned()),
                WorthUiArtifactInputBodyAtom::Identifier("projection".to_owned()),
                WorthUiArtifactInputBodyAtom::Identifier(source.to_owned()),
            ],
        )
    } else {
        module
            .with_component_authored_identity(component, format!("host-platform-mixed-{index:04}"))
    }
}

pub(super) fn mount(
    profile: MixedCarrierFixtureProfile,
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> MountedMixedRows {
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            worth_ui_host_contract::UiHostSurfacePresentationMode::RecordOnly,
            crate::mounted_application_lifecycle::known_empty_surface_world::profile(1),
        )
        .unwrap();
    let graph = session.graph();
    let mut nodes = graph
        .node_identities()
        .filter_map(|identity| session.mounted_graph_node(identity).ok())
        .filter_map(|node| {
            let lookup = graph.lookup().graph_node(node.graph_node_identity())?;
            let name = lookup
                .value()
                .declaration_identity()
                .authored_semantic_name()
                .to_owned();
            name.starts_with("component:host.platform.maximum.rect_")
                .then_some((name, node))
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(nodes.len(), profile.rectangle_component_count);
    let scalar_node = nodes[1].1;
    let mut rows = nodes
        .into_iter()
        .map(|(_, node)| {
            let instance = session.mount_instance(node, surface).unwrap();
            super::MountedMixedRow { node, instance }
        })
        .collect::<Vec<_>>();
    for _ in 1..profile.scalar_instance_count {
        rows.push(super::MountedMixedRow {
            node: scalar_node,
            instance: session.mount_instance(scalar_node, surface).unwrap(),
        });
    }
    assert_eq!(rows.len(), profile.rectangle_count);
    MountedMixedRows { surface, rows }
}
