//! Retirement courts for installed program support and its execution-owned users.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramAdoptionPreparationDenial, WorthQueryApplicationRequestExt,
    WorthQueryBranchAdoptionPublicationOutcome,
};
use worth_query_host::facade::application_installation::{
    WorthQueryProgramOwner, WorthQueryProgramSupportRetirementFailure,
};
use worth_query_host::facade::domain::{
    WorthQueryProgramAdoptionRequirementsDenial, WorthQueryProgramSupportRetirementDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial;

use crate::document_retention_model::host::publish_on_first_program;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use crate::document_retention_model::programs::RetentionProgramP1;

#[test]
fn current_branch_prevents_program_support_retirement() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let host = publish_on_first_program();
    let source = *host.owned_revision();

    let denial = host
        .retire_program_support(request_execution, &source)
        .expect_err("the live root still requires P0 support");
    let WorthQueryProgramSupportRetirementFailure::Policy(
        WorthQueryProgramSupportRetirementDenial::CurrentBranches(inventory),
    ) = denial
    else {
        panic!("the live branch must be the stated retirement blocker")
    };
    assert_eq!(inventory.revision(), &source);
    assert_eq!(inventory.current_branches(), 1);
    assert_eq!(inventory.retained_interpretations(), 0);
    assert_eq!(inventory.mandatory_custody(), 0);
    assert!(inventory.retained_program_bytes() > 0);
    assert!(!inventory.can_retire());
}

#[test]
fn retained_interpretation_blocks_retirement_after_the_live_branch_moves_on() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let host = publish_on_first_program();
    let branch = host.current_world();
    let source = *host.owned_revision();
    let retained = host
        .with_host_advancement(|host_phase| {
            host.runtime()
                .on_branch(branch)
                .select(&host_phase)
                .expect("the exact P0 branch selection is retained")
        })
        .expect("installed host call opens");
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision();
    adopt(&host, branch, &target);

    let denial = host
        .retire_program_support(request_execution, &source)
        .expect_err("the retained P0 interpretation remains a live user");
    let WorthQueryProgramSupportRetirementFailure::Policy(
        WorthQueryProgramSupportRetirementDenial::RetainedInterpretation(inventory),
    ) = denial
    else {
        panic!("the retained interpretation must be the stated retirement blocker")
    };
    assert_eq!(inventory.current_branches(), 0);
    assert_eq!(inventory.retained_interpretations(), 1);
    assert_eq!(inventory.mandatory_custody(), 0);

    drop(retained);
    let receipt = host
        .retire_program_support(request_execution, &source)
        .expect("P0 retires after its final retained interpretation is released");
    assert_eq!(receipt.inventory().revision(), &source);
    assert!(receipt.inventory().can_retire());
    assert!(receipt.inventory().retained_program_bytes() > 0);
    assert!(matches!(
        host.retire_program_support(request_execution, &source),
        Err(WorthQueryProgramSupportRetirementFailure::Policy(WorthQueryProgramSupportRetirementDenial::AlreadyRetired { revision }))
            if revision == source
    ));
}

#[test]
fn prepared_adoption_custody_blocks_target_support_retirement() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    let prepared = programs
        .adopt(&requirements)
        .expect("declared adoption capability admits")
        .prepare(64)
        .expect("the adoption prepares with exact support custody");

    let denial = host
        .retire_program_support(request_execution, &target)
        .expect_err("the prepared adoption still requires target support");
    let WorthQueryProgramSupportRetirementFailure::Policy(
        WorthQueryProgramSupportRetirementDenial::MandatoryCustody(inventory),
    ) = denial
    else {
        panic!("mandatory adoption custody must be the stated retirement blocker: {denial:?}")
    };
    assert_eq!(inventory.current_branches(), 0);
    assert_eq!(inventory.retained_interpretations(), 0);
    assert_eq!(inventory.mandatory_custody(), 1);

    drop(prepared);
    let receipt = host
        .retire_program_support(request_execution, &target)
        .expect("P1 retires after prepared custody is released");
    assert!(receipt.inventory().can_retire());
    assert!(host.supported_program::<RetentionProgramP1>().is_none());
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    assert!(matches!(
        programs.compare(&target),
        Err(WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
            WorthQueryBranchAdoptionPreparationDenial::Requirements(
                WorthQueryProgramAdoptionRequirementsDenial::UnrosteredTarget { revision }
            )
        )) if revision == target
    ));
}

pub(super) fn adopt(
    host: &crate::document_retention_model::host::DocumentRetentionRuntime<
        crate::document_retention_model::programs::RetentionProgramP0,
    >,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    target: &worth_query_host::facade::declaration::application_program::ApplicationProgramRevision,
) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(target).expect("P0 to P1 compares");
    let prepared = programs
        .adopt(&requirements)
        .expect("declared adoption capability admits")
        .prepare(64)
        .expect("the adoption prepares");
    assert!(matches!(
        prepared.publish(host.runtime(), &principal, &scope),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
}
