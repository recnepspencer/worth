#![allow(dead_code)] // This fixture is compiled by several independent certification targets.

pub use worth_query_host::facade::{primary_graph, product};

pub use super::super::adapters::{block_on, ReplacementPredicate};
pub use super::super::schema::*;
pub use super::{admit_identity_adapter, request_scope, CourtroomWorld};

impl CourtroomWorld {
    pub fn change_input_and_conditional_definition_on_branch(
        &self,
        branch: product::WorthQueryProductBranch,
        input: &str,
        provider: std::sync::Arc<ReplacementPredicate>,
        ordinal: u8,
    ) -> primary_graph::WorthQueryApplicationCommitReceipt {
        let selected = self
            .application
            .with_host_advancement(|phase| {
                self.application.on_branch(branch).select(&phase).unwrap()
            })
            .expect("installed host call opens");
        self.change_input_and_conditional_definition_on_selected(selected, input, provider, ordinal)
    }

    pub fn change_input_and_conditional_definition_on_selected(
        &self,
        selected: primary_graph::WorthQuerySelectedProductOperation<'_, TemporalHostSchema>,
        input: &str,
        provider: std::sync::Arc<ReplacementPredicate>,
        ordinal: u8,
    ) -> primary_graph::WorthQueryApplicationCommitReceipt {
        publish_on_selected(
            &self.application,
            &self.clock,
            &self.invariant,
            selected,
            input,
            provider,
            ordinal,
            true,
        )
    }
}

/// Executes the same admitted public operation against the borrowed serving owner.
/// A notice is an explicit fixture effect; authored-only journeys omit that effect.
pub fn publish_on_selected<Provider>(
    application: &primary_graph::WorthQueryPrimaryGraphApplicationRuntime<TemporalHostSchema>,
    clock: &primary_graph::WorthQueryConditionalClockHandle<
        TemporalHostSchema,
        super::super::contract::TemporalReadyNode,
        super::super::adapters::CourtroomClock,
    >,
    invariant: &std::sync::Arc<
        primary_graph::WorthQueryApplicationInvariantProjectionAuthority<TemporalHostSchema>,
    >,
    selected: primary_graph::WorthQuerySelectedProductOperation<'_, TemporalHostSchema>,
    input: &str,
    provider: std::sync::Arc<Provider>,
    ordinal: u8,
    emit_notice: bool,
) -> primary_graph::WorthQueryApplicationCommitReceipt
where
    Provider: worth_query_host::facade::domain::WorthQueryAuthoredHostConditionalPredicateProvider<
        super::super::contract::TemporalReadyNode,
    >,
{
    let schema = application.installed_schema();
    let principal_binding = schema
        .principal_binding(TemporalPrincipalBinding::reference())
        .unwrap();
    let authentication = admit_identity_adapter(schema);
    let request = request_scope();
    let external = block_on(authentication.authenticate((), &request)).unwrap();
    let branch = selected.product().product_branch();
    let conditional_definition = selected
        .admit_application_conditional_definition(clock, provider)
        .unwrap();
    let principal = application
        .with_host_advancement(|phase| {
            selected
                .resolve_authenticated_principal(
                    &phase,
                    &principal_binding,
                    &external,
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let intent = application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    IntentIdentityField::at("IntentIdentityField"),
                    "intent-1".to_string(),
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let operation = schema
        .installed_operation(AmendTemporalAndPublishDefinition::reference())
        .unwrap();
    let admission = application
        .with_host_advancement(|phase| {
            selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &intent,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let (_, projection, _) = application
        .with_host_advancement(|phase| {
            invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, scope| {
                        reader
                            .decision_field(scope, IntentRevisionField::at("IntentRevisionField"))
                            .unwrap();
                        reader
                            .decision_field(scope, IntentLifecycleField::at("IntentLifecycleField"))
                            .unwrap();
                        reader
                            .decision_field(scope, IntentGateField::at("IntentGateField"))
                            .unwrap();
                        reader
                            .decision_field(scope, IntentDueField::at("IntentDueField"))
                            .unwrap();
                        reader
                            .decision_field(scope, IntentInputField::at("IntentInputField"))
                            .unwrap();
                    },
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("installed host call opens");
    let mut effects = application
        .with_host_advancement(|phase| {
            let reads = application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            reads
                .complete_projected_dependencies(
                    &phase,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program()
        })
        .expect("installed host call opens");
    let intent = effects.existing_entity(&intent).unwrap();
    effects
        .write_field(&intent, IntentRevisionField::at("IntentRevisionField"), 2)
        .unwrap();
    effects
        .write_field(
            &intent,
            IntentLifecycleField::at("IntentLifecycleField"),
            "active".to_string(),
        )
        .unwrap();
    effects
        .write_field(&intent, IntentDueField::at("IntentDueField"), 11)
        .unwrap();
    effects
        .write_field(
            &intent,
            IntentInputField::at("IntentInputField"),
            input.to_string(),
        )
        .unwrap();
    effects
        .write_field(
            &intent,
            IntentGateField::at("IntentGateField"),
            "ready".to_string(),
        )
        .unwrap();
    effects
        .advance_conditional_definition(conditional_definition)
        .unwrap();
    if emit_notice {
        effects
            .emit_external(
                TemporalAmendmentEffect::reference(),
                TemporalAmendmentNotice(input.to_string()),
            )
            .unwrap();
    }
    let change = product::WorthQueryAdmittedChange::new(
        effects.finish().unwrap(),
        primary_graph::WorthQueryApplicationIdempotencyBinding::for_host_commit::<
            TemporalHostSchema,
            AmendTemporalAndPublishDefinition,
            _,
            _,
        >(
            &("combined-amendment", ordinal),
            &("combined-amendment", ordinal),
        )
        .unwrap(),
    );
    let outcome = application
        .on_branch(branch)
        .transaction()
        .apply(change)
        .commit();
    outcome
        .expect("the admitted combined change remains on its selected product")
        .require_committed()
        .unwrap_or_else(|uncommitted| match uncommitted {
            primary_graph::WorthQueryApplicationCommitOutcome::ProductUnpublished(original) => {
                panic!(
                    "the combined amendment must commit; retained original {:?}, refusal {:?}",
                    original.cause(),
                    original.before_effect_refusal()
                )
            }
            other => panic!("the combined amendment must commit: {other:?}"),
        })
}
