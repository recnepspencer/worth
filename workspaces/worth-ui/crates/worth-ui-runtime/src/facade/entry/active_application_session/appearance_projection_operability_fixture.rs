use super::super::{support, test_support};
use crate::facade::intent::*;
use worth_ui_dsl::*;

pub(super) const ROUTE: &str = "test.appearance.activate";
pub(super) const MUTABLE: &str = "test.appearance.mutable";
pub(super) const POLICY: &str = "test.appearance.policy";
pub(super) struct Payload;
pub(super) struct Outcome;
pub(super) struct Intent;
struct Provider;

impl UiIntentPayload for Payload {
    const SCHEMA: UiIntentSchema = UiIntentSchema::stable("test.appearance.payload", 1);
    const FIELDS: UiIntentPayloadFieldSet = UiIntentPayloadFieldSet::EMPTY;
    fn project(
        _: &mut UiIntentPayloadProjection<Self>,
    ) -> Result<Self, UiIntentPayloadProjectionViolation> {
        Ok(Self)
    }
}
impl UiIntentProductOutcome for Outcome {
    const SCHEMA: UiIntentSchema = UiIntentSchema::stable("test.appearance.outcome", 1);
    const CONSEQUENCE_FAMILIES: UiIntentProductConsequenceFamilies =
        UiIntentProductConsequenceFamilies::NONE;
    fn into_consequences(self) -> UiIntentProductConsequences {
        UiIntentProductConsequences::none()
    }
}
impl UiIntent for Intent {
    type Payload = Payload;
    type ProductOutcome = Outcome;
    const ID: UiIntentId = UiIntentId::stable("test.appearance.intent");
    const ACCEPTED_INTERACTIONS: UiIntentAcceptedInteractions =
        UiIntentAcceptedInteractions::new(&[
            UiSemanticInteractionFamily::Activate,
            UiSemanticInteractionFamily::Submit,
        ]);
}
impl UiIntentExecutionProvider<Intent> for Provider {
    const VERSION: UiIntentProviderVersion = UiIntentProviderVersion::stable(1);
    fn begin(&self, _: UiIntentExecutionRequest<Intent>) -> UiIntentProviderStart<Intent> {
        panic!("appearance must never execute the product provider")
    }
}

pub(super) fn fact(name: &str) -> UiIntentApplicationFact<UiIntentBoolean> {
    UiIntentApplicationFact::boolean(name).unwrap()
}

pub(super) fn role() -> UiAppearanceRoleDeclaration {
    let mut table = UiAppearancePartitionAuthoring::new([UiAppearanceAxisDomain::complete(
        UiAppearanceStateAxis::Operability,
    )]);
    for (class, red) in [
        (UiAppearanceAxisClass::OperabilityReady, 10),
        (UiAppearanceAxisClass::OperabilityDenied, 20),
        (UiAppearanceAxisClass::OperabilityPending, 30),
        (UiAppearanceAxisClass::OperabilityOccupied, 40),
        (UiAppearanceAxisClass::OperabilityUnsupported, 50),
        (UiAppearanceAxisClass::OperabilityStale, 60),
    ] {
        table = table.with_cell(
            UiAppearanceCell::when([UiAppearanceAxisPredicate::exact(class)]).literal(
                UiThemeValue::Color(UiThemeColor::from_channels([red, 0, 0, 255])),
            ),
        );
    }
    UiAppearanceRoleDeclaration::admit(
        UiAppearanceRoleIdentity::new("test.operability-background").unwrap(),
        UiAppearanceRoleRevision::new(1).unwrap(),
        UiAppearanceRoleApplicability::AnyComponent,
        &UiAppearanceAspectContract::component([UiAppearanceAspect::Background], []).unwrap(),
        [(
            UiAppearanceAspect::Background,
            table.compile(UiAppearanceAspect::Background).unwrap(),
        )],
    )
    .unwrap()
}

pub(super) fn source(
    role: &UiAppearanceRoleDeclaration,
    routes: usize,
) -> WorthUiRustAuthoredArtifactInput {
    source_with_role(Some(role), routes)
}

pub(super) fn source_with_role(
    role: Option<&UiAppearanceRoleDeclaration>,
    routes: usize,
) -> WorthUiRustAuthoredArtifactInput {
    let mut module = WorthUiRustAuthoredArtifactInputModule::new("appearance/consumer");
    let mut bindings = Vec::new();
    for (identity, family) in [
        (ROUTE, WorthUiIntentInteractionFamily::Activate),
        (
            "test.appearance.submit",
            WorthUiIntentInteractionFamily::Submit,
        ),
    ]
    .into_iter()
    .take(routes)
    {
        module = module.with_intent_declaration(WorthUiIntentDeclarationSpec::new(
            identity,
            "test.appearance.intent",
            family,
            WorthUiIntentOperabilityContractSpec::new(
                "test.appearance.operability",
                WorthUiIntentMutabilitySourceSpec::application_boolean(MUTABLE),
                WorthUiIntentReadinessSourceSpec::application_boolean("test.appearance.ready"),
                WorthUiIntentPolicySourceSpec::application_boolean(POLICY),
            ),
            WorthUiIntentConfirmationContractSpec::not_required("test.appearance.confirmation"),
            WorthUiIntentConcurrencyScope::TargetRouteSingleFlight,
            WorthUiIntentConsequenceContractSpec::none(),
        ));
        bindings.push(WorthUiIntentInteractionRoute::product(family, identity));
    }
    module = module.with_control_routes(support::APPEARANCE_NODE_A, bindings);
    if let Some(role) = role {
        module = module
            .with_appearance_role(role.clone())
            .with_component_appearance_role(
                support::APPEARANCE_NODE_A,
                UiAppearanceRoleAttachmentDeclaration::new(role.role().clone(), role.revision()),
            )
            .unwrap();
    }
    WorthUiRustAuthoredArtifactInput::from_modules([module])
}

pub(super) fn builder(
    role: &UiAppearanceRoleDeclaration,
) -> crate::facade::entry::WorthUiApplicationBuilder {
    let token = crate::capability::ThemeTokenId::new(support::LEGACY_STATIC_PAINT_TOKEN).unwrap();
    builder_with_component(
        role,
        support::static_paint_component(support::APPEARANCE_NODE_A, token).with_hit_test(
            crate::capability::ComponentHitTestContract::allocation_bounds(
                crate::capability::ComponentHitTestOrder::front_to_back(0),
                crate::capability::ComponentAllocationMeasurementContract::fill_viewport(),
            ),
        ),
    )
}

pub(super) fn builder_with_component(
    role: &UiAppearanceRoleDeclaration,
    component: crate::capability::ComponentDescriptor,
) -> crate::facade::entry::WorthUiApplicationBuilder {
    use crate::runtime::tests::source_ingress_boundary_test_support::{
        source_backed_package_region, source_backed_package_sizing,
    };
    let (_, _, world) = crate::evidence::measurement::projection::fact_test_support::display_field_projection_context("appearance-operability");
    let token = crate::capability::ThemeTokenId::new(support::LEGACY_STATIC_PAINT_TOKEN).unwrap();
    crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .with_graph_world_profile(world)
        .register_component(component)
        .register_theme_token(support::appearance_theme_token(token))
        .register_mosaic_region_kind(source_backed_package_region())
        .register_mosaic_sizing_contract(source_backed_package_sizing())
        .register_appearance_role(role.clone())
        .unwrap()
        .register_appearance_theme_bundle(test_support::theme_bundle())
        .unwrap()
        .register_intent_boolean_fact(fact(MUTABLE), true)
        .unwrap()
        .register_intent_boolean_fact(fact("test.appearance.ready"), true)
        .unwrap()
        .register_intent_boolean_fact(fact(POLICY), true)
        .unwrap()
        .register_intent_definition(UiIntentDefinition::<Intent>::application_effect())
        .unwrap()
        .register_intent_provider(Provider)
        .unwrap()
}

pub(super) fn candidate(
    session: &crate::facade::WorthUiActiveApplicationSession,
    role: &UiAppearanceRoleDeclaration,
    routes: usize,
    name: &str,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    lower(session.capabilities(), role, routes, name)
}

fn lower(
    capabilities: &crate::capability::CapabilitySnapshot,
    role: &UiAppearanceRoleDeclaration,
    routes: usize,
    name: &str,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission(
        crate::runtime::WorthUiSourceProvider::rust_authored(name)
            .with_rust_authored_input(source(role, routes)),
        [crate::runtime::WorthUiWatcherEvent::provider_revision(name)],
        capabilities,
    )
}

pub(super) fn session(
    role: &UiAppearanceRoleDeclaration,
    routes: usize,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    session_with_source(role, source(role, routes))
}

pub(super) fn session_with_source(
    role: &UiAppearanceRoleDeclaration,
    source: WorthUiRustAuthoredArtifactInput,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::staged_appearance_capability_report());
    let observer = host.clone();
    let capabilities = builder(role).freeze().unwrap();
    let launch = crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission(
        crate::runtime::WorthUiSourceProvider::rust_authored("operability-launch")
            .with_rust_authored_input(source),
        [crate::runtime::WorthUiWatcherEvent::provider_revision(
            "operability-launch",
        )],
        capabilities.capabilities(),
    );
    let app = builder(role)
        .with_candidate_submission(launch)
        .freeze()
        .unwrap();
    let session =
        crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
            app, host,
        )
        .launch()
        .unwrap();
    (session, observer)
}
