use crate::runtime::tests::appearance_component_session_test_support::six_axis_appearance_component_builder;
use crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission;

const SELECTION_INTENT_ID: &str = "test.six_axis.selection";
const SELECTION_PROJECTION_ID: &str = "test.six_axis.selection_projection";
const MUTABILITY_FACT: &str = "test.six_axis.selection.mutable";
const READINESS_FACT: &str = "test.six_axis.selection.ready";
const POLICY_FACT: &str = "test.six_axis.selection.policy";

struct SixAxisSelectionPayload {
    _selection: crate::capability::UiIntentSelectionValue,
}
struct SixAxisSelectionOutcome;
struct SixAxisSelectionIntent;
struct SixAxisFocusServiceIntent;
struct SixAxisSelectionProvider;

impl crate::capability::UiIntentPayload for SixAxisSelectionPayload {
    const SCHEMA: crate::capability::UiIntentSchema =
        crate::capability::UiIntentSchema::stable("test.six_axis.selection.payload", 1);
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

impl crate::capability::UiIntentProductOutcome for SixAxisSelectionOutcome {
    const SCHEMA: crate::capability::UiIntentSchema =
        crate::capability::UiIntentSchema::stable("test.six_axis.selection.outcome", 1);
    const CONSEQUENCE_FAMILIES: crate::capability::UiIntentProductConsequenceFamilies =
        crate::capability::UiIntentProductConsequenceFamilies::NONE;

    fn into_consequences(self) -> crate::capability::UiIntentProductConsequences {
        crate::capability::UiIntentProductConsequences::none()
    }
}

impl crate::capability::UiIntent for SixAxisSelectionIntent {
    type Payload = SixAxisSelectionPayload;
    type ProductOutcome = SixAxisSelectionOutcome;

    const ID: crate::capability::UiIntentId =
        crate::capability::UiIntentId::stable(SELECTION_INTENT_ID);
    const ACCEPTED_INTERACTIONS: crate::capability::UiIntentAcceptedInteractions =
        crate::capability::UiIntentAcceptedInteractions::new(&[
            crate::capability::UiSemanticInteractionFamily::SelectionCommit,
        ]);
}

impl crate::capability::UiIntent for SixAxisFocusServiceIntent {
    type Payload = SixAxisSelectionPayload;
    type ProductOutcome = SixAxisSelectionOutcome;

    const ID: crate::capability::UiIntentId =
        crate::capability::UiIntentId::stable("test.six_axis.focus_service");
    const ACCEPTED_INTERACTIONS: crate::capability::UiIntentAcceptedInteractions =
        crate::capability::UiIntentAcceptedInteractions::new(&[
            crate::capability::UiSemanticInteractionFamily::Activate,
        ]);
}

impl crate::runtime::intent_execution::UiIntentExecutionProvider<SixAxisSelectionIntent>
    for SixAxisSelectionProvider
{
    const VERSION: crate::runtime::intent_execution::UiIntentProviderVersion =
        crate::runtime::intent_execution::UiIntentProviderVersion::stable(1);

    fn begin(
        &self,
        _request: crate::runtime::intent_execution::UiIntentExecutionRequest<
            SixAxisSelectionIntent,
        >,
    ) -> crate::runtime::intent_execution::UiIntentProviderStart<SixAxisSelectionIntent> {
        crate::runtime::intent_execution::UiIntentProviderStart::RejectedBeforeEffect(
            crate::runtime::intent_execution::UiIntentProviderStop::stable(
                "test.six_axis.selection.not_executed",
            ),
        )
    }
}

pub(super) fn six_axis_builder(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> crate::facade::entry::WorthUiApplicationBuilder {
    let query_domain =
        worth_ui_query_binding::certification::worth_ui_installed_test_domain("six-axis-selection");
    let selection_projection =
        crate::facade::query_binding::UiCollectionProjectionRegistration::text(
            query_domain
                .projection_view(SELECTION_PROJECTION_ID)
                .expect("selection projection view should admit"),
            crate::facade::query_binding::UiProjectionFieldRequirement::identity_id(),
            [crate::facade::query_binding::UiProjectionFieldRequirement::query_text_status()],
            false,
            false,
        )
        .expect("selection projection registration should admit");
    six_axis_appearance_component_builder(role)
        .register_appearance_theme_bundle(
            crate::runtime::tests::appearance_theme_test_support::bundle(),
        )
        .expect("six-axis appearance theme should register")
        .register_collection_projection(selection_projection)
        .expect("selection projection should register")
        .register_intent_boolean_fact(
            crate::declaration::UiIntentApplicationFact::boolean(MUTABILITY_FACT).unwrap(),
            true,
        )
        .expect("selection mutability fact should register")
        .register_intent_boolean_fact(
            crate::declaration::UiIntentApplicationFact::boolean(READINESS_FACT).unwrap(),
            true,
        )
        .expect("selection readiness fact should register")
        .register_intent_boolean_fact(
            crate::declaration::UiIntentApplicationFact::boolean(POLICY_FACT).unwrap(),
            true,
        )
        .expect("selection policy fact should register")
        .register_intent_definition(crate::capability::UiIntentDefinition::<
            SixAxisSelectionIntent,
        >::application_effect())
        .expect("selection intent definition should register")
        .register_intent_provider(SixAxisSelectionProvider)
        .expect("selection intent provider should register")
        .register_runtime_service_intent_definition(crate::capability::UiIntentDefinition::<
            SixAxisFocusServiceIntent,
        >::runtime_service(
            crate::capability::UiIntentRuntimeServiceDestination::OpenPortal,
        ))
        .expect("focus service definition should register")
}

pub(super) fn candidate_submission(
    source_name: &str,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    capabilities: &crate::capability::CapabilitySnapshot,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    let component = worth_ui_dsl::WorthUiSemanticArtifactDeclaration::new(
        worth_ui_dsl::UiDslSemanticKey::new(
            crate::runtime::tests::appearance_component_session_test_support::APPEARANCE_NODE_A,
        ),
        worth_ui_dsl::UiDslSemanticFamily::Control,
    )
    .with_structural_token(worth_ui_dsl::UiDslStructuralToken::new(
        "control:appearance-consumer",
    ))
    .with_component_reference(
        worth_ui_dsl::UiDslComponentReference::new(
            crate::runtime::tests::appearance_component_session_test_support::APPEARANCE_NODE_A,
        )
        .unwrap(),
    )
    .unwrap()
    .with_appearance_role_attachment(worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
        role.role().clone(),
        role.revision(),
    ))
    .unwrap();
    let input = worth_ui_dsl::WorthUiRustAuthoredArtifactInput::from_modules([
        worth_ui_dsl::WorthUiRustAuthoredArtifactInputModule::new("appearance/consumer")
            .with_semantic_declaration(component)
            .with_intent_declaration(selection_intent_declaration()),
    ]);
    lower_rust_submission(
        crate::runtime::WorthUiSourceProvider::rust_authored(source_name)
            .with_rust_authored_input(input),
        [crate::runtime::WorthUiWatcherEvent::provider_revision(
            source_name,
        )],
        capabilities,
    )
}

fn selection_intent_declaration() -> worth_ui_dsl::WorthUiIntentDeclarationSpec {
    worth_ui_dsl::WorthUiIntentDeclarationSpec::new(
        "test.six_axis.selection_declaration",
        SELECTION_INTENT_ID,
        worth_ui_dsl::WorthUiIntentInteractionFamily::SelectionCommit,
        worth_ui_dsl::WorthUiIntentOperabilityContractSpec::new(
            "test.six_axis.selection.operability",
            worth_ui_dsl::WorthUiIntentMutabilitySourceSpec::application_boolean(MUTABILITY_FACT),
            worth_ui_dsl::WorthUiIntentReadinessSourceSpec::application_boolean(READINESS_FACT),
            worth_ui_dsl::WorthUiIntentPolicySourceSpec::application_boolean(POLICY_FACT),
        ),
        worth_ui_dsl::WorthUiIntentConfirmationContractSpec::not_required(
            "test.six_axis.selection.confirmation",
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
