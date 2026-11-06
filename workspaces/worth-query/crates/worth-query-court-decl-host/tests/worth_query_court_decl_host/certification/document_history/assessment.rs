//! Real action admission distinguishes model identity from branch targeting.
use super::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationHistoryDenial as Denial, WorthQueryApplicationRequestMutationDenial,
        WorthQueryDocumentHistoryDenial as NativeDenial, WorthQuerySourceOnlyOperation,
        WorthQuerySourceOnlyPosture,
    },
    declaration::application_program::{
        ApplicationActionReference, ApplicationRootComposition, ApplicationSourceCompletion,
    },
    product::WorthQueryProductBranch,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    presented_request::set_retention,
    programs::{completion::RetentionCompletion, DocumentRetentionFeature},
    record_history::reaffirm,
    retention_entry::{SetRetentionBinding, SetRetentionInput, SetRetentionIntent},
    settled_verdict::{settle, RetentionVerdict},
};

type Action = ApplicationActionReference<
    DocumentRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    SetRetentionBinding,
    RetentionCompletion,
>;
type Operation = WorthQuerySourceOnlyOperation<
    DocumentRetentionSchema,
    SetRetentionIntent,
    RetentionProgramP0,
    Action,
    ApplicationSourceCompletion,
>;

fn operation(host: &App, branch: WorthQueryProductBranch, identity: &str, key: u64) -> Operation {
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(SetRetentionIntent {
            input: SetRetentionInput {
                identity: identity.to_owned(),
                retention_days: 3,
            },
        })
        .without_source()
        .idempotency_owned(key)
        .unwrap_or_else(|(_, _, cause)| panic!("capture: {cause:?}"))
        .retain_operation::<RetentionProgramP0, Action>(host)
        .unwrap_or_else(|(_, cause)| panic!("action: {cause:?}"))
}

fn close_document(host: &App, document: Document<'_>) {
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn edit_assessment_compares_the_actual_model_and_preserves_both_original_owners() {
    let host = publish_on_first_program();
    let original = head(&host);
    let branch = original.branch();
    let document = attach(&host, original, DOCUMENT_IDENTITY);
    let untouched = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let wrong_model = operation(&host, branch, RELATED_DOCUMENT_IDENTITY, 0x9182_0300);
    let lawful = operation(&host, branch, DOCUMENT_IDENTITY, 0x9182_0301);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    assert!(matches!(
        document.assess_edit(&request, &wrong_model),
        Err(Denial::Document(NativeDenial::ForeignModel))
    ));
    for _ in 0..2 {
        document.assess_edit(&request, &lawful).unwrap();
    }
    assert_eq!(wrong_model.posture(), WorthQuerySourceOnlyPosture::Initial);
    assert_eq!(lawful.posture(), WorthQuerySourceOnlyPosture::Initial);
    assert_eq!(value(&host, &document), 7);
    assert_eq!(value(&host, &untouched), 7);
    wrong_model
        .close(&host.runtime().request(&principal, &scope).on_branch(branch))
        .unwrap_or_else(|(_, cause)| panic!("close: {cause:?}"));
    lawful
        .close(&host.runtime().request(&principal, &scope).on_branch(branch))
        .unwrap_or_else(|(_, cause)| panic!("close: {cause:?}"));
    close_document(&host, document);
    close_document(&host, untouched);
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn edit_assessment_refuses_a_genuine_installed_but_undeclared_operation() {
    type OtherAction = ApplicationActionReference<
        DocumentRetentionSchema,
        ApplicationRootComposition,
        DocumentRetentionFeature,
        reaffirm::Binding,
        RetentionCompletion,
    >;
    let host = publish_on_first_program();
    let original = head(&host);
    let branch = original.branch();
    let document = attach(&host, original, DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope).on_branch(branch);
    let other = request
        .mutate(reaffirm::Intent(SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days: 3,
        }))
        .without_source()
        .idempotency_owned(0x9182_0302)
        .unwrap_or_else(|(_, _, cause)| panic!("capture: {cause:?}"))
        .retain_operation::<RetentionProgramP0, OtherAction>(&host)
        .unwrap_or_else(|(_, cause)| panic!("action: {cause:?}"));
    other.assess_current_authorization(&request).unwrap();
    assert!(matches!(
        document.assess_edit(&request, &other),
        Err(Denial::Document(NativeDenial::OperationNotDeclared))
    ));
    let lawful = operation(&host, branch, DOCUMENT_IDENTITY, 0x9182_0303);
    document.assess_edit(&request, &lawful).unwrap();
    assert_eq!(value(&host, &document), 7);
    other
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("close: {cause:?}"));
    lawful
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("close: {cause:?}"));
    close_document(&host, document);
}

#[test]
fn edit_assessment_uses_fresh_authority_and_the_original_action_runtime_and_branch() {
    use worth_query_host::facade::admission::authenticated_principal::{
        WorthQueryCancellationSource, WorthQueryRequestScope,
    };
    use worth_query_host::facade::application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    };
    let host = publish_on_first_program();
    let foreign = publish_on_first_program();
    let original = head(&host);
    let branch = original.branch();
    let document = attach(&host, original, DOCUMENT_IDENTITY);
    let sibling = head(&host);
    let foreign_head = head(&foreign);
    let wrong_branch = operation(&host, sibling.branch(), DOCUMENT_IDENTITY, 0x9182_0304);
    let wrong_runtime = operation(
        &foreign,
        foreign_head.branch(),
        DOCUMENT_IDENTITY,
        0x9182_0305,
    );
    let lawful = operation(&host, branch, DOCUMENT_IDENTITY, 0x9182_0306);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    for wrong in [&wrong_branch, &wrong_runtime] {
        assert!(matches!(
            document.assess_edit(&request, wrong),
            Err(Denial::Mutation(
                WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch
            ))
        ));
    }
    let cancellation = WorthQueryCancellationSource::new();
    let stopped_scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let stopped_principal =
        authenticate_operator(host.runtime().installed_schema(), &stopped_scope);
    cancellation.cancel();
    assert!(matches!(
        document.assess_edit(
            &host.runtime().request(&stopped_principal, &stopped_scope),
            &lawful
        ),
        Err(Denial::ExecutionRequest(
            WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
            )
        ))
    ));
    document.assess_edit(&request, &lawful).unwrap();
    assert_eq!(value(&host, &document), 7);
    lawful
        .close(&request.on_branch(branch))
        .unwrap_or_else(|(_, cause)| panic!("close: {cause:?}"));
    wrong_branch
        .close(
            &host
                .runtime()
                .request(&principal, &scope)
                .on_branch(sibling.branch()),
        )
        .unwrap_or_else(|(_, cause)| panic!("close: {cause:?}"));
    let foreign_scope = request_scope();
    let foreign_principal =
        authenticate_operator(foreign.runtime().installed_schema(), &foreign_scope);
    let foreign_request = foreign
        .runtime()
        .request(&foreign_principal, &foreign_scope)
        .on_branch(foreign_head.branch());
    wrong_runtime
        .close(&foreign_request)
        .unwrap_or_else(|(_, cause)| panic!("close: {cause:?}"));
    close_document(&host, document);
    host.with_host_advancement(|phase| sibling.close(&phase).unwrap())
        .unwrap();
    foreign
        .with_host_advancement(|phase| foreign_head.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn edit_assessment_rejects_equal_value_head_aba_instead_of_rebinding() {
    let host = publish_on_first_program();
    let original = head(&host);
    let branch = original.branch();
    let document = attach(&host, original, DOCUMENT_IDENTITY);
    let lawful = operation(&host, branch, DOCUMENT_IDENTITY, 0x9182_0307);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    document.assess_edit(&request, &lawful).unwrap();
    assert_eq!(
        settle(set_retention(&host, branch, 3, 0x9182_0308)),
        RetentionVerdict::Performed(3)
    );
    assert_eq!(
        settle(set_retention(&host, branch, 7, 0x9182_0309)),
        RetentionVerdict::Performed(7)
    );
    assert!(matches!(
        document.assess_edit(&request, &lawful),
        Err(Denial::Document(NativeDenial::HeadDiverged))
    ));
    assert_eq!(lawful.posture(), WorthQuerySourceOnlyPosture::Initial);
    lawful
        .close(&request.on_branch(branch))
        .unwrap_or_else(|(_, cause)| panic!("close: {cause:?}"));
    close_document(&host, document);
}
