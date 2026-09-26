//! Certification that a branch's decision reads stay rooted at that branch.
//!
//! One relational runtime holds every branch, and commit versions are shared
//! across them. A fork that commits after its parent therefore reads at a
//! version newer than the parent's later writes, and a version-only read would
//! observe records the fork never had.

use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;
use worth_query_host::facade::product::WorthQueryProductBranch;

use super::bounded_dimension_model::{
    host::{publish_workflow_on_first_program, BoundedDimensionWorkflowRuntime, SEED_DIMENSION},
    presented_request::set_dimension,
    settled_verdict::{settle, DimensionVerdict},
    workflow::{link_review_requirement_on, unlink_review_requirement_on, ReviewRequirementDenial},
};

#[test]
fn a_fork_does_not_observe_a_relation_its_parent_removed_after_the_fork() {
    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    assert!(matches!(
        link_review_requirement_on(&application, main, 93_000).expect("main links"),
        WorthQueryApplicationMutationOutcome::Committed { .. },
    ));
    let fork = fork_of(&application, main);
    assert!(matches!(
        unlink_review_requirement_on(&application, main, 93_001).expect("main unlinks"),
        WorthQueryApplicationMutationOutcome::Committed { .. },
    ));
    advance_past_sibling(&application, fork, 93_002);

    let outcome = unlink_review_requirement_on(&application, fork, 93_003).expect("fork decides");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "the fork still holds the relation it was forked with: {outcome:?}",
    );
}

#[test]
fn a_fork_does_not_observe_a_relation_its_parent_added_after_the_fork() {
    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    let fork = fork_of(&application, main);
    assert!(matches!(
        link_review_requirement_on(&application, main, 93_100).expect("main links"),
        WorthQueryApplicationMutationOutcome::Committed { .. },
    ));
    advance_past_sibling(&application, fork, 93_101);

    let outcome = unlink_review_requirement_on(&application, fork, 93_102).expect("fork decides");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::DomainDenied(
                ReviewRequirementDenial::RelationMismatch
            ),
        ),
        "the fork never held the parent's later relation: {outcome:?}",
    );
}

#[test]
fn a_parent_does_not_observe_a_relation_its_fork_added() {
    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    let fork = fork_of(&application, main);
    assert!(matches!(
        link_review_requirement_on(&application, fork, 93_200).expect("fork links"),
        WorthQueryApplicationMutationOutcome::Committed { .. },
    ));
    advance_past_sibling(&application, main, 93_201);

    let outcome = unlink_review_requirement_on(&application, main, 93_202).expect("main decides");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::DomainDenied(
                ReviewRequirementDenial::RelationMismatch
            ),
        ),
        "the parent never held its fork's relation: {outcome:?}",
    );
}

fn fork_of(
    application: &BoundedDimensionWorkflowRuntime,
    branch: WorthQueryProductBranch,
) -> WorthQueryProductBranch {
    application
        .runtime()
        .branches()
        .fork(branch)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the fork publishes")
}

/// Commits an unrelated edit so the branch reads at a version newer than every
/// write its sibling has made.
fn advance_past_sibling(
    application: &BoundedDimensionWorkflowRuntime,
    branch: WorthQueryProductBranch,
    idempotency: u64,
) {
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            branch,
            SEED_DIMENSION + 1,
            idempotency,
        )),
        DimensionVerdict::Performed(SEED_DIMENSION + 1),
    );
}
