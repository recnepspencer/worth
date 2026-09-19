const INTENT: &str = "integrated.ap07.selection";
const MUTABLE: &str = "integrated.ap07.selection.mutable";
const READY: &str = "integrated.ap07.selection.ready";
const POLICY: &str = "integrated.ap07.selection.policy";

struct Payload {
    _selection: crate::capability::UiIntentSelectionValue,
}
struct Outcome;
struct Intent;
struct Provider;
struct ActivationPayload;
struct ActivationOutcome;
struct ActivationIntent;
struct ActivationProvider;

impl crate::capability::UiIntentPayload for Payload {
    const SCHEMA: crate::capability::UiIntentSchema =
        crate::capability::UiIntentSchema::stable("integrated.ap07.selection.payload", 1);
    const FIELDS: crate::capability::UiIntentPayloadFieldSet =
        crate::capability::UiIntentPayloadFieldSet::new(&[
            crate::capability::UiIntentPayloadField::<Self, crate::capability::UiIntentSelection>::selection(
                0,
                "selection",
            )
            .descriptor(),
        ]);

    fn project(
        fields: &mut crate::capability::UiIntentPayloadProjection<Self>,
    ) -> Result<Self, crate::capability::UiIntentPayloadProjectionViolation> {
        Ok(Self {
            _selection: fields.take(crate::capability::UiIntentPayloadField::<
                Self,
                crate::capability::UiIntentSelection,
            >::selection(0, "selection"))?,
        })
    }
}

impl crate::capability::UiIntentProductOutcome for Outcome {
    const SCHEMA: crate::capability::UiIntentSchema =
        crate::capability::UiIntentSchema::stable("integrated.ap07.selection.outcome", 1);
    const CONSEQUENCE_FAMILIES: crate::capability::UiIntentProductConsequenceFamilies =
        crate::capability::UiIntentProductConsequenceFamilies::NONE;

    fn into_consequences(self) -> crate::capability::UiIntentProductConsequences {
        crate::capability::UiIntentProductConsequences::none()
    }
}

impl crate::capability::UiIntent for Intent {
    type Payload = Payload;
    type ProductOutcome = Outcome;

    const ID: crate::capability::UiIntentId = crate::capability::UiIntentId::stable(INTENT);
    const ACCEPTED_INTERACTIONS: crate::capability::UiIntentAcceptedInteractions =
        crate::capability::UiIntentAcceptedInteractions::new(&[
            crate::capability::UiSemanticInteractionFamily::SelectionCommit,
        ]);
}

impl crate::capability::UiIntentPayload for ActivationPayload {
    const SCHEMA: crate::capability::UiIntentSchema =
        crate::capability::UiIntentSchema::stable("integrated.ap07.activate.payload", 1);
    const FIELDS: crate::capability::UiIntentPayloadFieldSet =
        crate::capability::UiIntentPayloadFieldSet::EMPTY;

    fn project(
        _: &mut crate::capability::UiIntentPayloadProjection<Self>,
    ) -> Result<Self, crate::capability::UiIntentPayloadProjectionViolation> {
        Ok(Self)
    }
}

impl crate::capability::UiIntentProductOutcome for ActivationOutcome {
    const SCHEMA: crate::capability::UiIntentSchema =
        crate::capability::UiIntentSchema::stable("integrated.ap07.activate.outcome", 1);
    const CONSEQUENCE_FAMILIES: crate::capability::UiIntentProductConsequenceFamilies =
        crate::capability::UiIntentProductConsequenceFamilies::NONE;

    fn into_consequences(self) -> crate::capability::UiIntentProductConsequences {
        crate::capability::UiIntentProductConsequences::none()
    }
}

impl crate::capability::UiIntent for ActivationIntent {
    type Payload = ActivationPayload;
    type ProductOutcome = ActivationOutcome;

    const ID: crate::capability::UiIntentId =
        crate::capability::UiIntentId::stable("integrated.ap07.activate");
    const ACCEPTED_INTERACTIONS: crate::capability::UiIntentAcceptedInteractions =
        crate::capability::UiIntentAcceptedInteractions::new(&[
            crate::capability::UiSemanticInteractionFamily::Activate,
        ]);
}

impl crate::runtime::intent_execution::UiIntentExecutionProvider<ActivationIntent>
    for ActivationProvider
{
    const VERSION: crate::runtime::intent_execution::UiIntentProviderVersion =
        crate::runtime::intent_execution::UiIntentProviderVersion::stable(1);

    fn begin(
        &self,
        _: crate::runtime::intent_execution::UiIntentExecutionRequest<ActivationIntent>,
    ) -> crate::runtime::intent_execution::UiIntentProviderStart<ActivationIntent> {
        panic!("AP-07 evaluates operability without executing the activation provider")
    }
}

impl crate::runtime::intent_execution::UiIntentExecutionProvider<Intent> for Provider {
    const VERSION: crate::runtime::intent_execution::UiIntentProviderVersion =
        crate::runtime::intent_execution::UiIntentProviderVersion::stable(1);

    fn begin(
        &self,
        _: crate::runtime::intent_execution::UiIntentExecutionRequest<Intent>,
    ) -> crate::runtime::intent_execution::UiIntentProviderStart<Intent> {
        crate::runtime::intent_execution::UiIntentProviderStart::RejectedBeforeEffect(
            crate::runtime::intent_execution::UiIntentProviderStop::stable(
                "integrated.ap07.selection.not_executed",
            ),
        )
    }
}

pub(super) fn register(
    builder: crate::facade::entry::WorthUiCertificationApplicationBuilder,
) -> crate::facade::entry::WorthUiCertificationApplicationBuilder {
    builder
        .register_intent_boolean_fact(
            crate::declaration::UiIntentApplicationFact::boolean(MUTABLE).unwrap(),
            true,
        )
        .unwrap()
        .register_intent_boolean_fact(
            crate::declaration::UiIntentApplicationFact::boolean(READY).unwrap(),
            true,
        )
        .unwrap()
        .register_intent_boolean_fact(
            crate::declaration::UiIntentApplicationFact::boolean(POLICY).unwrap(),
            true,
        )
        .unwrap()
        .register_intent_definition(
            crate::capability::UiIntentDefinition::<Intent>::application_effect(),
        )
        .unwrap()
        .register_intent_provider(Provider)
        .unwrap()
        .register_intent_definition(
            crate::capability::UiIntentDefinition::<ActivationIntent>::application_effect(),
        )
        .unwrap()
        .register_intent_provider(ActivationProvider)
        .unwrap()
}
