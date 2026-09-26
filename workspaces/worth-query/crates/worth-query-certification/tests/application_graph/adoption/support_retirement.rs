//! Retirement courts for installed program support and its execution-owned users.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryApplicationProgramAdoptionRecoveryFailure, WorthQueryApplicationRequestExt,
    WorthQueryBranchAdoptionPublicationOutcome, WorthQueryBranchAdoptionRecoveryOutcome,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::domain::{
    WorthQueryProgramAdoptionRequirementsDenial, WorthQueryProgramSupportRetirementDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial;

use crate::bounded_dimension_model::host::publish_on_first_program;
use crate::bounded_dimension_model::operator_identity::{authenticate_operator, request_scope};
use crate::bounded_dimension_model::programs::DimensionProgramP1;

#[test]
fn current_branch_prevents_program_support_retirement() {
    let host = publish_on_first_program();
    let source = host.owned_revision().clone();

    let denial = host
        .retire_program_support(&source)
        .expect_err("the live root still requires P0 support");
    let WorthQueryProgramSupportRetirementDenial::CurrentBranches(inventory) = denial else {
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
    let host = publish_on_first_program();
    let branch = host.current_world();
    let source = host.owned_revision().clone();
    let retained = host
        .runtime()
        .on_branch(branch)
        .select()
        .expect("the exact P0 branch selection is retained");
    let target = host
        .supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision()
        .clone();
    adopt(&host, branch, &target);

    let denial = host
        .retire_program_support(&source)
        .expect_err("the retained P0 interpretation remains a live user");
    let WorthQueryProgramSupportRetirementDenial::RetainedInterpretation(inventory) = denial else {
        panic!("the retained interpretation must be the stated retirement blocker")
    };
    assert_eq!(inventory.current_branches(), 0);
    assert_eq!(inventory.retained_interpretations(), 1);
    assert_eq!(inventory.mandatory_custody(), 0);

    drop(retained);
    let receipt = host
        .retire_program_support(&source)
        .expect("P0 retires after its final retained interpretation is released");
    assert_eq!(receipt.inventory().revision(), &source);
    assert!(receipt.inventory().can_retire());
    assert!(receipt.inventory().retained_program_bytes() > 0);
    assert!(matches!(
        host.retire_program_support(&source),
        Err(WorthQueryProgramSupportRetirementDenial::AlreadyRetired { revision })
            if revision == source
    ));
}

#[test]
fn prepared_adoption_custody_blocks_target_support_retirement() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = host
        .supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision()
        .clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    let prepared = programs
        .adopt(&target)
        .requirements(&requirements)
        .prepare(64)
        .expect("the adoption prepares with exact support custody");

    let denial = host
        .retire_program_support(&target)
        .expect_err("the prepared adoption still requires target support");
    let WorthQueryProgramSupportRetirementDenial::MandatoryCustody(inventory) = denial else {
        panic!("mandatory adoption custody must be the stated retirement blocker: {denial:?}")
    };
    assert_eq!(inventory.current_branches(), 0);
    assert_eq!(inventory.retained_interpretations(), 0);
    assert_eq!(inventory.mandatory_custody(), 1);

    drop(prepared);
    let receipt = host
        .retire_program_support(&target)
        .expect("P1 retires after prepared custody is released");
    assert!(receipt.inventory().can_retire());
    assert!(host.supported_program::<DimensionProgramP1>().is_none());
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

#[test]
fn unpublished_adoption_prevents_retirement_until_exact_recovery_is_released() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = host
        .supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision()
        .clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    let prepared = programs
        .adopt(&target)
        .requirements(&requirements)
        .prepare(64)
        .expect("the adoption prepares with exact support custody");
    host.runtime().fail_next_durable_append_for_test();
    let unpublished = match prepared.publish() {
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => unpublished,
        _ => panic!("the injected durability loss must retain adoption custody"),
    };

    let denial = host
        .retire_program_support(&target)
        .expect_err("the unpublished adoption's custody holds the target support");
    // The branch still publishes P0, so the inventory is complete: only the
    // retained adoption custody uses P1.
    let WorthQueryProgramSupportRetirementDenial::MandatoryCustody(inventory) = denial else {
        panic!("the unpublished adoption must be named as mandatory custody: {denial:?}")
    };
    assert_eq!(inventory.revision(), &target);
    assert_eq!(inventory.current_branches(), 0);
    assert_eq!(inventory.retained_interpretations(), 0);
    assert_eq!(inventory.mandatory_custody(), 1);
    assert!(inventory.retained_program_bytes() > 0);
    assert!(host.supported_program::<DimensionProgramP1>().is_some());
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .recover(unpublished.into_recovery())
        .unwrap_or_else(|failure| match failure {
            WorthQueryApplicationProgramAdoptionRecoveryFailure::Recovery(failure) => {
                panic!(
                    "the exact recovery custody remains usable: {:?}",
                    failure.denial()
                )
            }
            WorthQueryApplicationProgramAdoptionRecoveryFailure::ProductSelection {
                denial,
                ..
            } => {
                panic!("the exact recovery branch remains selectable: {denial:?}")
            }
        });
    assert!(matches!(
        outcome,
        WorthQueryBranchAdoptionRecoveryOutcome::Performed { .. }
    ));
    let denial = host
        .retire_program_support(&target)
        .expect_err("performed recovery makes P1 the live branch program");
    assert!(matches!(
        denial,
        WorthQueryProgramSupportRetirementDenial::CurrentBranches(_)
    ));
}

#[test]
fn denied_recovery_release_preserves_support_custody_for_retry() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = host
        .supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision()
        .clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    let prepared = programs
        .adopt(&target)
        .requirements(&requirements)
        .prepare(64)
        .expect("the adoption prepares with exact support custody");
    host.runtime().fail_next_durable_append_for_test();
    let recovery = match prepared.publish() {
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => {
            unpublished.into_recovery()
        }
        _ => panic!("the injected durability loss must retain adoption custody"),
    };

    let recovery = match host
        .runtime()
        .release_branch_adoption_recovery(recovery, u64::MAX)
    {
        Ok(_) => panic!("the intentionally age-gated recovery must remain retained"),
        Err(failure) => failure
            .into_recovery()
            .expect("a retryable release denial must return complete adoption recovery"),
    };
    let denial = host
        .retire_program_support(&target)
        .expect_err("returned recovery must continue owning target support custody");
    let WorthQueryProgramSupportRetirementDenial::MandatoryCustody(inventory) = denial else {
        panic!("the returned recovery must be named as mandatory custody: {denial:?}")
    };
    assert_eq!(inventory.current_branches(), 0);
    assert_eq!(inventory.mandatory_custody(), 1);

    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .recover(recovery)
        .unwrap_or_else(|failure| match failure {
            WorthQueryApplicationProgramAdoptionRecoveryFailure::Recovery(failure) => {
                panic!(
                    "returned recovery must remain usable: {:?}",
                    failure.denial()
                )
            }
            WorthQueryApplicationProgramAdoptionRecoveryFailure::ProductSelection {
                denial,
                ..
            } => panic!("returned recovery branch must remain selectable: {denial:?}"),
        });
    assert!(matches!(
        outcome,
        WorthQueryBranchAdoptionRecoveryOutcome::Performed { .. }
    ));
    assert!(matches!(
        host.retire_program_support(&target),
        Err(WorthQueryProgramSupportRetirementDenial::CurrentBranches(_))
    ));
}

pub(super) fn adopt(
    host: &crate::bounded_dimension_model::host::BoundedDimensionRuntime<
        crate::bounded_dimension_model::programs::DimensionProgramP0,
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
        .adopt(target)
        .requirements(&requirements)
        .prepare(64)
        .expect("the adoption prepares");
    assert!(matches!(
        prepared.publish(),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
}
