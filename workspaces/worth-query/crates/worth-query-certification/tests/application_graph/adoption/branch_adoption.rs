//! Public branch-adoption journeys across the host facade.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted;

use crate::document_retention_model::host::{publish_on_first_program, SEED_RETENTION};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use crate::document_retention_model::presented_request::set_retention;
use crate::document_retention_model::programs::RetentionProgramP1;
use crate::document_retention_model::readback::read_retention;
use crate::document_retention_model::settled_verdict::{settle, RetentionVerdict};

const P0_ONLY_RETENTION: u64 = 3;
const P1_ONLY_RETENTION: u64 = 15;

#[test]
fn one_branch_adopts_p1_while_its_sibling_keeps_running_p0() {
    let host = publish_on_first_program();
    let main = host.current_world();
    let activation = host
        .runtime()
        .program_activation_entity_for_test()
        .expect("the activation record is published once at bootstrap");
    let sibling = host
        .runtime()
        .branches()
        .fork(main)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the sibling branch must publish");
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(main)
        .programs();
    let requirements = programs
        .compare(&target)
        .expect("the host must describe P0 to P1 requirements");
    assert!(requirements.requires_existing_state_validation());
    let prepared = programs
        .adopt(&requirements)
        .prepare(64)
        .expect("the seed satisfies P1 and adoption must prepare");
    assert_eq!(
        prepared.selected_entity_count(),
        2,
        "adoption validates both seeded documents, including the related workflow subject"
    );
    let performed = match prepared.publish() {
        WorthQueryBranchAdoptionPublicationOutcome::ExecutionDenied(cause) => {
            panic!("the declared host policy admits adoption: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::Performed(performed) => performed,
        WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect) => {
            panic!("adoption unexpectedly had no effect: {no_effect:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => {
            panic!("adoption lost product publication: {unpublished:?}")
        }
    };
    assert_eq!(performed.target(), &target);
    assert_eq!(
        host.runtime().program_activation_entity_for_test(),
        Some(activation),
        "adoption updates the activation record; it never rebinds the activation cell"
    );

    let adopted = host.current_world();
    assert_eq!(
        settle(set_retention(
            &host,
            adopted,
            P0_ONLY_RETENTION,
            0x9175_1001
        )),
        RetentionVerdict::violated("document-retention-v2"),
        "P0's law no longer governs the adopted branch"
    );
    assert_eq!(
        settle(set_retention(
            &host,
            adopted,
            P1_ONLY_RETENTION,
            0x9175_1002
        )),
        RetentionVerdict::Performed(P1_ONLY_RETENTION)
    );
    assert_eq!(read_retention(host.runtime(), adopted), P1_ONLY_RETENTION);

    assert_eq!(
        settle(set_retention(
            &host,
            sibling,
            P0_ONLY_RETENTION,
            0x9175_1003
        )),
        RetentionVerdict::Performed(P0_ONLY_RETENTION),
        "the sibling keeps the source program"
    );
    assert_eq!(read_retention(host.runtime(), sibling), P0_ONLY_RETENTION);
}

#[test]
fn target_rule_rejects_existing_state_by_its_installed_identity() {
    let host = publish_on_first_program();
    let initial = host.current_world();
    assert_eq!(
        settle(set_retention(
            &host,
            initial,
            P0_ONLY_RETENTION,
            0x9175_1011
        )),
        RetentionVerdict::Performed(P0_ONLY_RETENTION)
    );
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
    let requirements = programs.compare(&target).expect("comparison must succeed");
    let denial = match programs.adopt(&requirements).prepare(64) {
        Ok(_) => panic!("P1 must judge and reject the existing P0-only value"),
        Err(denial) => denial,
    };
    let WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
        worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial::TargetRuleRejected {
            identity,
        },
    ) = denial
    else {
        panic!("unexpected adoption denial: {denial:?}");
    };
    assert_eq!(identity.rule_id.as_str(), "document-retention-v2");
    assert_eq!(read_retention(host.runtime(), branch), P0_ONLY_RETENTION);
}

#[test]
fn prepared_adoption_refuses_a_branch_head_that_moved() {
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
    let requirements = programs.compare(&target).expect("comparison must succeed");
    let prepared = programs
        .adopt(&requirements)
        .prepare(64)
        .expect("the adoption must prepare against the selected head");

    assert_eq!(
        settle(set_retention(
            &host,
            branch,
            SEED_RETENTION + 1,
            0x9175_1021
        )),
        RetentionVerdict::Performed(SEED_RETENTION + 1)
    );
    match prepared.publish() {
        WorthQueryBranchAdoptionPublicationOutcome::ExecutionDenied(cause) => {
            panic!("the declared host policy admits adoption: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect) => assert_eq!(
            no_effect.cause(),
            worth_query_host::facade::runtime::NoEffectCause::StaleExpectedProductHead
        ),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_) => {
            panic!("a prepared adoption cannot silently rebase")
        }
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => {
            panic!("stale publication must refuse before owner effects: {unpublished:?}")
        }
    }
}

#[test]
fn affected_state_selection_refuses_an_understated_ceiling() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let first_programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = first_programs
        .compare(&target)
        .expect("comparison must succeed");
    let prepared = first_programs
        .adopt(&requirements)
        .prepare(64)
        .expect("a sufficient ceiling must prepare");
    let exact_work = prepared.selection_work_units();
    assert!(exact_work > 0);
    drop(prepared);

    let denied_programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let denial = match denied_programs.adopt(&requirements).prepare(exact_work - 1) {
        Ok(_) => panic!("a ceiling below the measured cost cannot cover the live state"),
        Err(denial) => denial,
    };
    let WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
        worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial::SelectionLimitExceeded {
            maximum_work_units,
            consumed_work_units,
        },
    ) = denial
    else {
        panic!("unexpected selection denial: {denial:?}");
    };
    assert_eq!(maximum_work_units, exact_work - 1);
    assert_eq!(consumed_work_units, maximum_work_units);
    assert!(consumed_work_units < exact_work);

    let exact_programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let exact = exact_programs
        .adopt(&requirements)
        .prepare(exact_work)
        .expect("the exact measured ceiling must prepare");
    assert_eq!(exact.selection_work_units(), exact_work);
}

#[test]
fn a_p0_candidate_prepared_before_adoption_cannot_publish_after_p1_activates() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let pause = host
        .runtime()
        .pause_after_application_candidate_preparation_for_test(
            std::num::NonZeroUsize::new(1).unwrap(),
        );

    std::thread::scope(|threads| {
        let old_writer = threads.spawn(|| {
            set_retention(&host, branch, SEED_RETENTION + 1, 0x9175_1031)
                .expect("the old P0 request must reach publication")
        });
        assert!(pause.wait_until_reached(std::time::Duration::from_secs(10)));

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
        let requirements = programs.compare(&target).expect("comparison must succeed");
        let prepared = programs
            .adopt(&requirements)
            .prepare(64)
            .expect("adoption must prepare while the old candidate is parked");
        assert!(matches!(
            prepared.publish(),
            WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
        ));

        pause.release();
        let old_outcome = old_writer.join().expect("the old writer must settle");
        assert!(matches!(
            old_outcome,
            WorthQueryApplicationMutationOutcome::Commit(
                WorthQueryApplicationUncommitted::ProductStale(_)
            )
        ));
    });

    assert_eq!(
        read_retention(host.runtime(), host.current_world()),
        SEED_RETENTION,
        "the stale P0 candidate must not land after activation"
    );
}

#[test]
fn requirements_from_an_old_activation_cannot_authorize_a_later_adoption() {
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
    let old_requirements = programs.compare(&target).expect("comparison must succeed");
    let prepared = programs
        .adopt(&old_requirements)
        .prepare(64)
        .expect("the first adoption must prepare");
    assert!(matches!(
        prepared.publish(),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));

    let adopted = host.current_world();
    let denial = match host
        .runtime()
        .request(&principal, &scope)
        .on_branch(adopted)
        .programs()
        .adopt(&old_requirements)
        .prepare(64)
    {
        Ok(_) => panic!("requirements compiled under P0 cannot authorize a P1 occurrence"),
        Err(denial) => denial,
    };
    assert!(matches!(
        denial,
        WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
            worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial::RequirementsChanged
        )
    ));
}
