//! Fresh admission and installed handler execution for captured label input.
use crate::domain_computation::primary_graph::tests::fixture::correction_source::*;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::*;
use worth_query_execution::facade::domain_computation::HandlerResult;

pub fn admitted(
    world: &AuthorizationWorld,
    phase: &WorthQueryAdvancementPhase<'_>,
    request: &WorthQueryRequestScope,
) -> (
    WorthQueryAdmittedApplicationOperation<IdentityExecutionSchema, EditLabel, LabelEdit, Account>,
    u64,
) {
    let custody = phase.test_host_request_for_test();
    let principal =
        crate::domain_computation::primary_graph::tests::application_attempt::authenticated_principal(&custody, world, request);
    let account =
        crate::domain_computation::primary_graph::tests::application_attempt::resolved_account(
            &custody, world, "open", request,
        );
    let operation = world
        .application
        .installed_schema()
        .installed_operation(EditLabel::reference())
        .unwrap();
    let admission = world
        .selected_product(&custody)
        .authorize_operation(
            phase,
            &principal,
            &account,
            &operation,
            TypedMutationPreconditions::new(),
            request,
        )
        .unwrap();
    (admission, *principal.principal_identity())
}

pub fn candidate(
    world: &AuthorizationWorld,
    phase: &WorthQueryAdvancementPhase<'_>,
    request: &WorthQueryRequestScope,
    input: &ApplicationEncodedInput<LabelInput>,
    key: &String,
) -> LabelProgram {
    let (admission, principal_identity) = admitted(world, phase, request);
    let identities =
        ApplicationMutationIdentities::<IdentityExecutionSchema, LabelBinding>::with_encoded_input(
            key, input,
        )
        .unwrap();
    let HandlerResult::Completed(candidate) = world
        .application
        .execute_mutation_handler::<LabelBinding>(
            phase,
            &identities,
            &principal_identity,
            admission,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
    else {
        panic!("the ordinary installed label handler completes");
    };
    let (program, _) = candidate.into_parts();
    program
        .retain_original_mutation_input::<LabelBinding>(
            &worth_query_execution::publication_boundary::program_publication_access(),
            phase,
            &world.application,
            input.clone(),
        )
        .unwrap()
}

pub fn derive(
    world: &AuthorizationWorld,
    source: &WorthQueryApplicationCommitReceipt,
    wire_key: &str,
) -> PreparedLabelCorrection {
    let request =
        crate::domain_computation::primary_graph::tests::fixture::correction_source::request_scope(
            world,
        );
    world.application.with_application_advancement(&request, |phase| {
        let principal = crate::domain_computation::primary_graph::tests::fixture::authentication::authenticate_external(
            world.application.installed_schema(), "alice", std::time::Duration::from_secs(120), &request,
        );
        let target = world.selected_product(&phase.test_host_request_for_test());
        let access = worth_query_execution::publication_boundary::program_publication_access();
        let selected = target.select_correction_source(
            &phase, &principal, &request, source, None, std::num::NonZeroUsize::MIN,
        ).unwrap();
        let command = target.correction_command_key::<crate::domain_computation::primary_graph::tests::fixture::correction_source::recorded::RecordedLabelEdit>(
            &access, &phase, &principal, &request, wire_key,
        ).unwrap();
        let (input, source) = target.derive_correction_inverse::<crate::domain_computation::primary_graph::tests::fixture::correction_source::recorded::RecordedLabelEdit>(
            &access, &phase, &principal, &request, &selected, &command, None, std::num::NonZeroUsize::MIN,
        ).unwrap();
        let key = <crate::domain_computation::primary_graph::tests::fixture::correction_source::recorded::RecordedLabelEdit as worth_query_declaration::facade::application_aftermath::ApplicationCorrectionBinding<IdentityExecutionSchema>>::inverse_key(wire_key).unwrap();
        let identities = ApplicationMutationIdentities::<IdentityExecutionSchema, LabelBinding>::with_encoded_input(&key, &input).unwrap();
        let binding = source.intent().bind_idempotency(WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities));
        let program = candidate(world, &phase, &request, &input, &key);
        PreparedLabelCorrection { program, input, key, binding, source }
    }).unwrap()
}

/// Retain the real label input and native source grant without a live candidate.
/// The actual receiver owns authentication, selection, disclosure and derivation.
pub fn retain_home_label_input(
    application: &worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<IdentityExecutionSchema>,
    coordinates: &worth_query_execution::facade::primary_graph::WorthQueryCorrectionSourceCoordinates,
    wire_key: &str,
    request: &WorthQueryRequestScope,
) -> (
    ApplicationEncodedInput<LabelInput>,
    std::sync::Arc<WorthQueryPreparedCorrectionSource>,
) {
    application.with_application_advancement(request, |phase| {
        let principal = crate::domain_computation::primary_graph::tests::fixture::authentication::authenticate_external(
            application.installed_schema(), "alice", std::time::Duration::from_secs(120), request,
        );
        let target = application.on_branch(application.current_world()).select(&phase).unwrap();
        let access = worth_query_execution::publication_boundary::program_publication_access();
        let source = target.select_correction_source_coordinates(&phase, &principal, request,
            coordinates, None, std::num::NonZeroUsize::new(8).unwrap()).unwrap();
        let key = target.correction_command_key::<super::recorded::RecordedLabelEdit>(
            &access, &phase, &principal, request, wire_key).unwrap();
        target.derive_correction_inverse::<super::recorded::RecordedLabelEdit>(
            &access, &phase, &principal, request, &source, &key, None,
            std::num::NonZeroUsize::new(8).unwrap()).unwrap()
    }).unwrap()
}
