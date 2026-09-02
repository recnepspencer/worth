use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentChildPolicy, ComponentDescriptor,
    ComponentFocusSupport, ComponentHitTestContract, ComponentHitTestOrder, ComponentId,
    ComponentPropSchema, ComponentStateOwnership, ComponentViewportInset,
};
use worth_ui::facade::intent::{
    UiIntentConcurrencyScope, UiIntentConfirmationContract, UiIntentConsequenceContract,
    UiIntentDeclaration, UiIntentDefinition, UiIntentMutabilitySource, UiIntentOperabilityContract,
    UiIntentPolicySource, UiIntentReadinessSource, UiIntentRuntimeServiceDestination,
};
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_dsl::{
    WorthUiArtifactInputBodyAtom, WorthUiIntentInteractionFamily, WorthUiIntentInteractionRoute,
    WorthUiProjectionLifecycle, WorthUiRustAuthoredArtifactInput,
    WorthUiRustAuthoredArtifactInputModule,
};
use worth_ui_host_headless::{UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder};
use worth_ui_runtime::facade::measurement_exchange::UiViewportExtentObservation;

use super::facts::OperabilityFacts;
use super::intent_types::PrimaryIntent;
use super::topology::{module, PRIMARY_DECLARATION};

const CONTRACT: &str = "phase3.portal.operability";
const CONFIRMATION_POLICY: &str = "phase3.portal.confirmation-policy";
const PROJECTION: &str = "platform.pulse.status";
const PAINT_ONLY: &str = "visual.identity.component.paint_only";
const HIT_ONLY: &str = "visual.identity.component.hit_only";
const PAINT_AND_HIT: &str = "visual.identity.component.paint_and_hit";
const SECOND_FOCUS: &str = "phase3.portal.component.second_focus";
const NEITHER: &str = "visual.identity.component.neither";
const SURFACE: &str = "visual.identity.surface.main";
const PAINT_ONLY_TOKEN: &str = "theme.visual_identity.paint_only";
const PAINT_AND_HIT_TOKEN: &str = "theme.visual_identity.paint_and_hit";

pub(in crate::intent) fn build_open_portal_application(
    capacity: UiHeadlessRecorderCapacity,
) -> (
    worth_ui::facade::app::WorthUiApp,
    OperabilityFacts,
    WorthUiHeadlessRecorder,
) {
    let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
        capacity,
        UiViewportExtentObservation {
            width: 640.0,
            height: 480.0,
        },
    );
    let (app, facts) = build_open_portal_application_with_host(recorder.clone());
    (app, facts, recorder)
}

pub(in crate::intent) fn build_open_portal_application_with_host<Host>(
    host: Host,
) -> (worth_ui::facade::app::WorthUiApp, OperabilityFacts)
where
    Host: worth_ui_certification::scenario::application_authority_closure::fixed_host::FixedCertificationHostBinding,
{
    let facts = OperabilityFacts::new();
    let input = module(
        PRIMARY_DECLARATION,
        PRIMARY_DECLARATION,
        WorthUiIntentInteractionFamily::Activate,
        [declaration(&facts)],
    );
    let app = FilesystemApplicationLifecycleScenario::new("phase-3-portal-service-world")
        .portal_semantic_text_action_application_builder(host)
        .register_intent_boolean_fact(facts.mutability.clone(), true)
        .unwrap()
        .register_intent_boolean_fact(facts.readiness.clone(), true)
        .unwrap()
        .register_intent_boolean_fact(facts.policy.clone(), true)
        .unwrap()
        .register_intent_boolean_fact(facts.confirmation.clone(), false)
        .unwrap()
        .register_runtime_service_intent_definition(
            UiIntentDefinition::<PrimaryIntent>::runtime_service(
                UiIntentRuntimeServiceDestination::OpenPortal,
            ),
        )
        .unwrap()
        .with_rust_authored_input(input)
        .freeze()
        .expect("the real portal-service definition freezes through production preparation");
    (app, facts)
}

pub(in crate::intent) fn build_open_portal_two_focus_application_with_host<Host>(
    host: Host,
) -> (worth_ui::facade::app::WorthUiApp, OperabilityFacts)
where
    Host: worth_ui_certification::scenario::application_authority_closure::fixed_host::FixedCertificationHostBinding,
{
    let facts = OperabilityFacts::new();
    let route = WorthUiIntentInteractionRoute::product(
        WorthUiIntentInteractionFamily::Activate,
        PRIMARY_DECLARATION,
    );
    let input = WorthUiRustAuthoredArtifactInput::from_modules([
        WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
            .with_component(PAINT_ONLY)
            .with_control_routes(HIT_ONLY, [route.clone()])
            .with_control_routes(PAINT_AND_HIT, [route.clone()])
            .with_control_routes(SECOND_FOCUS, [route])
            .with_component(NEITHER)
            .with_surface(SURFACE)
            .with_token(PAINT_ONLY_TOKEN, "theme.visual_identity.red")
            .with_token(PAINT_AND_HIT_TOKEN, "theme.visual_identity.purple")
            .with_intent_declaration(declaration(&facts)),
    ]);
    let app = FilesystemApplicationLifecycleScenario::new("phase-3-portal-two-focus-world")
        .portal_semantic_text_action_application_builder(host)
        .register_component(
            ComponentDescriptor::new(
                ComponentId::new(SECOND_FOCUS).expect("valid second focus component id"),
                ComponentPropSchema::named("phase3.portal.second-focus.props"),
                ComponentChildPolicy::no_children(),
                ComponentStateOwnership::runtime_owned(),
            )
            .with_hit_test(ComponentHitTestContract::allocation_bounds(
                ComponentHitTestOrder::front_to_back(2),
                ComponentAllocationMeasurementContract::viewport_inset(
                    ComponentViewportInset::symmetric(24, 20),
                ),
            ))
            .with_focus(ComponentFocusSupport::focusable()),
        )
        .register_intent_boolean_fact(facts.mutability.clone(), true)
        .unwrap()
        .register_intent_boolean_fact(facts.readiness.clone(), true)
        .unwrap()
        .register_intent_boolean_fact(facts.policy.clone(), true)
        .unwrap()
        .register_intent_boolean_fact(facts.confirmation.clone(), false)
        .unwrap()
        .register_runtime_service_intent_definition(
            UiIntentDefinition::<PrimaryIntent>::runtime_service(
                UiIntentRuntimeServiceDestination::OpenPortal,
            ),
        )
        .unwrap()
        .with_rust_authored_input(input)
        .freeze()
        .expect("the two-focus portal definition freezes through production preparation");
    (app, facts)
}

pub(in crate::intent) fn portal_owner_removal_input(
    facts: &OperabilityFacts,
) -> WorthUiRustAuthoredArtifactInput {
    let route = WorthUiIntentInteractionRoute::product(
        WorthUiIntentInteractionFamily::Activate,
        PRIMARY_DECLARATION,
    );
    WorthUiRustAuthoredArtifactInput::from_modules([WorthUiRustAuthoredArtifactInputModule::new(
        "app/main.wui",
    )
    .with_component(PAINT_ONLY)
    .with_control_routes(HIT_ONLY, [route.clone()])
    .with_component(NEITHER)
    .with_surface(SURFACE)
    .with_token(PAINT_ONLY_TOKEN, "theme.visual_identity.red")
    .with_token(PAINT_AND_HIT_TOKEN, "theme.visual_identity.purple")
    .with_intent_declaration(declaration(facts))])
}

pub(in crate::intent) fn build_open_portal_projection_application_with_host<Host>(
    host: Host,
    registration: worth_ui::facade::query_binding::UiScalarProjectionRegistration,
) -> (worth_ui::facade::app::WorthUiApp, OperabilityFacts)
where
    Host: worth_ui_certification::scenario::application_authority_closure::fixed_host::FixedCertificationHostBinding,
{
    let facts = OperabilityFacts::new();
    let input = projected_module(&facts);
    let app = FilesystemApplicationLifecycleScenario::new("phase-3-portal-projection-world")
        .portal_semantic_text_action_application_builder(host)
        .register_scalar_projection(registration)
        .expect("the product Query projection registration matches its declaration")
        .register_intent_boolean_fact(facts.mutability.clone(), true)
        .unwrap()
        .register_intent_boolean_fact(facts.readiness.clone(), true)
        .unwrap()
        .register_intent_boolean_fact(facts.policy.clone(), true)
        .unwrap()
        .register_intent_boolean_fact(facts.confirmation.clone(), false)
        .unwrap()
        .register_runtime_service_intent_definition(
            UiIntentDefinition::<PrimaryIntent>::runtime_service(
                UiIntentRuntimeServiceDestination::OpenPortal,
            ),
        )
        .unwrap()
        .with_rust_authored_input(input)
        .freeze()
        .expect("the portal and Query projection compile into one production application");
    (app, facts)
}

fn projected_module(facts: &OperabilityFacts) -> WorthUiRustAuthoredArtifactInput {
    let projection_and_route = [
        WorthUiArtifactInputBodyAtom::Identifier("content".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("projection".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier(PROJECTION.to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("interaction".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("activate".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("routes".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier(PRIMARY_DECLARATION.to_owned()),
    ];
    let module = WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
        .with_component(PAINT_ONLY)
        .with_control_routes(
            HIT_ONLY,
            [WorthUiIntentInteractionRoute::product(
                WorthUiIntentInteractionFamily::Activate,
                PRIMARY_DECLARATION,
            )],
        )
        .with_component_body_atoms(PAINT_AND_HIT, projection_and_route)
        .with_component(NEITHER)
        .with_surface(SURFACE)
        .with_token(PAINT_ONLY_TOKEN, "theme.visual_identity.red")
        .with_token(PAINT_AND_HIT_TOKEN, "theme.visual_identity.purple")
        .try_with_query_scalar_text(
            PROJECTION,
            PROJECTION,
            "status",
            WorthUiProjectionLifecycle::Live,
        )
        .expect("the product scalar projection declaration is valid")
        .with_intent_declaration(declaration(facts));
    WorthUiRustAuthoredArtifactInput::from_modules([module])
}

fn declaration(facts: &OperabilityFacts) -> worth_ui_dsl::WorthUiIntentDeclarationSpec {
    UiIntentDeclaration::<PrimaryIntent>::activate(PRIMARY_DECLARATION)
        .unwrap()
        .operability_from(
            UiIntentOperabilityContract::new(
                CONTRACT,
                UiIntentMutabilitySource::application_fact(&facts.mutability),
                UiIntentReadinessSource::application_fact(&facts.readiness),
                UiIntentPolicySource::application_fact(&facts.policy),
            )
            .unwrap(),
        )
        .confirmation(
            UiIntentConfirmationContract::application_fact(
                CONFIRMATION_POLICY,
                &facts.confirmation,
            )
            .unwrap(),
        )
        .concurrency(UiIntentConcurrencyScope::TargetRouteSingleFlight)
        .consequences(UiIntentConsequenceContract::mounted_posture())
        .into_dsl_spec()
}
