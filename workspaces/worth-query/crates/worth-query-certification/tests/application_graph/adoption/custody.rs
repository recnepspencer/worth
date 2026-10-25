//! Owner-derived custody obligations and the owner gates that enforce them.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::declaration::application_program::ApplicationSemanticChangeKind;
use worth_query_host::facade::domain::WorthQueryProgramCustodyInventoryKind;
use worth_query_host::facade::primary_graph::WorthQueryProgramCustodyDispositionKind;

use crate::document_retention_model::host::publish_on_first_program;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use crate::document_retention_model::presented_request::set_retention;
use crate::document_retention_model::programs::{
    ChangedOperationRetentionProgram, RemovedOperationRetentionProgram,
};
use crate::document_retention_model::settled_verdict::{settle, RetentionVerdict};

#[test]
fn removed_operation_derives_retire_disposition_and_closes_its_source_owner() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = *host
        .supported_program::<RemovedOperationRetentionProgram>()
        .expect("the removal target is rostered")
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs
        .compare(&target)
        .expect("the owner must describe operation removal");
    let removed_operations = requirements
        .custody_inventory_requirements()
        .iter()
        .filter(|requirement| {
            requirement.kind() == WorthQueryProgramCustodyInventoryKind::OperationContinuation
        })
        .collect::<Vec<_>>();
    use crate::document_retention_model::{
        retention_entry::{
            ExplicitReviewedSetRetentionBinding, OccurrenceSetRetentionBinding,
            ReviewedSetRetentionBinding, SetRetentionBinding,
        },
        schema::DocumentRetentionSchema,
    };
    use worth_query_host::facade::declaration::application_operation::ApplicationMutationBinding;
    let removed_bindings = [
        <crate::document_retention_model::correctable_retention::CorrectableRetentionBinding as ApplicationMutationBinding<DocumentRetentionSchema>>::IDENTITY,
        <SetRetentionBinding as ApplicationMutationBinding<DocumentRetentionSchema>>::IDENTITY,
        <ReviewedSetRetentionBinding as ApplicationMutationBinding<DocumentRetentionSchema>>::IDENTITY,
        <OccurrenceSetRetentionBinding as ApplicationMutationBinding<DocumentRetentionSchema>>::IDENTITY,
        <ExplicitReviewedSetRetentionBinding as ApplicationMutationBinding<DocumentRetentionSchema>>::IDENTITY,
    ];
    assert_eq!(removed_operations.len(), removed_bindings.len() * 2);
    for binding in removed_bindings {
        assert!(
            removed_operations
                .iter()
                .any(|requirement| requirement.subject().ends_with(binding)),
            "the removal target retires {binding}"
        );
        assert_eq!(
            removed_operations
                .iter()
                .filter(|requirement| requirement.subject().ends_with(binding))
                .count(),
            2,
            "both operation and delivery retire for {binding}"
        );
    }
    assert!(removed_operations
        .iter()
        .all(|requirement| requirement.change() == ApplicationSemanticChangeKind::Removed));

    let prepared = programs
        .adopt(&requirements)
        .expect("declared adoption capability admits")
        .prepare(64)
        .expect("execution derives the disposition from installed owner truth");
    let expected_custody = prepared.custody().clone();
    let dispositions = expected_custody.dispositions();
    assert_eq!(dispositions.len(), removed_operations.len());
    for requirement in removed_operations {
        let disposition = dispositions
            .iter()
            .find(|disposition| disposition.requirement() == requirement)
            .expect("every removed operation needs a custody disposition");
        assert_eq!(
            disposition.kind(),
            WorthQueryProgramCustodyDispositionKind::RetireUneffectedContinuation
        );
    }

    let performed = match prepared.publish(host.runtime(), &principal, &scope) {
        WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause, .. } => {
            panic!("authorized adoption refused: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::AdmissionDenied { cause, .. } => {
            panic!("execution admission refused: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::ExecutionDenied(cause) => {
            panic!("the declared host policy admits adoption: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::Performed(performed) => performed,
        WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect) => {
            panic!("operation-removal adoption unexpectedly had no effect: {no_effect:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => {
            panic!("operation-removal adoption lost publication: {unpublished:?}")
        }
    };
    assert_eq!(performed.custody(), &expected_custody);
    assert_eq!(
        settle(set_retention(&host, host.current_world(), 8, 0x9175_3c00)),
        RetentionVerdict::inactive(&target),
        "the removed operation cannot continue through its obsolete source-program owner"
    );
}

#[test]
fn changed_operation_requires_and_then_receives_fresh_target_admission() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target_owner = host
        .supported_program::<ChangedOperationRetentionProgram>()
        .expect("the changed-operation target is rostered");
    let target = *target_owner.owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("comparison succeeds");
    let requirement = requirements
        .custody_inventory_requirements()
        .iter()
        .find(|requirement| {
            requirement.kind() == WorthQueryProgramCustodyInventoryKind::OperationContinuation
                && requirement.subject().ends_with(<crate::document_retention_model::retention_entry::SetRetentionBinding as worth_query_host::facade::declaration::application_operation::ApplicationMutationBinding<crate::document_retention_model::schema::DocumentRetentionSchema>>::IDENTITY)
        })
        .expect("changed operation requires continuation disposition");
    assert_eq!(requirement.change(), ApplicationSemanticChangeKind::Changed);

    let prepared = programs
        .adopt(&requirements)
        .expect("declared adoption capability admits")
        .prepare(64)
        .expect("changed operation has an owner-derived disposition");
    assert!(prepared.custody().dispositions().iter().any(|disposition| {
        disposition.requirement() == requirement
            && disposition.kind() == WorthQueryProgramCustodyDispositionKind::FreshCurrentAdmission
    }));
    assert!(matches!(
        prepared.publish(host.runtime(), &principal, &scope),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));

    let adopted = host.current_world();
    assert_eq!(
        settle(set_retention(&host, adopted, 9, 0x9175_3c01)),
        RetentionVerdict::Performed(9),
        "the changed operation executes through a fresh request admitted by the active target"
    );
}
