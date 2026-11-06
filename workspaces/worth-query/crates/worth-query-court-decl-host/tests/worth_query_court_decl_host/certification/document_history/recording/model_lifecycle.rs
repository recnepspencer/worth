//! Same domain lookup and values do not restore an earlier model occurrence.
use super::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationHistoryDenial, WorthQueryApplicationMutationOutcome,
        WorthQueryDocumentHistoryDenial,
        WorthQueryHistoryNavigationAdvanceDenial as NavigationFailure,
        WorthQueryHistoryNavigationDenial as NavigationDenial,
    },
    application_installation::WorthQueryApplicationProgramRoster,
    primary_graph::{
        WorthQueryOperationAuthorizationDenialKind as AuthorizationDenial,
        WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal,
        WorthQueryPrincipalResolutionMode,
    },
    product::WorthQueryProductBranch,
    runtime::ExecutionAllocationPolicy,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    document_history::navigation::Navigation,
    host::{publish, DocumentRetentionRuntime},
    model_lifecycle::declaration::{ModelLifecycleChanged, ModelLifecycleDenial},
    model_lifecycle::{
        validated_model_lifecycle_program, ModelLifecycleInput, ModelLifecycleIntent,
        ModelLifecycleProgram,
    },
    programs::validated_second_program,
    schema::DocumentIdentityField,
};

fn identity(runtime: &Runtime, branch: WorthQueryProductBranch) -> (u32, u64, u32) {
    let scope = request_scope();
    runtime
        .with_host_advancement(|phase| {
            let selected = runtime.on_branch(branch).select(&phase).unwrap();
            // Only a description for comparison. It is never used to admit the
            // history document, lifecycle operation, replacement head or edit.
            let issued = selected
                .resolve_entity(
                    &phase,
                    DocumentIdentityField::at("DocumentIdentityField"),
                    DOCUMENT_IDENTITY.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Certification,
                )
                .unwrap()
                .relational_record_identity_parts();
            (
                issued.partition_id(),
                issued.local_slot(),
                issued.generation(),
            )
        })
        .unwrap()
}

fn change(
    host: &DocumentRetentionRuntime<ModelLifecycleProgram>,
    branch: WorthQueryProductBranch,
    identity: &str,
    restore: bool,
    key: u64,
) -> WorthQueryApplicationMutationOutcome<ModelLifecycleDenial, ModelLifecycleChanged> {
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .advanced()
        .mutate(ModelLifecycleIntent {
            input: ModelLifecycleInput {
                context: RELATED_DOCUMENT_IDENTITY.to_owned(),
                identity: identity.to_owned(),
                restore,
            },
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}

#[test]
fn recreated_same_name_model_refuses_the_old_document_and_admits_a_fresh_owned_history() {
    let host = publish(
        validated_model_lifecycle_program(),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .unwrap();
    let original = head(&host);
    let branch = original.branch();
    let old_identity = identity(host.runtime(), branch);
    let document = attach(&host, original, DOCUMENT_IDENTITY);
    let untouched = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let old_root = document.current_step(&request).unwrap();
    let navigation = document
        .navigate::<Navigation>()
        .checkout(old_root.id())
        .expected_selection(document.selection(&request).unwrap())
        .idempotency(0x9182_0498)
        .prepare(&request)
        .unwrap();
    let completed = super::super::navigation::complete(&host, navigation);
    assert_eq!(completed.requested_node(), old_root.id());
    assert_eq!(value(&host, &document), 7);
    let branch = completed.selection().physical_endpoint().product_branch();
    let old_selection = document.selection(&request).unwrap();
    let refused = change(&host, branch, RELATED_DOCUMENT_IDENTITY, false, 0x9182_0499);
    assert!(
        matches!(
            refused,
            WorthQueryApplicationMutationOutcome::DomainDenied(ModelLifecycleDenial::WrongState)
        ),
        "the lifecycle operation cannot retire its surviving context: {refused:?}"
    );
    assert_eq!(value(&host, &document), 7);
    for (restore, key) in [(false, 0x9182_0500), (true, 0x9182_0501)] {
        let changed = change(&host, branch, DOCUMENT_IDENTITY, restore, key);
        assert!(
            matches!(
                changed,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{changed:?}"
        );
    }
    let new_identity = identity(host.runtime(), branch);
    assert_ne!(
        old_identity, new_identity,
        "Native must actually issue a different full record identity"
    );
    assert_eq!(read_retention(host.runtime(), branch), 7);
    let related = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .query(DocumentRetentionRead {
            identity: RELATED_DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(related.rows()[0].retention_days, 7);
    drop(related);
    assert!(
        matches!(
            document.read(&request),
            Err(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::ForeignModel
            ))
        ),
        "the same lookup name must not rebind the old model owner"
    );

    let refused = document
        .navigate::<Navigation>()
        .checkout(old_root.id())
        .expected_selection(old_selection)
        .idempotency(0x9182_0503)
        .prepare(&request)
        .unwrap_err();
    assert!(
        matches!(
            refused,
            NavigationFailure::Navigation(NavigationDenial::Document(
                WorthQueryDocumentHistoryDenial::ForeignModel
            ))
        ),
        "navigation must not rebind the retired model by its reused name: {refused:?}"
    );

    // Acquire the lawful replacement head before retiring the old document's
    // actual branch. The issued record descriptions above grant no custody.
    let fresh = attach(&host, head_from(&host, branch), DOCUMENT_IDENTITY);
    let fresh_root = fresh.current_step(&request).unwrap();
    assert_ne!(fresh_root.id(), old_root.id());
    assert_eq!(value(&host, &fresh), 7);
    close(&host, document, 2);
    let edit = fresh
        .edit(SetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 3,
            },
        })
        .idempotency_owned(&request, 0x9182_0502)
        .unwrap()
        .retain_in_program::<ModelLifecycleProgram, Action>(&host)
        .unwrap_or_else(|(_, cause)| {
            panic!("ordinary action from the same installed contribution: {cause:?}")
        });
    let (fresh, step) = finish(&host, ready(&host, edit), 3);
    assert_eq!(step.parent(), Some(fresh_root.id()));
    let refused = fresh
        .navigate::<Navigation>()
        .checkout(fresh_root.id())
        .expected_selection(fresh.selection(&request).unwrap())
        .idempotency(0x9182_0504)
        .prepare(&request)
        .unwrap_err();
    assert!(
        matches!(refused, NavigationFailure::Navigation(
            NavigationDenial::Authorization(ref cause)
        ) if cause.kind() == AuthorizationDenial::Refused(AuthorizationRefusal::CapabilityGrantMissing)),
        "the original grant cannot follow a recreated model by lookup name: {refused:?}"
    );
    assert_eq!(value(&host, &fresh), 3);
    assert_eq!(value(&host, &untouched), 7);
    close(&host, fresh, 2);
    close(&host, untouched, 1);
}
