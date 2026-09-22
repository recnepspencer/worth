//! Exact World recovery for a branch adoption whose Relational effect already exists.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramAdoptionRecoveryFailure, WorthQueryApplicationRequestExt,
    WorthQueryBranchAdoptionPublicationOutcome, WorthQueryBranchAdoptionRecoveryOutcome,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionRecoveryDenial;

use crate::bounded_dimension_model::host::publish_on_first_program;
use crate::bounded_dimension_model::operator_identity::{authenticate_operator, request_scope};
use crate::bounded_dimension_model::presented_request::set_dimension;
use crate::bounded_dimension_model::programs::DimensionProgramP1;
use crate::bounded_dimension_model::readback::read_dimension;
use crate::bounded_dimension_model::settled_verdict::{settle, DimensionVerdict};

const P1_VALUE: u64 = 15;

#[test]
fn unpublished_adoption_settles_and_publishes_without_replaying_relational_work() {
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
    let unpublished = match prepared.publish() {
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => unpublished,
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_) => {
            panic!("the injected durability loss must retain exact custody")
        }
        WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect) => {
            panic!("owner settlement failure is not pre-effect no-effect: {no_effect:?}")
        }
    };
    assert!(unpublished.relational_requires_settlement());
    assert_eq!(unpublished.source(), requirements.source());
    assert_eq!(unpublished.target(), &target);

    let recovery = unpublished.into_recovery();
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .recover(recovery)
        .unwrap_or_else(|_| panic!("the exact branch must recover its adoption"));
    let performed = match outcome {
        WorthQueryBranchAdoptionRecoveryOutcome::Performed { adoption, cleanup } => {
            cleanup.expect("the superseded recovery record and owner cleanup must drain");
            adoption
        }
        WorthQueryBranchAdoptionRecoveryOutcome::NoEffect { no_effect, .. } => {
            panic!("settled adoption unexpectedly had no effect: {no_effect:?}")
        }
        WorthQueryBranchAdoptionRecoveryOutcome::ProductUnpublished { next, .. } => {
            panic!("settled adoption remained unpublished: {next:?}")
        }
    };
    assert_eq!(performed.source(), requirements.source());
    assert_eq!(performed.target(), &target);
    assert_eq!(
        performed.relational_owner_contacts(),
        0,
        "recovery adopts the settled result and never replays the Relational mutation"
    );

    let adopted = host.current_world();
    let p1 = host
        .supported_program::<DimensionProgramP1>()
        .expect("P1 remains rostered");
    assert_eq!(
        settle(set_dimension(&p1, adopted, P1_VALUE, 0x9175_2001)),
        DimensionVerdict::Performed(P1_VALUE)
    );
    assert_eq!(read_dimension(host.runtime(), adopted), P1_VALUE);
}

#[test]
fn sibling_branch_cannot_consume_an_unpublished_adoption_recovery() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let sibling = host
        .runtime()
        .branches()
        .fork(branch)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("sibling branch publishes");
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
        _ => panic!("the injected durability loss must retain exact custody"),
    };

    let failure = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(sibling)
        .programs()
        .recover(recovery)
        .err()
        .expect("a sibling cannot consume foreign adoption custody");
    let WorthQueryApplicationProgramAdoptionRecoveryFailure::Recovery(failure) = failure else {
        panic!("the sibling remains selectable, so execution must reject affinity")
    };
    assert!(matches!(
        failure.denial(),
        WorthQueryBranchAdoptionRecoveryDenial::ProductAffinityMismatch
    ));
    let recovery = failure.into_recovery();

    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .recover(recovery)
        .unwrap_or_else(|_| panic!("the owning branch must retain recovery authority"));
    match outcome {
        WorthQueryBranchAdoptionRecoveryOutcome::Performed { adoption, cleanup } => {
            cleanup.expect("recovery cleanup drains");
            assert_eq!(adoption.target(), &target);
            assert_eq!(adoption.relational_owner_contacts(), 0);
        }
        _ => panic!("the exact owning branch must complete recovery"),
    }

    assert_eq!(
        settle(set_dimension(&host, sibling, 3, 0x9175_2011)),
        DimensionVerdict::Performed(3),
        "the sibling remains on P0"
    );
}

#[test]
fn performed_recovery_retains_cleanup_failure_after_the_effect() {
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
        _ => panic!("the injected durability loss must retain exact custody"),
    };

    let outcome = host
        .runtime()
        .with_owner_cleanup_capacity_exhausted_for_test(|| {
            host.runtime()
                .request(&principal, &scope)
                .on_branch(branch)
                .programs()
                .recover(recovery)
                .unwrap_or_else(|_| panic!("cleanup capacity cannot erase the performed effect"))
        });
    let (adoption, cleanup_failure) = match outcome {
        WorthQueryBranchAdoptionRecoveryOutcome::Performed { adoption, cleanup } => (
            adoption,
            cleanup
                .err()
                .expect("the performed terminal must retain cleanup-capacity failure"),
        ),
        _ => panic!("cleanup failure occurs after the adoption has performed"),
    };
    assert_eq!(adoption.target(), &target);
    assert_eq!(adoption.relational_owner_contacts(), 0);
    let retained = cleanup_failure
        .into_recovery()
        .expect("capacity denial retains the exact prior recovery record");
    host.runtime()
        .release_product_publication_recovery(retained, 0)
        .expect("released cleanup capacity permits exact retry");
}

#[test]
fn two_same_head_adoptions_report_one_performed_world_transition() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let activation_entity = host
        .runtime()
        .program_activation_entity_for_test()
        .expect("the activation record is published once");
    let target = host
        .supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision()
        .clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let requirements = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .compare(&target)
        .expect("comparison succeeds");
    let prepare = || {
        host.runtime()
            .request(&principal, &scope)
            .on_branch(branch)
            .programs()
            .adopt(&target)
            .requirements(&requirements)
            .prepare(64)
            .expect("both attempts prepare against the same head")
    };
    let first = prepare();
    let second = prepare();
    let (first_start, first_ready) = std::sync::mpsc::sync_channel(1);
    let (second_start, second_ready) = std::sync::mpsc::sync_channel(1);

    let (first, second) = std::thread::scope(|threads| {
        let first = threads.spawn(move || {
            first_ready
                .recv_timeout(std::time::Duration::from_secs(10))
                .expect("the first prepared adoption receives its bounded start signal");
            first.publish()
        });
        let second = threads.spawn(move || {
            second_ready
                .recv_timeout(std::time::Duration::from_secs(10))
                .expect("the second prepared adoption receives its bounded start signal");
            second.publish()
        });
        first_start
            .send(())
            .expect("the first prepared adoption remains ready");
        second_start
            .send(())
            .expect("the second prepared adoption remains ready");
        (first.join().unwrap(), second.join().unwrap())
    });

    let mut performed = 0;
    let mut retained = 0;
    for outcome in [first, second] {
        match outcome {
            WorthQueryBranchAdoptionPublicationOutcome::Performed(adoption) => {
                performed += 1;
                assert_eq!(adoption.target(), &target);
            }
            WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => {
                retained += 1;
                assert_eq!(
                    unpublished.cause(),
                    worth_query_host::facade::runtime::ProductUnpublishedCause::ProductPublicationLost
                );
                host.runtime()
                    .release_branch_adoption_recovery(unpublished.into_recovery(), 0)
                    .expect("the losing exact owner result must release cleanly");
            }
            WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect) => {
                retained += 1;
                assert_eq!(
                    no_effect.cause(),
                    worth_query_host::facade::runtime::NoEffectCause::StaleExpectedProductHead
                );
            }
        }
    }
    assert_eq!(performed, 1);
    assert_eq!(retained, 1);
    assert_eq!(
        host.runtime().program_activation_entity_for_test(),
        Some(activation_entity),
        "the losing attempt cannot replace the once-published activation record"
    );

    let adopted = host.current_world();
    let p1 = host
        .supported_program::<DimensionProgramP1>()
        .expect("P1 remains rostered after the race");
    assert_eq!(
        settle(set_dimension(&p1, adopted, P1_VALUE, 0x9175_2021)),
        DimensionVerdict::Performed(P1_VALUE),
        "the next ordinary call must execute under the performed program"
    );
    assert_eq!(
        read_dimension(host.runtime(), host.current_world()),
        P1_VALUE,
        "the post-race World must expose one coherent component combination"
    );
}
