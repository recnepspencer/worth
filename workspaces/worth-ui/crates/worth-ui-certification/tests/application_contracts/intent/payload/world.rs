use std::sync::Arc;

use worth_ui::facade::intent::{
    UiIntent, UiIntentApplicationFact, UiIntentBoolean, UiIntentConcurrencyScope,
    UiIntentConfirmationContract, UiIntentConsequenceContract, UiIntentDeclaration,
    UiIntentDefinition, UiIntentMutabilitySource, UiIntentOperabilityContract,
    UiIntentPolicySource, UiIntentReadinessSource, UiIntentRuntimeServiceDestination, UiIntentText,
    UiIntentUnsigned64,
};
use worth_ui_certification::scenario::application_authority_closure::fixed_host::FixedCertificationHostBinding;
use worth_ui_dsl::{
    WorthUiIntentInteractionFamily, WorthUiIntentInteractionRoute,
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule,
};
use worth_ui_host_headless::{UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder};
use worth_ui_query_binding::{
    UiCollectionProjectionRegistration, UiProjectionInputSlot, UiScalarProjectionRegistration,
    WorthUiQueryBindingPlan,
};
use worth_ui_runtime::facade::measurement_exchange::UiViewportExtentObservation;
use worth_ui_runtime::facade::mounted::UiHostSurfacePresentationMode;

use super::super::super::filesystem_mounted_world::{
    component_graph_nodes, launch_mounted_components,
};
use super::super::interaction_world::InteractionWorld;
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;

type BoundBuilder = worth_ui_certification::scenario::application_authority_closure::FixedCertificationApplicationBuilder;

#[path = "world/scroll_portal.rs"]
mod scroll_portal;
pub(in crate::intent) use scroll_portal::{launch_scroll_portal, routed_scroll_selection_input};

pub(in crate::intent) const DECLARATION: &str = "phase3.payload.route";
const PAINT_ONLY: &str = "visual.identity.component.paint_only";
const HIT_ONLY: &str = "visual.identity.component.hit_only";
const PAINT_AND_HIT: &str = "visual.identity.component.paint_and_hit";
const NEITHER: &str = "visual.identity.component.neither";
const SURFACE: &str = "visual.identity.surface.main";
const PAINT_ONLY_TOKEN: &str = "theme.visual_identity.paint_only";
const PAINT_AND_HIT_TOKEN: &str = "theme.visual_identity.paint_and_hit";
const OPERABILITY_FACT: &str = "phase3.payload.operable";
const OPERABILITY_CONTRACT: &str = "phase3.payload.operability";
const CONFIRMATION_POLICY: &str = "phase3.payload.confirmation";

pub(in crate::intent) struct PayloadWorld {
    pub(in crate::intent) interaction: InteractionWorld,
    pub(super) projection_slot: Option<UiProjectionInputSlot>,
}

#[derive(Clone)]
pub(in crate::intent) enum PayloadProjectionRegistration {
    None,
    Scalar(UiScalarProjectionRegistration),
    Collection(UiCollectionProjectionRegistration),
}

#[derive(Clone)]
pub(in crate::intent) struct PayloadApplicationFacts {
    operability: (UiIntentApplicationFact<UiIntentBoolean>, bool),
    text: Option<(UiIntentApplicationFact<UiIntentText>, Arc<str>)>,
    boolean: Option<(UiIntentApplicationFact<UiIntentBoolean>, bool)>,
    unsigned64: Option<(UiIntentApplicationFact<UiIntentUnsigned64>, u64)>,
}

impl PayloadApplicationFacts {
    pub(super) fn standard(
        text: UiIntentApplicationFact<UiIntentText>,
        boolean: UiIntentApplicationFact<UiIntentBoolean>,
        unsigned64: UiIntentApplicationFact<UiIntentUnsigned64>,
    ) -> Self {
        Self {
            text: Some((text, Arc::from("application-current"))),
            boolean: Some((boolean, true)),
            unsigned64: Some((unsigned64, 42)),
            ..Self::default()
        }
    }

    pub(in crate::intent) fn text(
        fact: UiIntentApplicationFact<UiIntentText>,
        initial: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            text: Some((fact, initial.into())),
            ..Self::default()
        }
    }
}

impl Default for PayloadApplicationFacts {
    fn default() -> Self {
        Self {
            operability: (operability_fact(), true),
            text: None,
            boolean: None,
            unsigned64: None,
        }
    }
}

pub(in crate::intent) fn launch<I: UiIntent>(
    input: WorthUiRustAuthoredArtifactInput,
    projection: PayloadProjectionRegistration,
    facts: PayloadApplicationFacts,
) -> PayloadWorld {
    let projection_slot = projection_slot(&projection);
    let application = prepare::<I>(input, projection, facts)
        .expect("payload world compiles through production application preparation");
    launch_prepared(application, projection_slot)
}

pub(super) fn prepare<I: UiIntent>(
    input: WorthUiRustAuthoredArtifactInput,
    projection: PayloadProjectionRegistration,
    facts: PayloadApplicationFacts,
) -> Result<
    worth_ui::facade::app::WorthUiApp,
    worth_ui::facade::app::WorthUiApplicationPreparationDenial,
> {
    let host = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::production_default(),
        UiViewportExtentObservation {
            width: 160.0,
            height: 96.0,
        },
    );
    prepare_with_host::<I, _>(input, projection, facts, host)
}

fn prepare_with_host<I, Host>(
    input: WorthUiRustAuthoredArtifactInput,
    projection: PayloadProjectionRegistration,
    facts: PayloadApplicationFacts,
    host: Host,
) -> Result<
    worth_ui::facade::app::WorthUiApp,
    worth_ui::facade::app::WorthUiApplicationPreparationDenial,
>
where
    I: UiIntent,
    Host: FixedCertificationHostBinding + 'static,
{
    let scenario = FilesystemApplicationLifecycleScenario::new("phase-3-payload-world");
    let builder = scenario
        .visual_identity_application_builder(host)
        .register_intent_definition(UiIntentDefinition::<I>::application_effect())
        .expect("typed payload definition registers")
        .register_intent_provider(
            worth_ui_certification::WorthUiCertificationBeforeEffectProvider::<I>::new(),
        )
        .expect("typed payload provider registers");
    let builder = register_projection(builder, projection);
    let builder = register_facts(builder, facts);
    builder.with_rust_authored_input(input).freeze()
}

fn launch_prepared(
    application: worth_ui::facade::app::WorthUiApp,
    projection_slot: Option<UiProjectionInputSlot>,
) -> PayloadWorld {
    let component_nodes = component_graph_nodes(&application);
    let session = launch_mounted_components(
        application,
        component_nodes,
        UiHostSurfacePresentationMode::RecordOnly,
    );
    PayloadWorld {
        interaction: InteractionWorld::from_session(session),
        projection_slot,
    }
}

pub(in crate::intent) fn routed_input<I: UiIntent>(
    declaration: UiIntentDeclaration<I>,
    family: WorthUiIntentInteractionFamily,
) -> WorthUiRustAuthoredArtifactInput {
    let module = WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
        .with_component(PAINT_ONLY)
        .with_control_routes(
            HIT_ONLY,
            [WorthUiIntentInteractionRoute::product(family, DECLARATION)],
        )
        .with_component(PAINT_AND_HIT)
        .with_component(NEITHER)
        .with_surface(SURFACE)
        .with_token(PAINT_ONLY_TOKEN, "theme.visual_identity.red")
        .with_token(PAINT_AND_HIT_TOKEN, "theme.visual_identity.purple")
        .with_intent_declaration(bind_operability(declaration));
    WorthUiRustAuthoredArtifactInput::from_modules([module])
}
fn projection_slot(projection: &PayloadProjectionRegistration) -> Option<UiProjectionInputSlot> {
    let plan = match projection {
        PayloadProjectionRegistration::None => return None,
        PayloadProjectionRegistration::Scalar(registration) => WorthUiQueryBindingPlan::default()
            .register_scalar_projection(registration.clone())
            .expect("scenario scalar projection produces one installed plan"),
        PayloadProjectionRegistration::Collection(registration) => {
            WorthUiQueryBindingPlan::default()
                .register_collection_projection(registration.clone())
                .expect("scenario collection projection produces one installed plan")
        }
    };
    let identity = match projection {
        PayloadProjectionRegistration::None => unreachable!(),
        PayloadProjectionRegistration::Scalar(registration) => registration.view().identity(),
        PayloadProjectionRegistration::Collection(registration) => registration.view().identity(),
    };
    plan.projection_input_slot(identity)
}

fn register_projection(
    builder: BoundBuilder,
    projection: PayloadProjectionRegistration,
) -> BoundBuilder {
    match projection {
        PayloadProjectionRegistration::None => builder,
        PayloadProjectionRegistration::Scalar(registration) => builder
            .register_scalar_projection(registration)
            .expect("scenario scalar projection registers"),
        PayloadProjectionRegistration::Collection(registration) => builder
            .register_collection_projection(registration)
            .expect("scenario collection projection registers"),
    }
}

fn register_facts(mut builder: BoundBuilder, facts: PayloadApplicationFacts) -> BoundBuilder {
    builder = builder
        .register_intent_boolean_fact(facts.operability.0, facts.operability.1)
        .expect("payload operability fact registers");
    if let Some((fact, initial)) = facts.text {
        builder = builder
            .register_intent_text_fact(fact, initial)
            .expect("scenario text fact registers");
    }
    if let Some((fact, initial)) = facts.boolean {
        builder = builder
            .register_intent_boolean_fact(fact, initial)
            .expect("scenario boolean fact registers");
    }
    if let Some((fact, initial)) = facts.unsigned64 {
        builder = builder
            .register_intent_unsigned64_fact(fact, initial)
            .expect("scenario unsigned fact registers");
    }
    builder
}

fn operability_fact() -> UiIntentApplicationFact<UiIntentBoolean> {
    UiIntentApplicationFact::boolean(OPERABILITY_FACT)
        .expect("payload operability fact identity is valid")
}

fn bind_operability<I: UiIntent>(
    declaration: UiIntentDeclaration<I>,
) -> worth_ui_dsl::WorthUiIntentDeclarationSpec {
    let fact = operability_fact();
    declaration
        .operability_from(
            UiIntentOperabilityContract::new(
                OPERABILITY_CONTRACT,
                UiIntentMutabilitySource::application_fact(&fact),
                UiIntentReadinessSource::application_fact(&fact),
                UiIntentPolicySource::application_fact(&fact),
            )
            .expect("payload operability contract identity is valid"),
        )
        .confirmation(
            UiIntentConfirmationContract::not_required(CONFIRMATION_POLICY)
                .expect("payload confirmation policy identity is valid"),
        )
        .concurrency(UiIntentConcurrencyScope::TargetRouteSingleFlight)
        .consequences(UiIntentConsequenceContract::none())
        .into_dsl_spec()
}
