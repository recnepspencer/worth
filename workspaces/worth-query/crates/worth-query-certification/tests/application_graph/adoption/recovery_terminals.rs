//! Adversarial terminals reached while continuing exact branch-adoption custody.

use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramAdoptionRecoveryFailure, WorthQueryApplicationRequestExt,
    WorthQueryBranchAdoptionPublicationOutcome, WorthQueryBranchAdoptionRecoveryOutcome,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::primary_graph::{
    RuntimeWorldRecoveryDenial, RuntimeWorldSettledRelationalAdoptionDenial,
    WorthQueryBranchAdoptionRecoveryDenial,
};

use crate::bounded_dimension_model::host::publish_on_first_program;
use crate::bounded_dimension_model::operator_identity::{authenticate_operator, request_scope};
use crate::bounded_dimension_model::programs::DimensionProgramP1;

#[test]
fn cancellation_after_settled_adoption_hands_off_next_custody_and_prior_cleanup() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = host
        .supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision()
        .clone();
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(30),
        cancellation.token(),
    );
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("comparison succeeds");
    let prepared = programs
        .adopt(&target)
        .requirements(&requirements)
        .prepare(64)
        .expect("adoption prepares");
    host.runtime().fail_next_durable_append_for_test();
    let unpublished = match prepared.publish() {
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => unpublished,
        _ => panic!("the first injected durability loss must retain exact custody"),
    };
    let selected_entity_count = unpublished.selected_entity_count();
    let recovery = unpublished.into_recovery();
    let pause = host
        .runtime()
        .world_operation_control_for_test()
        .pause_before_product_compare(NonZeroUsize::new(1).unwrap());

    let outcome = std::thread::scope(|threads| {
        let recovering = threads.spawn(|| {
            host.runtime()
                .request(&principal, &scope)
                .on_branch(branch)
                .programs()
                .recover(recovery)
        });
        assert!(
            pause.wait_until_reached(Duration::from_secs(10)),
            "recovery must reach the bounded final product comparison"
        );
        cancellation.cancel();
        pause.release();
        recovering.join().expect("recovery must not panic")
    })
    .unwrap_or_else(|_| panic!("late cancellation is a terminal, not a recovery denial"));

    let (next, prior_cleanup) = match outcome {
        WorthQueryBranchAdoptionRecoveryOutcome::ProductUnpublished {
            next,
            prior_cleanup,
        } => (next, prior_cleanup),
        _ => panic!("cancellation after the settled owner effect must retain next custody"),
    };
    assert_eq!(
        next.cause(),
        worth_query_host::facade::runtime::ProductUnpublishedCause::CancellationAfterEffect
    );
    prior_cleanup.expect("the consumed prior recovery record must drain independently");
    assert_eq!(next.source(), requirements.source());
    assert_eq!(next.target(), &target);
    assert_eq!(next.selected_entity_count(), selected_entity_count);
    assert!(
        !next.relational_requires_settlement(),
        "the settled Relational result must not regain a replay obligation"
    );
    host.runtime()
        .release_branch_adoption_recovery(next.into_recovery(), 0)
        .expect("the exact next recovery record must release cleanly");
}

#[test]
fn unavailable_settlement_evidence_returns_the_same_recovery_for_retry() {
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
    let requirements = programs.compare(&target).expect("comparison succeeds");
    let prepared = programs
        .adopt(&target)
        .requirements(&requirements)
        .prepare(64)
        .expect("adoption prepares");
    host.runtime().fail_next_durable_append_for_test();
    let recovery = match prepared.publish() {
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => {
            unpublished.into_recovery()
        }
        _ => panic!("the first durability loss must retain exact custody"),
    };

    host.runtime().fail_next_durable_append_for_test();
    let failure = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .recover(recovery)
        .err()
        .expect("the second durability loss must deny settled-adoption preparation");
    let WorthQueryApplicationProgramAdoptionRecoveryFailure::Recovery(failure) = failure else {
        panic!("the selected branch remains valid during settlement")
    };
    assert!(matches!(
        failure.denial(),
        WorthQueryBranchAdoptionRecoveryDenial::AdoptionPreparation(
            RuntimeWorldSettledRelationalAdoptionDenial::Recovery(
                RuntimeWorldRecoveryDenial::SettlementEvidenceUnavailable
            )
        )
    ));
    let recovery = failure.into_recovery();

    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .recover(recovery)
        .unwrap_or_else(|_| panic!("the retained recovery must retry after the one-shot fault"));
    match outcome {
        WorthQueryBranchAdoptionRecoveryOutcome::Performed { cleanup, .. } => {
            cleanup.expect("the retried recovery and prior record must drain");
        }
        _ => panic!("the exact retry must finish the adoption"),
    }
}
