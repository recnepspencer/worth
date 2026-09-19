const SELECTION_INTENT_ID: &str = "test.mounted_selection.selection";
const SELECTION_PROJECTION_ID: &str = super::PROJECTION;
const MUTABILITY_FACT: &str = "test.mounted_selection.selection.mutable";
const READINESS_FACT: &str = "test.mounted_selection.selection.ready";
const POLICY_FACT: &str = "test.mounted_selection.selection.policy";

struct MountedSelectionPayload {
    _selection: crate::capability::UiIntentSelectionValue,
}
struct MountedSelectionOutcome;
struct MountedSelectionIntent;
struct MountedSelectionProvider;

impl crate::capability::UiIntentPayload for MountedSelectionPayload {
    const SCHEMA: crate::capability::UiIntentSchema =
        crate::capability::UiIntentSchema::stable("test.mounted_selection.selection.payload", 1);
    const FIELDS: crate::capability::UiIntentPayloadFieldSet =
        crate::capability::UiIntentPayloadFieldSet::new(&[
            crate::capability::UiIntentPayloadField::<
                Self,
                crate::capability::UiIntentSelection,
            >::selection(0, "selection")
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

impl crate::capability::UiIntentProductOutcome for MountedSelectionOutcome {
    const SCHEMA: crate::capability::UiIntentSchema =
        crate::capability::UiIntentSchema::stable("test.mounted_selection.selection.outcome", 1);
    const CONSEQUENCE_FAMILIES: crate::capability::UiIntentProductConsequenceFamilies =
        crate::capability::UiIntentProductConsequenceFamilies::NONE;

    fn into_consequences(self) -> crate::capability::UiIntentProductConsequences {
        crate::capability::UiIntentProductConsequences::none()
    }
}

impl crate::capability::UiIntent for MountedSelectionIntent {
    type Payload = MountedSelectionPayload;
    type ProductOutcome = MountedSelectionOutcome;

    const ID: crate::capability::UiIntentId =
        crate::capability::UiIntentId::stable(SELECTION_INTENT_ID);
    const ACCEPTED_INTERACTIONS: crate::capability::UiIntentAcceptedInteractions =
        crate::capability::UiIntentAcceptedInteractions::new(&[
            crate::capability::UiSemanticInteractionFamily::SelectionCommit,
        ]);
}

impl crate::runtime::intent_execution::UiIntentExecutionProvider<MountedSelectionIntent>
    for MountedSelectionProvider
{
    const VERSION: crate::runtime::intent_execution::UiIntentProviderVersion =
        crate::runtime::intent_execution::UiIntentProviderVersion::stable(1);

    fn begin(
        &self,
        _request: crate::runtime::intent_execution::UiIntentExecutionRequest<
            MountedSelectionIntent,
        >,
    ) -> crate::runtime::intent_execution::UiIntentProviderStart<MountedSelectionIntent> {
        crate::runtime::intent_execution::UiIntentProviderStart::RejectedBeforeEffect(
            crate::runtime::intent_execution::UiIntentProviderStop::stable(
                "test.mounted_selection.selection.not_executed",
            ),
        )
    }
}

pub(super) fn register(
    builder: crate::facade::entry::WorthUiApplicationBuilder,
) -> crate::facade::entry::WorthUiApplicationBuilder {
    builder
        .register_intent_boolean_fact(
            crate::declaration::UiIntentApplicationFact::boolean(MUTABILITY_FACT).unwrap(),
            true,
        )
        .unwrap()
        .register_intent_boolean_fact(
            crate::declaration::UiIntentApplicationFact::boolean(READINESS_FACT).unwrap(),
            true,
        )
        .unwrap()
        .register_intent_boolean_fact(
            crate::declaration::UiIntentApplicationFact::boolean(POLICY_FACT).unwrap(),
            true,
        )
        .unwrap()
        .register_intent_definition(crate::capability::UiIntentDefinition::<
            MountedSelectionIntent,
        >::application_effect())
        .unwrap()
        .register_intent_provider(MountedSelectionProvider)
        .unwrap()
}
pub(super) fn declaration() -> worth_ui_dsl::WorthUiIntentDeclarationSpec {
    worth_ui_dsl::WorthUiIntentDeclarationSpec::new(
        "test.mounted_selection.selection_declaration",
        SELECTION_INTENT_ID,
        worth_ui_dsl::WorthUiIntentInteractionFamily::SelectionCommit,
        worth_ui_dsl::WorthUiIntentOperabilityContractSpec::new(
            "test.mounted_selection.selection.operability",
            worth_ui_dsl::WorthUiIntentMutabilitySourceSpec::application_boolean(MUTABILITY_FACT),
            worth_ui_dsl::WorthUiIntentReadinessSourceSpec::application_boolean(READINESS_FACT),
            worth_ui_dsl::WorthUiIntentPolicySourceSpec::application_boolean(POLICY_FACT),
        ),
        worth_ui_dsl::WorthUiIntentConfirmationContractSpec::not_required(
            "test.mounted_selection.selection.confirmation",
        ),
        worth_ui_dsl::WorthUiIntentConcurrencyScope::TargetRouteSingleFlight,
        worth_ui_dsl::WorthUiIntentConsequenceContractSpec::none(),
    )
    .with_payload_source(
        worth_ui_dsl::WorthUiIntentPayloadSourceSpec::projection_selection(
            "selection",
            SELECTION_PROJECTION_ID,
        ),
    )
}
