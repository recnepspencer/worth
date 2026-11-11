//! A Home with the temporal installation program advances an authored definition
//! through its program-owned `AmendTemporal` lane, without any external effect.
#![allow(dead_code)] // This fixture is compiled by several independent certification targets.

use super::combined_amendment::*;
use worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime;

/// The installed temporal program owns `AmendTemporal`, so the change commits
/// through its admitted program lane. Dispatch custody of a notice belongs to H8.
pub fn publish_definition_on_program_lane<Provider>(
    program: &WorthQueryProgramApplicationRuntime<
        TemporalHostSchema,
        crate::facade::host::temporal_conditional_operation::contribution_installation::TemporalInstallationProgram,
    >,
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
) -> primary_graph::WorthQueryApplicationCommitReceipt
where
    Provider: worth_query_host::facade::domain::WorthQueryAuthoredHostConditionalPredicateProvider<
        super::super::contract::TemporalReadyNode,
    >,
{
    let application: &primary_graph::WorthQueryPrimaryGraphApplicationRuntime<TemporalHostSchema> =
        program;
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
                    IntentIdentityField::reference(),
                    "intent-1".to_string(),
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let operation = schema
        .installed_operation(AmendTemporal::reference())
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
                            .decision_field(scope, IntentRevisionField::reference())
                            .unwrap();
                        reader
                            .decision_field(scope, IntentLifecycleField::reference())
                            .unwrap();
                        reader
                            .decision_field(scope, IntentGateField::reference())
                            .unwrap();
                        reader
                            .decision_field(scope, IntentDueField::reference())
                            .unwrap();
                        reader
                            .decision_field(scope, IntentInputField::reference())
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
        .write_field(&intent, IntentRevisionField::reference(), 2)
        .unwrap();
    effects
        .write_field(
            &intent,
            IntentLifecycleField::reference(),
            "active".to_string(),
        )
        .unwrap();
    effects
        .write_field(&intent, IntentDueField::reference(), 11)
        .unwrap();
    effects
        .write_field(&intent, IntentInputField::reference(), input.to_string())
        .unwrap();
    effects
        .write_field(&intent, IntentGateField::reference(), "ready".to_string())
        .unwrap();
    effects
        .advance_conditional_definition(conditional_definition)
        .unwrap();
    let change = product::WorthQueryAdmittedChange::new(
        effects.finish().unwrap(),
        primary_graph::WorthQueryApplicationIdempotencyBinding::for_host_commit::<
            TemporalHostSchema,
            AmendTemporal,
            _,
            _,
        >(
            &("program-definition-amendment", ordinal),
            &("program-definition-amendment", ordinal),
        )
        .unwrap(),
    );
    let funding = crate::facade::host::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&funding);
    let outcome = application
        .on_branch(branch)
        .transaction()
        .apply(change)
        .commit_for_program(
            &execution,
            program.admit_program_operation::<AmendTemporal>().unwrap(),
        );
    outcome
        .expect("the admitted combined change remains on its selected product")
        .require_committed()
        .unwrap_or_else(|uncommitted| match uncommitted {
            primary_graph::WorthQueryApplicationCommitOutcome::ProductUnpublished(original) => {
                panic!(
                    "the program-lane definition amendment must commit; retained original {:?}, refusal {:?}",
                    original.cause(),
                    original.before_effect_refusal()
                )
            }
            other => panic!("the program-lane definition amendment must commit: {other:?}"),
        })
}
