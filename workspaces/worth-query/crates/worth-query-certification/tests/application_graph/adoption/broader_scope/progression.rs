use std::num::NonZeroUsize;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryBranchSetAdoptionAdvanceDenial,
    WorthQueryBranchSetAdoptionProgress,
};
use worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial;

use super::{fork, target_revision, P0_ONLY_DIMENSION, P1_ONLY_DIMENSION};
use crate::bounded_dimension_model::host::{publish_on_first_program, SEED_DIMENSION};
use crate::bounded_dimension_model::operator_identity::{authenticate_operator, request_scope};
use crate::bounded_dimension_model::presented_request::set_dimension;
use crate::bounded_dimension_model::programs::DimensionProgramP1;
use crate::bounded_dimension_model::readback::observe_head;
use crate::bounded_dimension_model::settled_verdict::{settle, DimensionVerdict};

#[test]
fn one_incompatible_target_denies_preflight_before_any_branch_effect() {
    let host = publish_on_first_program();
    let a = host.current_world();
    let b = fork(&host, a);
    let c = fork(&host, a);
    assert_eq!(
        settle(set_dimension(&host, c, P0_ONLY_DIMENSION, 0x9175_4020)),
        DimensionVerdict::Performed(P0_ONLY_DIMENSION)
    );
    let before_a = observe_head(host.runtime(), a);
    let before_b = observe_head(host.runtime(), b);
    let coverage = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[a, b, c], NonZeroUsize::new(3).unwrap())
        .unwrap();
    let target = target_revision(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let denial = match host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage, &[a, b, c])
        .unwrap()
        .programs()
        .adopt(&target)
        .prepare(64)
    {
        Ok(_) => panic!("C is incompatible with P1"),
        Err(denial) => denial,
    };

    assert_eq!(denial.branch(), Some(c));
    assert!(matches!(
        denial,
        worth_query_host::facade::application_entry::WorthQueryBranchSetAdoptionPreparationDenial::Adoption {
            denial: WorthQueryBranchAdoptionPreparationDenial::TargetRuleRejected { .. },
            ..
        }
    ));
    assert_eq!(
        observe_head(host.runtime(), a).selected_commit(),
        before_a.selected_commit()
    );
    assert_eq!(
        observe_head(host.runtime(), b).selected_commit(),
        before_b.selected_commit()
    );
}

#[test]
fn stale_middle_branch_stops_progress_and_cancellation_preserves_the_prefix() {
    let host = publish_on_first_program();
    let a = host.current_world();
    let b = fork(&host, a);
    let c = fork(&host, a);
    let coverage = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[a, b, c], NonZeroUsize::new(3).unwrap())
        .unwrap();
    let target = target_revision(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let mut adoption = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage, &[a, b, c])
        .unwrap()
        .programs()
        .adopt(&target)
        .prepare(64)
        .expect("all three branches initially preflight");
    assert_eq!(
        settle(set_dimension(&host, b, SEED_DIMENSION + 1, 0x9175_4030)),
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
    assert_eq!(adoption.pending_branches().collect::<Vec<_>>(), [c]);
    assert!(matches!(
        adoption.advance(),
        Err(WorthQueryBranchSetAdoptionAdvanceDenial::ResolutionRequired { branch }) if branch == b
    ));
    let cancelled = match adoption.cancel() {
        Ok(cancelled) => cancelled,
        Err(_) => panic!("stale no-effect has no owner custody to strand"),
    };
    assert_eq!(cancelled.cancelled_pending_branch_count(), 1);
    assert_eq!(cancelled.progress().len(), 2);

    let p1 = host.supported_program::<DimensionProgramP1>().unwrap();
    assert_eq!(
        settle(set_dimension(&p1, a, P1_ONLY_DIMENSION, 0x9175_4031)),
        DimensionVerdict::Performed(P1_ONLY_DIMENSION)
    );
    assert_eq!(
        settle(set_dimension(&host, c, P0_ONLY_DIMENSION, 0x9175_4032)),
        DimensionVerdict::Performed(P0_ONLY_DIMENSION),
        "cancellation cannot relabel the unstarted suffix"
    );
}
