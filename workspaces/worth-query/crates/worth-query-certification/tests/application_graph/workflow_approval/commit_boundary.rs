//! The public program-owner commit door cannot bypass workflow authority.

use super::*;
use worth_query_decl::facade::application_operation::ApplicationMutationIdentities;
use worth_query_host::facade::{
    application_installation::WorthQueryProgramOwner,
    primary_graph::{HandlerResult, WorthQueryPrincipalResolutionMode},
};

use super::super::document_retention_model::retention_entry::SetRetentionBinding;
use super::super::document_retention_model::schema::{
    DocumentIdentityField, DocumentRetentionSchema, SetRetention, SetRetentionInput,
};

fn reviewed_identities<'a>(
    key: &'a u64,
    input: &'a SetRetentionInput,
) -> ApplicationMutationIdentities<'a, DocumentRetentionSchema, ReviewedSetRetentionBinding> {
    ApplicationMutationIdentities::encode(key, input).expect("key and input must encode")
}

#[path = "commit_boundary/guarded_program.rs"]
mod guarded_program;

#[test]
fn one_approval_transition_cannot_commit_twice_under_different_client_keys() {
    let _observation = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    use worth_query_execution::publication_boundary::workflow_advance::WorthQueryWorkflowAdvanceAdapter;

    let (application, _, instance, proposal, approval, _) = approval_journey("applied", 960);
    assert!(matches!(
        approve_instance(
            &application,
            instance.clone(),
            &approval,
            &proposal,
            WorkflowApprovalDecision::Approve,
            970,
        ),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    let required = match advance_instance(&application, instance.clone(), 971).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected an approved operation, got {other:?}"),
    };
    let runtime = application.runtime();
    let scope = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &scope);
    let selected = runtime
        .with_host_advancement(|host_phase| {
            runtime
                .on_branch(instance.branch())
                .select(&host_phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let principal_binding = runtime
        .installed_schema()
        .principal_binding(
            super::super::document_retention_model::schema::DocumentPrincipalBinding::reference(),
        )
        .unwrap();
    let principal = runtime
        .with_host_advancement(|host_phase| {
            selected
                .resolve_authenticated_principal(
                    &host_phase,
                    &principal_binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let document = runtime
        .with_host_advancement(|host_phase| {
            selected
                .resolve_entity(
                    &host_phase,
                    DocumentIdentityField::reference(),
                    DOCUMENT_IDENTITY.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let operation = runtime
        .installed_schema()
        .installed_operation(SetRetention::reference())
        .unwrap();
    let input = SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: 8,
    };
    let candidate = |key: u64| {
        let admission = runtime
            .with_host_advancement(|host_phase| {
                selected
                    .authorize_operation(
                        &host_phase,
                        &principal,
                        &document,
                        &operation,
                        Default::default(),
                        &scope,
                    )
                    .unwrap()
            })
            .expect("installed host call opens");
        let HandlerResult::Completed(completed) = runtime
            .with_application_advancement(&scope, |phase| {
                runtime.execute_mutation_handler::<ReviewedSetRetentionBinding>(
                    &phase,
                    &reviewed_identities(&key, &input),
                    principal.principal_identity(),
                    admission,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
            })
            .expect("the fixture policy admits its handler advancement")
            .unwrap()
        else {
            panic!("the reviewed handler must produce a candidate");
        };
        completed.into_parts().0
    };
    let authority = required
        .authority_slot()
        .take()
        .expect("the owner issued one operation authority");
    let extend = |idempotency| {
        WorthQueryWorkflowAdvanceAdapter::bind_operation_idempotency(
            idempotency,
            required.transition_identity_bytes(),
        )
    };
    let sibling_key = 974_u64;
    let sibling_admission = runtime
        .with_host_advancement(|host_phase| {
            selected
                .authorize_operation(
                    &host_phase,
                    &principal,
                    &document,
                    &operation,
                    Default::default(),
                    &scope,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let HandlerResult::Completed(sibling) = runtime
        .with_application_advancement(&scope, |phase| {
            runtime.execute_mutation_handler::<
            super::super::document_retention_model::retention_entry::SetRetentionBinding,
        >(&phase,
&ApplicationMutationIdentities::<DocumentRetentionSchema, SetRetentionBinding>::encode(
                &sibling_key,
                &input,
            )
            .unwrap(),
            principal.principal_identity(),
            sibling_admission,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        })
        .expect("the fixture policy admits its handler advancement")
        .unwrap()
    else {
        panic!("the ordinary sibling handler must produce a candidate");
    };
    let sibling_program = runtime
        .with_host_advancement(|host_phase| {
            sibling
        .into_parts()
        .0
        .bind_workflow_operation_authority(
            &host_phase, &authority,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        })
        .expect("installed host call opens");
    let relabeled = application
        .program_runtime()
        .compare_and_commit_program_action(
            sibling_program,
            &reviewed_identities(&sibling_key, &input),
            extend,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    assert!(matches!(
        relabeled,
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.kind() == WorthQueryApplicationCommitDenialKind::WorkflowAuthorityRequired
    ));
    assert_eq!(read_retention(runtime, instance.branch()), SEED_RETENTION);
    let first_program = runtime
        .with_host_advancement(|host_phase| {
            candidate(972)
        .bind_workflow_operation_authority(
            &host_phase, &authority,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        })
        .expect("installed host call opens");
    let second_program = runtime
        .with_host_advancement(|host_phase| {
            candidate(973)
        .bind_workflow_operation_authority(
            &host_phase, &authority,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        })
        .expect("installed host call opens");
    let first = application
        .program_runtime()
        .compare_and_commit_program_action(
            first_program,
            &reviewed_identities(&972, &input),
            extend,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    assert!(matches!(
        first,
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let second = application
        .program_runtime()
        .compare_and_commit_program_action(
            second_program,
            &reviewed_identities(&973, &input),
            extend,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    assert!(matches!(
        second,
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.kind() == WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
    ));
    assert_eq!(read_retention(runtime, instance.branch()), 8);
}

// This court alone exercises the workflow owner below publication.
mod advancement_identity {
    //! The real workflow publication adapter accepts only its installed owner's phase.
    use super::super::super::document_retention_model::{
        schema::{Document, DocumentPrincipalBinding, DocumentRetentionSchema},
        workflow::{WorkflowAdvanceCapability, WorkflowAdvanceInput, WorkflowAdvanceOperation},
    };
    use super::*;
    use worth_query_execution::publication_boundary::workflow_advance::{
        PreparedWorkflowAdvance, WorthQueryWorkflowAdvanceAdapter as Adapter,
    };
    use worth_query_host::facade::{
        application_contribution::WorthQueryManagedComputationResourceDenial as Resource,
        primary_graph::{
            advancement_requests_on_this_thread_for_test as reports,
            installed_source_reads_on_this_thread_for_test as reads,
            place_managed_computations_on_this_thread_for_test as place,
            WorthQueryApplicationIdempotencyBinding,
            WorthQueryExecutionPlacementForTest as Placement, WorthQueryPrincipalResolutionMode,
        },
    };
    type Prepared = PreparedWorkflowAdvance<
        DocumentRetentionSchema,
        WorkflowAdvanceOperation,
        WorkflowAdvanceInput,
        Document,
    >;
    fn fixture(
        key: u64,
    ) -> (
        DocumentWorkflowRuntime,
        Prepared,
        WorthQueryApplicationIdempotencyBinding,
    ) {
        let (application, _, instance, _, _, _) = approval_journey("identity", key);
        let runtime = application.runtime();
        let scope = request_scope();
        let external = authenticate_operator(runtime.installed_schema(), &scope);
        let prepared = runtime
            .with_application_advancement(&scope, |_phase| {
                let selected = runtime
                    .on_branch(instance.branch())
                    .select(&_phase)
                    .unwrap();
                let binding = runtime
                    .installed_schema()
                    .principal_binding(DocumentPrincipalBinding::reference())
                    .unwrap();
                let principal = selected
                    .resolve_authenticated_principal(
                        &_phase,
                        &binding,
                        &external,
                        &scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let operation = runtime
                    .installed_schema()
                    .installed_operation(WorkflowAdvanceOperation::reference())
                    .unwrap();
                let capability = runtime
                    .installed_schema()
                    .capability(
                        WorkflowAdvanceCapability::reference(),
                        WorkflowAdvanceOperation::reference(),
                    )
                    .unwrap();
                let access = selected
                    .admit_capability_access(
                        &_phase,
                        &principal,
                        &capability,
                        WorkflowAdvanceInput {
                            document_identity: DOCUMENT_IDENTITY.to_owned(),
                        },
                        &scope,
                    )
                    .unwrap();
                let admission = runtime
                    .authorize_capability_operation(&_phase, access, &operation, Default::default())
                    .unwrap();
                Adapter::prepare::<_, WorkflowAdvanceCapability, _, _, _, _>(
                    &_phase,
                    &selected,
                    application.vocabulary().workflow_spec_for(&selected),
                    instance,
                    admission,
                )
                .unwrap()
            })
            .unwrap();
        let idempotency = WorthQueryApplicationIdempotencyBinding::for_host_commit::<
            DocumentRetentionSchema,
            WorkflowAdvanceOperation,
            _,
            _,
        >(&key, &"cross-runtime-phase")
        .unwrap();
        (application, prepared, idempotency)
    }
    #[test]
    fn foreign_phase_cannot_advance_another_workflow() {
        let _observation = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
        for placement in [
            Placement::Serial,
            Placement::Leased(std::num::NonZeroUsize::MIN),
        ] {
            let previous = place(placement);
            let (a, ap, ai) = fixture(1_400);
            let (b, bp, bi) = fixture(1_500);
            let scope = request_scope();
            reports();
            a.runtime().with_application_advancement(&scope, |phase| {
            let before = reads();
            let stopped = Adapter::compare_and_commit(&phase, b.runtime(), bp, bi);
            assert!(matches!(stopped, WorkflowProgressOutcome::Application(
                WorthQueryApplicationUncommitted::Denied(ref denial)
            ) if matches!(denial.kind(), WorthQueryApplicationCommitDenialKind::ExecutionResource {
                denial: Resource::ForeignAdvancementPhase, partition_identity: None, policy_ancestor: None,
            })));
            assert_eq!(reads(), before);
        }).unwrap();
            let refused = reports();
            assert_eq!(refused.len(), 1);
            assert_eq!(refused[0].as_ref().unwrap().charged_work(), 0);
            a.runtime()
                .with_application_advancement(&scope, |phase| {
                    let before = reads();
                    assert!(matches!(
                        Adapter::compare_and_commit(&phase, a.runtime(), ap, ai),
                        WorkflowProgressOutcome::AwaitingApproval(_)
                    ));
                    assert!(
                        reads() > before,
                        "A's same workflow door revalidates its own source"
                    );
                })
                .unwrap();
            place(previous);
        }
    }
}
