//! Fresh admission and installed handler execution for captured label input.
use super::{label_edit::*, *};
use crate::domain_computation::primary_graph::application_entry::mutation::HandlerResult;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::*;

pub(super) fn admitted(
    world: &AuthorizationWorld,
    phase: &WorthQueryAdvancementPhase<'_>,
    request: &WorthQueryRequestScope,
) -> (
    WorthQueryAdmittedApplicationOperation<IdentityExecutionSchema, EditLabel, LabelEdit, Account>,
    u64,
) {
    let custody = phase.test_host_request();
    let principal =
        super::super::super::application_attempt::authenticated_principal(&custody, world, request);
    let account = super::super::super::application_attempt::resolved_account(
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

pub(super) fn candidate(
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
            &crate::publication_boundary::program_publication_access(),
            &world.application,
            input.clone(),
        )
        .unwrap()
}

pub(super) fn derive(
    world: &AuthorizationWorld,
    source: &WorthQueryApplicationCommitReceipt,
    wire_key: &str,
) -> PreparedLabelCorrection {
    let request = super::super::live_scope();
    world.application.with_application_advancement(&request, |phase| {
        let principal = super::super::authenticate_external(
            world.application.installed_schema(), "alice", std::time::Duration::from_secs(120), &request,
        );
        let target = world.selected_product(&phase.test_host_request());
        let access = crate::publication_boundary::program_publication_access();
        let selected = target.select_correction_source(
            &phase, &principal, &request, source, None, std::num::NonZeroUsize::MIN,
        ).unwrap();
        let command = target.correction_command_key::<recorded::RecordedLabelEdit>(
            &access, &phase, &principal, &request, wire_key,
        ).unwrap();
        let (input, source) = target.derive_correction_inverse::<recorded::RecordedLabelEdit>(
            &access, &phase, &principal, &request, &selected, &command, None, std::num::NonZeroUsize::MIN,
        ).unwrap();
        let key = <recorded::RecordedLabelEdit as worth_query_declaration::facade::application_aftermath::ApplicationCorrectionBinding<IdentityExecutionSchema>>::inverse_key(wire_key).unwrap();
        let identities = ApplicationMutationIdentities::<IdentityExecutionSchema, LabelBinding>::with_encoded_input(&key, &input).unwrap();
        let binding = source.intent().bind_idempotency(WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities));
        let program = candidate(world, &phase, &request, &input, &key);
        PreparedLabelCorrection { program, input, key, binding, source }
    }).unwrap()
}
