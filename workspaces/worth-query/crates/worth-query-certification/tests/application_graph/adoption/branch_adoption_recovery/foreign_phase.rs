//! A real unpublished adoption reaches its private continuation with a foreign phase.
use super::*;
use worth_query_host::facade::application_contribution::{
    WorthQueryAdvancementDenial, WorthQueryManagedComputationResourceDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryExecutionObservation;
#[test]
fn recovery_continuation_refuses_a_foreign_phase_and_retains_custody() {
    let observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = publish_on_first_program();
    let foreign = publish_on_first_program();
    let branch = host.current_world();
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).unwrap();
    let prepared = programs.adopt(&requirements).prepare(64).unwrap();
    let selected = host
        .with_host_advancement(|phase| {
            let selected = host.runtime().on_branch(branch).select(&phase).unwrap();
            let execution = phase
                .checked_execution_request_for(host.runtime().product_runtime())
                .unwrap();
            host.runtime().fail_next_durable_append_for_test(&execution);
            selected
        })
        .unwrap();
    let WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) =
        prepared.publish()
    else {
        panic!("durability refusal must retain real adoption custody");
    };
    let recovery = foreign
        .with_host_advancement(|foreign_phase| {
            let failure = observation
                .recover_branch_adoption_in_advancement_for_test(
                    &selected,
                    &foreign_phase,
                    unpublished.into_recovery(),
                    &scope,
                )
                .err()
                .expect("a foreign phase must refuse before recovery inspection");
            match failure.denial() {
                WorthQueryBranchAdoptionRecoveryDenial::ExecutionDenied(cause) => assert_eq!(
                    *cause,
                    WorthQueryAdvancementDenial::Resource(
                        WorthQueryManagedComputationResourceDenial::ForeignAdvancementPhase
                    )
                ),
                other => panic!("the continuation must keep its phase cause: {other:?}"),
            }
            failure.into_recovery()
        })
        .unwrap();
    drop(selected);
    let recovered = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .recover(recovery)
        .unwrap_or_else(|_| panic!("the exact custody remains recoverable"));
    let WorthQueryBranchAdoptionRecoveryOutcome::Performed { adoption, cleanup } = recovered else {
        panic!("the owning continuation must settle the retained effect");
    };
    cleanup.unwrap();
    assert_eq!(adoption.target(), &target);
    assert_eq!(adoption.relational_owner_contacts(), 0);
}
