//! A consumer whose payload reads a `derived text`, a condition and a
//! `derived integer` over application facts no operability axis reads, so
//! every payload expression is payload-only.

use std::cell::RefCell;
use std::sync::Arc;

use crate::facade::intent::*;
use crate::runtime::tests::appearance_component_session_test_support::APPEARANCE_NODE_A;
use worth_ui_dsl::*;

pub(super) const LABEL: &str = "test.expression_payload.label";
pub(super) const CAPTION: &str = "test.expression_payload.caption";
pub(super) const ENABLED: &str = "test.expression_payload.enabled";
pub(super) const NAME: &str = "test.expression_payload.name";
pub(super) const SWITCH: &str = "test.expression_payload.switch";
pub(super) const COUNT: &str = "test.expression_payload.count";
pub(super) const TALLY: &str = "test.expression_payload.tally";
pub(super) const LABEL_BUDGET: usize = 8;
const INTENT: &str = "test.expression_payload.intent";
/// Reads `name`, mapping `gamma` to the value `alpha` already yields.
const LABEL_BODY: &str = r#"n == "gamma" ? "alpha" : n"#;
/// Two below the tally, so a tally under two reads as a negative count.
const COUNT_BODY: &str = "exact_cast<Int64>(t) - 2";

const LABEL_FIELD: UiIntentPayloadField<Payload, UiIntentText> =
    UiIntentPayloadField::text(0, "label", LABEL_BUDGET);
const ENABLED_FIELD: UiIntentPayloadField<Payload, UiIntentBoolean> =
    UiIntentPayloadField::boolean(1, "enabled");
const COUNT_FIELD: UiIntentPayloadField<Payload, UiIntentUnsigned64> =
    UiIntentPayloadField::unsigned64(2, "count");

/// The label, enabled and count values one payload projected.
pub(super) type Projected = (Arc<str>, bool, u64);

thread_local! {
    static PROJECTED: RefCell<Option<Projected>> = const { RefCell::new(None) };
}

/// The values the last payload projected on this thread.
pub(super) fn projected() -> Option<Projected> {
    PROJECTED.with(|projected| projected.borrow().clone())
}

pub(super) struct Payload;
pub(super) struct Outcome;
pub(super) struct PayloadIntent;
struct Provider;

impl UiIntentPayload for Payload {
    const SCHEMA: UiIntentSchema = UiIntentSchema::stable("test.expression_payload.payload", 1);
    const FIELDS: UiIntentPayloadFieldSet = UiIntentPayloadFieldSet::new(&[
        LABEL_FIELD.descriptor(),
        ENABLED_FIELD.descriptor(),
        COUNT_FIELD.descriptor(),
    ]);
    fn project(
        fields: &mut UiIntentPayloadProjection<Self>,
    ) -> Result<Self, UiIntentPayloadProjectionViolation> {
        let values = (
            fields.take(LABEL_FIELD)?,
            fields.take(ENABLED_FIELD)?,
            fields.take(COUNT_FIELD)?,
        );
        PROJECTED.with(|projected| *projected.borrow_mut() = Some(values));
        Ok(Self)
    }
}
impl UiIntentProductOutcome for Outcome {
    const SCHEMA: UiIntentSchema = UiIntentSchema::stable("test.expression_payload.outcome", 1);
    const CONSEQUENCE_FAMILIES: UiIntentProductConsequenceFamilies =
        UiIntentProductConsequenceFamilies::NONE;
    fn into_consequences(self) -> UiIntentProductConsequences {
        UiIntentProductConsequences::none()
    }
}
impl UiIntent for PayloadIntent {
    type Payload = Payload;
    type ProductOutcome = Outcome;
    const ID: UiIntentId = UiIntentId::stable(INTENT);
    const ACCEPTED_INTERACTIONS: UiIntentAcceptedInteractions =
        UiIntentAcceptedInteractions::new(&[UiSemanticInteractionFamily::Activate]);
}
impl UiIntentExecutionProvider<PayloadIntent> for Provider {
    const VERSION: UiIntentProviderVersion = UiIntentProviderVersion::stable(1);
    fn begin(
        &self,
        _: UiIntentExecutionRequest<PayloadIntent>,
    ) -> UiIntentProviderStart<PayloadIntent> {
        panic!("a payload fixture never executes its provider")
    }
}

/// Where the label field reads from.
#[derive(Clone, Copy)]
pub(super) enum Label {
    Constant,
    Derived(&'static str),
    /// A condition: the package admits the role, the text field refuses it.
    Condition,
}

fn label_source(label: Label) -> WorthUiIntentPayloadSourceSpec {
    match label {
        Label::Constant => WorthUiIntentPayloadSourceSpec::constant_text("label", "alpha"),
        Label::Derived(identity) => WorthUiIntentPayloadSourceSpec::derived("label", identity),
        Label::Condition => WorthUiIntentPayloadSourceSpec::condition("label", ENABLED),
    }
}

/// The consumer module: the payload declaration on the fixture's control,
/// and the expressions its fields read.
pub(super) fn payload_module(label: Label) -> WorthUiRustAuthoredArtifactInputModule {
    let role = super::fixture::role();
    let module = WorthUiRustAuthoredArtifactInputModule::new("appearance/consumer")
        .with_intent_declaration(payload_declaration(label))
        .with_control_routes(
            APPEARANCE_NODE_A,
            [WorthUiIntentInteractionRoute::product(
                WorthUiIntentInteractionFamily::Activate,
                super::fixture::ROUTE,
            )],
        )
        .with_appearance_role(role.clone())
        .with_component_appearance_role(
            APPEARANCE_NODE_A,
            UiAppearanceRoleAttachmentDeclaration::new(role.role().clone(), role.revision()),
        )
        .unwrap();
    with_payload_expressions(module)
}

/// Operability over the fixture facts, and a payload whose label reads
/// `label`, whose `enabled` field reads a condition and whose `count` field
/// reads a derived integer.
fn payload_declaration(label: Label) -> WorthUiIntentDeclarationSpec {
    let operability = WorthUiIntentOperabilityContractSpec::new(
        "test.expression_payload.operability",
        WorthUiIntentMutabilitySourceSpec::application_boolean(super::fixture::MUTABLE),
        WorthUiIntentReadinessSourceSpec::application_boolean(super::fixture::READY),
        WorthUiIntentPolicySourceSpec::application_boolean(super::fixture::POLICY),
    );
    WorthUiIntentDeclarationSpec::new(
        super::fixture::ROUTE,
        INTENT,
        WorthUiIntentInteractionFamily::Activate,
        operability,
        WorthUiIntentConfirmationContractSpec::not_required("test.expression_payload.confirmation"),
        WorthUiIntentConcurrencyScope::TargetRouteSingleFlight,
        WorthUiIntentConsequenceContractSpec::none(),
    )
    .with_payload_source(label_source(label))
    .with_payload_source(WorthUiIntentPayloadSourceSpec::condition(
        "enabled", ENABLED,
    ))
    .with_payload_source(WorthUiIntentPayloadSourceSpec::derived("count", COUNT))
}

fn with_payload_expressions(
    module: WorthUiRustAuthoredArtifactInputModule,
) -> WorthUiRustAuthoredArtifactInputModule {
    let name = || {
        [WorthUiExpressionOperand::new(
            "n",
            WorthUiExpressionOperandSource::ApplicationText {
                fact: NAME.to_owned(),
            },
        )]
    };
    module
        .try_with_derived(LABEL, name(), WorthUiExpressionResultType::Text, LABEL_BODY)
        .unwrap()
        .try_with_derived(CAPTION, name(), WorthUiExpressionResultType::Text, "n")
        .unwrap()
        .try_with_condition(
            ENABLED,
            [WorthUiExpressionOperand::new(
                "s",
                WorthUiExpressionOperandSource::ApplicationBoolean {
                    fact: SWITCH.to_owned(),
                },
            )],
            "s",
        )
        .unwrap()
        .try_with_derived(
            COUNT,
            [WorthUiExpressionOperand::new(
                "t",
                WorthUiExpressionOperandSource::ApplicationUnsigned64 {
                    fact: TALLY.to_owned(),
                },
            )],
            WorthUiExpressionResultType::Integer,
            COUNT_BODY,
        )
        .unwrap()
}

pub(super) fn payload_input(label: Label) -> WorthUiRustAuthoredArtifactInput {
    WorthUiRustAuthoredArtifactInput::from_modules([payload_module(label)])
}

pub(super) fn name_fact() -> UiIntentApplicationFact<UiIntentText> {
    UiIntentApplicationFact::text(NAME, 16).unwrap()
}

pub(super) fn switch_fact() -> UiIntentApplicationFact<UiIntentBoolean> {
    UiIntentApplicationFact::boolean(SWITCH).unwrap()
}

pub(super) fn tally_fact() -> UiIntentApplicationFact<UiIntentUnsigned64> {
    UiIntentApplicationFact::unsigned64(TALLY).unwrap()
}

/// A launched session over `label`, with the payload facts at `alpha`,
/// `true` and a tally of 3.
pub(super) fn payload_session(
    label: Label,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    super::fixture::session_with_registrations(
        &super::fixture::role(),
        payload_input(label),
        super::fixture::component(),
        |builder| {
            builder
                .register_intent_text_fact(name_fact(), "alpha")
                .unwrap()
                .register_intent_boolean_fact(switch_fact(), true)
                .unwrap()
                .register_intent_unsigned64_fact(tally_fact(), 3)
                .unwrap()
                .register_intent_definition(
                    UiIntentDefinition::<PayloadIntent>::application_effect(),
                )
                .unwrap()
                .register_intent_provider(Provider)
                .unwrap()
        },
    )
}

pub(super) fn payload_submission(
    session: &crate::facade::WorthUiActiveApplicationSession,
    label: Label,
    name: &str,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission(
        crate::runtime::WorthUiSourceProvider::rust_authored(name)
            .with_rust_authored_input(payload_input(label)),
        [crate::runtime::WorthUiWatcherEvent::provider_revision(name)],
        session.capabilities(),
    )
}

pub(super) struct PayloadWorld {
    pub(super) session: crate::facade::WorthUiActiveApplicationSession,
    pub(super) host: crate::certification_support::ScriptedPresentationHost,
    pub(super) surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
}

/// A mounted, published consumer whose payload reads `label`.
pub(super) fn payload_world(label: Label) -> PayloadWorld {
    let (mut session, host) = payload_session(label);
    let (surface, _) = super::super::super::super::mounting_fixture::mount(&mut session, 1_000);
    let source = payload_submission(&session, label, "expression-payload-initial");
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(source).unwrap();
    let admitted = turn.seal().unwrap();
    session.classify_observations(admitted).unwrap();
    session.advance_mounted_identity_frame().unwrap();
    let frame = super::prepare(&mut session);
    super::publish(&mut session, &host, frame, 1);
    PayloadWorld {
        session,
        host,
        surface,
    }
}
