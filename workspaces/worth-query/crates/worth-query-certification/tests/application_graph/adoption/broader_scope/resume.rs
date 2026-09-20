use std::num::NonZeroUsize;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryBranchSetAdoptionProgress,
    WorthQueryBranchSetAdoptionResumeDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial;

use super::{fork, target_revision, P0_ONLY_DIMENSION, P1_ONLY_DIMENSION};
use crate::bounded_dimension_model::host::{publish_on_first_program, SEED_DIMENSION};
use crate::bounded_dimension_model::operator_identity::{authenticate_operator, request_scope};
use crate::bounded_dimension_model::presented_request::set_dimension;
use crate::bounded_dimension_model::programs::DimensionProgramP1;
use crate::bounded_dimension_model::settled_verdict::{settle, DimensionVerdict};

#[test]
fn stale_suffix_resumes_only_after_exact_fresh_coverage_and_preflight() {
    let host = publish_on_first_program();
    let a = host.current_world();
    let b = fork(&host, a);
    let c = fork(&host, a);
    let initial = coverage(&host, &[a, b, c]);
    let target = target_revision(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let mut adoption = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(initial, &[a, b, c])
        .unwrap()
        .programs()
        .adopt(&target)
        .prepare(64)
        .expect("the initial target set preflights");
    let d = fork(&host, a);
    assert_eq!(
        settle(set_dimension(&host, b, SEED_DIMENSION + 1, 0x9175_4060)),
        DimensionVerdict::Performed(SEED_DIMENSION + 1)
    );
    assert!(matches!(
        adoption.advance().unwrap().unwrap(),
        WorthQueryBranchSetAdoptionProgress::Performed { branch, .. } if *branch == a
    ));
    assert!(matches!(
        adoption.advance().unwrap().unwrap(),
        WorthQueryBranchSetAdoptionProgress::NoEffect { branch, .. } if *branch == b
    ));
    let stopped = adoption
        .begin_resume()
        .unwrap_or_else(|_| panic!("a stale stop must release its obsolete prepared suffix"));
    assert_eq!(stopped.remaining_branches(), &[b, c]);

    let widened = coverage(&host, &[b, c, d]);
    let failure = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(widened, &[b, c, d])
        .unwrap()
        .programs()
        .resume(stopped, 64)
        .err()
        .expect("fresh coverage cannot add the newly created branch");
    assert!(matches!(
        failure.denial(),
        WorthQueryBranchSetAdoptionResumeDenial::CoverageMismatch
    ));
    let stopped = failure.into_adoption();

    let exact = coverage(&host, &[b, c]);
    let resumed = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(exact, &[b, c])
        .unwrap()
        .programs()
        .resume(stopped, 64);
    let mut resumed = match resumed {
        Ok(resumed) => resumed,
        Err(_) => panic!("the exact remaining branches must freshly preflight"),
    };
    assert_eq!(resumed.progress().len(), 1);
    for expected in [b, c] {
        assert!(matches!(
            resumed.advance().unwrap().unwrap(),
            WorthQueryBranchSetAdoptionProgress::Performed { branch, .. } if *branch == expected
        ));
    }
    let closed = resumed
        .close()
        .unwrap_or_else(|_| panic!("the freshly prepared suffix must close"));
    assert_eq!(closed.progress().len(), 3);

    let p1 = host.supported_program::<DimensionProgramP1>().unwrap();
    for (ordinal, branch) in [a, b, c].into_iter().enumerate() {
        assert_eq!(
            settle(set_dimension(
                &p1,
                branch,
                P1_ONLY_DIMENSION,
                0x9175_4061 + ordinal as u64
            )),
            DimensionVerdict::Performed(P1_ONLY_DIMENSION)
        );
    }
    assert_eq!(
        settle(set_dimension(&host, d, P0_ONLY_DIMENSION, 0x9175_4064)),
        DimensionVerdict::Performed(P0_ONLY_DIMENSION),
        "fresh coverage cannot absorb a branch created outside the original operation"
    );
}

#[test]
fn exhausted_preflight_releases_reservations_and_cancel_before_effect_is_exact() {
    let host = publish_on_first_program();
    let a = host.current_world();
    let b = fork(&host, a);
    let c = fork(&host, a);
    let target = target_revision(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let denial = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage(&host, &[a, b, c]), &[a, b, c])
        .unwrap()
        .programs()
        .adopt(&target)
        .prepare(0)
        .err()
        .expect("zero selection work must deny before effects");
    assert!(matches!(
        denial,
        worth_query_host::facade::application_entry::WorthQueryBranchSetAdoptionPreparationDenial::Adoption {
            denial: WorthQueryBranchAdoptionPreparationDenial::SelectionLimitExceeded { .. },
            ..
        }
    ));

    let prepared = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage(&host, &[a, b, c]), &[a, b, c])
        .unwrap()
        .programs()
        .adopt(&target)
        .prepare(64)
        .expect("failed preflight must release every temporary reservation");
    let cancelled = prepared
        .cancel()
        .unwrap_or_else(|_| panic!("cancellation before effects cannot retain recovery custody"));
    assert_eq!(cancelled.cancelled_branch_count(), 3);
    assert!(cancelled.progress().is_empty());

    for (ordinal, branch) in [a, b, c].into_iter().enumerate() {
        assert_eq!(
            settle(set_dimension(
                &host,
                branch,
                P0_ONLY_DIMENSION,
                0x9175_4070 + ordinal as u64
            )),
            DimensionVerdict::Performed(P0_ONLY_DIMENSION),
            "preflight and cancellation cannot activate the target"
        );
    }
}

#[test]
fn cancellation_after_fresh_resume_counts_each_unperformed_branch_once() {
    let host = publish_on_first_program();
    let a = host.current_world();
    let b = fork(&host, a);
    let c = fork(&host, a);
    let target = target_revision(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let mut adoption = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage(&host, &[a, b, c]), &[a, b, c])
        .unwrap()
        .programs()
        .adopt(&target)
        .prepare(64)
        .expect("the initial target set preflights");
    assert_eq!(
        settle(set_dimension(&host, b, SEED_DIMENSION + 1, 0x9175_4080)),
        DimensionVerdict::Performed(SEED_DIMENSION + 1)
    );
    assert!(matches!(
        adoption.advance().unwrap().unwrap(),
        WorthQueryBranchSetAdoptionProgress::Performed { branch, .. } if *branch == a
    ));
    assert!(matches!(
        adoption.advance().unwrap().unwrap(),
        WorthQueryBranchSetAdoptionProgress::NoEffect { branch, .. } if *branch == b
    ));
    let stopped = adoption
        .begin_resume()
        .unwrap_or_else(|_| panic!("the stale suffix must become resumable"));
    let resumed = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage(&host, &[b, c]), &[b, c])
        .unwrap()
        .programs()
        .resume(stopped, 64)
        .unwrap_or_else(|_| panic!("the exact remaining set must freshly preflight"));

    let cancelled = resumed
        .cancel()
        .unwrap_or_else(|_| panic!("freshly prepared work has no unresolved custody"));
    assert_eq!(cancelled.cancelled_branch_count(), 2);
    assert_eq!(cancelled.progress().len(), 1);
}

#[test]
fn historical_no_effect_cannot_stop_a_freshly_resumed_suffix_again() {
    let host = publish_on_first_program();
    let a = host.current_world();
    let b = fork(&host, a);
    let c = fork(&host, a);
    let target = target_revision(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let mut adoption = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage(&host, &[a, b, c]), &[a, b, c])
        .unwrap()
        .programs()
        .adopt(&target)
        .prepare(64)
        .expect("the initial target set preflights");
    assert_eq!(
        settle(set_dimension(&host, b, SEED_DIMENSION + 1, 0x9175_4090)),
        DimensionVerdict::Performed(SEED_DIMENSION + 1)
    );
    assert!(matches!(
        adoption.advance().unwrap().unwrap(),
        WorthQueryBranchSetAdoptionProgress::Performed { branch, .. } if *branch == a
    ));
    assert!(matches!(
        adoption.advance().unwrap().unwrap(),
        WorthQueryBranchSetAdoptionProgress::NoEffect { branch, .. } if *branch == b
    ));
    let stopped = adoption
        .begin_resume()
        .unwrap_or_else(|_| panic!("the active stale stop must become resumable"));
    let resumed = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage(&host, &[b, c]), &[b, c])
        .unwrap()
        .programs()
        .resume(stopped, 64)
        .unwrap_or_else(|_| panic!("the exact remaining set must freshly preflight"));

    let mut resumed = match resumed.begin_resume() {
        Ok(_) => panic!("historical no-effect progress cannot reopen a resolved stop"),
        Err(resumed) => resumed,
    };
    assert_eq!(resumed.pending_branches().collect::<Vec<_>>(), [b, c]);
    assert!(matches!(
        resumed.advance().unwrap().unwrap(),
        WorthQueryBranchSetAdoptionProgress::Performed { branch, .. } if *branch == b
    ));
    assert!(matches!(
        resumed.advance().unwrap().unwrap(),
        WorthQueryBranchSetAdoptionProgress::Performed { branch, .. } if *branch == c
    ));
    let closed = resumed
        .close()
        .unwrap_or_else(|_| panic!("refusing a duplicate stop must preserve the fresh suffix"));
    assert_eq!(closed.progress().len(), 3);
}

#[test]
fn repeated_staleness_replaces_superseded_no_effect_history() {
    let host = publish_on_first_program();
    let a = host.current_world();
    let b = fork(&host, a);
    let c = fork(&host, a);
    let target = target_revision(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let mut adoption = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage(&host, &[a, b, c]), &[a, b, c])
        .unwrap()
        .programs()
        .adopt(&target)
        .prepare(64)
        .expect("the initial target set preflights");
    let initial_work = adoption.total_selection_work_units();

    assert_eq!(
        settle(set_dimension(&host, b, SEED_DIMENSION + 1, 0x9175_40a0)),
        DimensionVerdict::Performed(SEED_DIMENSION + 1)
    );
    assert!(matches!(
        adoption.advance().unwrap().unwrap(),
        WorthQueryBranchSetAdoptionProgress::Performed { branch, .. } if *branch == a
    ));
    assert!(matches!(
        adoption.advance().unwrap().unwrap(),
        WorthQueryBranchSetAdoptionProgress::NoEffect { branch, .. } if *branch == b
    ));

    let stopped = adoption
        .begin_resume()
        .unwrap_or_else(|_| panic!("the first stale disposition must release its prepared suffix"));
    assert_eq!(stopped.progress().len(), 2);
    let mut resumed = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage(&host, &[b, c]), &[b, c])
        .unwrap()
        .programs()
        .resume(stopped, 64)
        .unwrap_or_else(|_| panic!("the first fresh suffix preflights"));
    assert_eq!(resumed.progress().len(), 1);
    let first_resumed_work = resumed.total_selection_work_units();
    assert!(first_resumed_work > initial_work);

    assert_eq!(
        settle(set_dimension(&host, b, SEED_DIMENSION + 2, 0x9175_40a1)),
        DimensionVerdict::Performed(SEED_DIMENSION + 2)
    );
    assert!(matches!(
        resumed.advance().unwrap().unwrap(),
        WorthQueryBranchSetAdoptionProgress::NoEffect { branch, .. } if *branch == b
    ));
    let stopped = resumed.begin_resume().unwrap_or_else(|_| {
        panic!("the second stale disposition must release its prepared suffix")
    });
    assert_eq!(stopped.progress().len(), 2);
    let mut resumed = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage(&host, &[b, c]), &[b, c])
        .unwrap()
        .programs()
        .resume(stopped, 64)
        .unwrap_or_else(|_| panic!("the second fresh suffix preflights"));
    assert_eq!(resumed.progress().len(), 1);
    assert!(resumed.total_selection_work_units() > first_resumed_work);

    for expected in [b, c] {
        assert!(matches!(
            resumed.advance().unwrap().unwrap(),
            WorthQueryBranchSetAdoptionProgress::Performed { branch, .. } if *branch == expected
        ));
    }
    let closed = resumed
        .close()
        .unwrap_or_else(|_| panic!("the repeatedly refreshed suffix must close"));
    assert_eq!(closed.progress().len(), 3);
}

fn coverage(
    host: &crate::bounded_dimension_model::host::BoundedDimensionRuntime<
        crate::bounded_dimension_model::programs::DimensionProgramP0,
    >,
    branches: &[worth_query_host::facade::product::WorthQueryProductBranch],
) -> worth_query_host::facade::application_entry::WorthQueryProgramAdoptionCoverage {
    host.runtime()
        .branches()
        .program_adoption_coverage(branches, NonZeroUsize::new(branches.len()).unwrap())
        .unwrap()
}
