//! Real program adoption changes eligibility without rewriting model history.
use super::*;
use std::num::NonZeroU64;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryHistoryContextRefreshLimits, WorthQueryHistoryNavigationEligibilityDenial,
    },
    primary_graph::WorthQueryBranchAdoptionPublicationOutcome,
};
use worth_query_testkit_document_retention::facade::document_retention_model::programs::RemovedAssessmentSupplierProgram;

#[test]
fn current_program_change_refuses_the_old_source_and_fresh_supported_history_can_checkout() {
    let host = publish_on_first_program();
    let mut document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let original = document.current_step(&request).unwrap();
    let before = document.selection(&request).unwrap();
    let branch = before.physical_endpoint().product_branch();
    let original_program = host
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .inspect()
        .unwrap();
    let target = *host
        .supported_program::<RemovedAssessmentSupplierProgram>()
        .unwrap()
        .installed_program()
        .revision();
    let programs = host
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).unwrap();
    assert!(!requirements.requires_existing_state_validation());
    assert!(!requirements.requires_migration_assessment());
    let prepared = programs
        .adopt(&requirements)
        .unwrap()
        .prepare(1_024)
        .unwrap();
    assert!(
        matches!(
            prepared.publish(host.runtime(), &principal, &scope),
            WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
        ),
        "publish an actual rostered program on the original document branch"
    );
    let current_program = host
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .inspect()
        .unwrap();
    assert_eq!(current_program.revision(), &target);
    assert_ne!(current_program, original_program);
    document
        .refresh_context(
            &request,
            WorthQueryHistoryContextRefreshLimits {
                maximum_commits: NonZeroUsize::new(8).unwrap(),
                maximum_records: NonZeroUsize::new(256).unwrap(),
                maximum_evidence_bytes: NonZeroU64::new(8 * 1024 * 1024).unwrap(),
            },
        )
        .unwrap();
    let expected = document.selection(&request).unwrap();
    assert_eq!(expected.step(), &original);
    assert_ne!(expected.physical_endpoint(), before.physical_endpoint());
    assert_eq!(value(&host, &document), 7);

    let refused = document
        .navigate::<Navigation>()
        .checkout(original.id())
        .expected_selection(expected.clone())
        .idempotency(0x9182_ef00)
        .prepare(&request)
        .unwrap_err();
    assert!(
        matches!(
            refused,
            Failure::Navigation(Denial::RequiresSemanticCorrection(
                WorthQueryHistoryNavigationEligibilityDenial::ProgramChanged
            ))
        ),
        "old program meaning must refuse at its actual navigation gate: {refused:?}"
    );
    assert_eq!(document.selection(&request).unwrap(), expected);
    assert_eq!(value(&host, &document), 7);

    // A genuine new history owns the new program source. Equal model values
    // did not make the earlier program source eligible.
    let fresh = attach(
        &host,
        super::super::head_from(&host, branch),
        DOCUMENT_IDENTITY,
    );
    let fresh_root = fresh.current_step(&request).unwrap();
    close(&host, document, 1);
    let completed = checkout(
        &host,
        &fresh,
        fresh_root.id(),
        selection(&host, &fresh),
        0x9182_ef01,
    );
    assert_eq!(completed.requested_node(), fresh_root.id());
    assert_ne!(
        completed.selection().physical_endpoint(),
        fresh_root.endpoint()
    );
    assert_eq!(value(&host, &fresh), 7);
    let selected_program = host
        .request(&principal, &scope)
        .on_branch(completed.selection().physical_endpoint().product_branch())
        .programs()
        .inspect()
        .unwrap();
    assert_eq!(selected_program.revision(), &target);
    close(&host, fresh, 2);
}
