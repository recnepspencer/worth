use worth_ui::facade::appearance::UiAppearanceRoleAttachmentDeclaration;
use worth_ui::facade::declaration::*;
use worth_ui_dsl::{
    WorthUiArtifactInputBodyAtom, WorthUiProjectionLifecycle, WorthUiRustAuthoredArtifactInput,
    WorthUiRustAuthoredArtifactInputModule,
};

const TEXT_COLOR: &str = "theme.appearance.scale.text";
const TEXT_SOURCE: &str = "appearance.scale.text";
pub(super) const TEXT_NODE_COUNT: usize = 16;

pub(super) fn build(
    recorder: worth_ui_host_headless::WorthUiHeadlessRecorder,
    scalar: worth_ui_query_binding::UiScalarProjectionRegistration,
) -> worth_ui::facade::app::WorthUiApp {
    let builder = worth_ui::facade::app::WorthUi::app().with_change_profile(change_profile());
    let mut builder = worth_ui_certification::scenario::application_authority_closure::FixedCertificationApplicationBuilder::new(builder, recorder)
        .register_appearance_theme_bundle(super::appearance::theme_bundle())
        .unwrap()
        .register_theme_token(super::super::world::color_token(TEXT_COLOR, "#ffffff"))
        .register_scalar_projection(scalar)
        .unwrap();
    let mut module = WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
        .with_token(TEXT_COLOR, "#ffffff")
        .try_with_query_scalar_text(
            TEXT_SOURCE,
            "platform.pulse.status",
            "status",
            WorthUiProjectionLifecycle::Live,
        )
        .unwrap();
    for role in (0..super::ROLE_COUNT).map(super::appearance::role) {
        module = module.with_appearance_role(role.clone());
        builder = builder.register_appearance_role(role).unwrap();
    }
    for index in 0..super::NODE_COUNT {
        let identity = component_identity(index);
        let mut component = ComponentDescriptor::new(
            ComponentId::new(&identity).unwrap(),
            ComponentPropSchema::named(format!("{identity}.props")),
            ComponentChildPolicy::no_children(),
            ComponentStateOwnership::runtime_owned(),
        )
        .with_allocation_measurement_contract(
            ComponentAllocationMeasurementContract::fill_viewport(),
        )
        .with_surface_paint_order(index as u32);
        if index < TEXT_NODE_COUNT {
            component = component.with_semantic_text(ComponentSemanticTextContract::body_default(
                ThemeTokenId::new(TEXT_COLOR).unwrap(),
                2_048,
            ));
            module = module.with_component_body_atoms_and_authored_identity(
                &identity,
                authored_identity(index),
                vec![
                    WorthUiArtifactInputBodyAtom::Identifier("content".to_owned()),
                    WorthUiArtifactInputBodyAtom::Identifier("projection".to_owned()),
                    WorthUiArtifactInputBodyAtom::Identifier(TEXT_SOURCE.to_owned()),
                ],
            );
        } else {
            module = module.with_component_authored_identity(&identity, authored_identity(index));
        }
        if index < super::STYLED_COUNT {
            let role = super::appearance::role(component_role_index(index));
            component = component
                .with_appearance_aspect_contract(role.aspect_contract().clone())
                .unwrap();
            module = module
                .with_component_appearance_role(
                    &identity,
                    UiAppearanceRoleAttachmentDeclaration::new(
                        role.role().clone(),
                        role.revision(),
                    ),
                )
                .unwrap();
        }
        builder = builder.register_component(component);
    }
    builder
        .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([module]))
        .freeze()
        .expect("AP10 distinct declarations and appearance capabilities freeze")
}

pub(super) fn component_role_index(index: usize) -> usize {
    if index < 3_000 {
        index / super::CONSUMERS_PER_ROLE
    } else {
        8 + (index - 3_000) / super::CONSUMERS_PER_ROLE
    }
}

pub(super) fn component_identity(index: usize) -> String {
    format!("appearance.scale.node_{index:04}")
}

fn authored_identity(index: usize) -> String {
    format!(
        "appearance-scale-neighborhood-{:02}-node-{index:04}",
        index / 64
    )
}

fn change_profile() -> worth_ui::facade::rebind::UiChangeProfile {
    use worth_ui::facade::rebind::{UiChangeProfile, UiRebindProfile};
    let baseline = UiChangeProfile::platform_pulse();
    let mut budget = baseline.rebind().budget();
    // Each of at most 64 consumers contributes its graph/eligibility relation,
    // graph/mounted lookup, and predecessor/candidate scope membership.
    budget.graph_and_mounted_entries = 64 * 6;
    budget.native_surfaces = super::SURFACE_COUNT;
    UiChangeProfile::new(
        baseline.observation(),
        UiRebindProfile::bounded(budget, baseline.rebind().concurrency()).unwrap(),
    )
}
