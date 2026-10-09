//! Prepare an authored correction through public principal, scope and operation admission.
use super::*;
use worth_query_decl::facade::application_operation::{
    ApplicationMutationIdentities, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_decl::facade::application_schema::TypedMutationPreconditions;
use worth_query_host::facade::primary_graph::{
    HandlerResult, WorthQueryApplicationEffectProgram, WorthQueryPrincipalResolutionMode,
};

pub(super) type Identities<'a, const WORKFLOW: bool = false> = ApplicationMutationIdentities<
    'a,
    CheckpointSchema,
    EntryCorrectionBinding<CheckpointSchema, WORKFLOW>,
>;
pub(super) type Program =
    WorthQueryApplicationEffectProgram<CheckpointSchema, CorrectEntryValue, EntryCorrection, Body>;
pub(super) fn prepare<const REUSE: bool, const MODE: u8, const WORKFLOW: bool>(
    application: &Application<REUSE, TOTALS_WORK, 1, MODE>,
    principal: &authentication::WorthQueryAuthenticatedExternalPrincipal<CheckpointSchema>,
    scope: &authentication::WorthQueryRequestScope,
    identities: &Identities<'_, WORKFLOW>,
    recovery: Option<&primary_graph::WorthQueryRecoveryHandle>,
) -> (
    Program,
    Option<primary_graph::WorthQueryRecoveryEffectAuthority>,
) {
    let binding = application
        .installed_schema()
        .installed_mutation_binding::<EntryCorrectionBinding<CheckpointSchema, WORKFLOW>>()
        .unwrap();
    let selected = application
        .runtime()
        .on_branch(application.current_world())
        .select()
        .unwrap();
    let resolved = selected
        .resolve_authenticated_principal(
            binding.principal_binding(),
            principal,
            scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let (field, value) = identities
        .mutation_input()
        .scope_binding()
        .into_field_parts(resolved.principal_identity());
    let target = selected
        .resolve_entity(
            field,
            value,
            scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let _target = target;
    let capability =
        application
            .installed_schema()
            .capability(
                super::super::super::super::entry_correction::CorrectEntryCapability::<
                    CheckpointSchema,
                >::reference(),
                CorrectEntryValue::reference(),
            )
            .unwrap();
    let access = selected
        .admit_capability_access(
            &resolved,
            &capability,
            identities.mutation_input().clone(),
            scope,
        )
        .unwrap();
    let admission = application
        .runtime()
        .authorize_capability_operation(
            access,
            binding.operation(),
            TypedMutationPreconditions::new(),
        )
        .unwrap();
    let authority = recovery.map(|handle| {
        application
            .runtime()
            .admit_recovery_effect_authority(handle, &admission)
            .unwrap()
    });
    let HandlerResult::Completed(candidate) = application
        .runtime()
        .execute_mutation_handler(identities, resolved.principal_identity(), admission)
        .unwrap()
    else {
        panic!("the correction handler completes");
    };
    (candidate.into_parts().0, authority)
}
pub(super) fn committed(
    outcome: primary_graph::WorthQueryApplicationCommitOutcome,
) -> primary_graph::WorthQueryApplicationCommitReceipt {
    match outcome {
        primary_graph::WorthQueryApplicationCommitOutcome::Committed(receipt) => receipt,
        other => panic!("the governed correction commits: {other:?}"),
    }
}
