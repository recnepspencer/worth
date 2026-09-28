use std::num::NonZeroUsize;

use super::{fork, target_revision, P0_ONLY_RETENTION, P1_ONLY_RETENTION};
use crate::document_retention_model::host::publish_on_first_program;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use crate::document_retention_model::presented_request::set_retention;
use crate::document_retention_model::readback::read_retention;
use crate::document_retention_model::settled_verdict::{settle, RetentionVerdict};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryBranchSetAdoptionProgress,
    WorthQueryProgramAdoptionCoverageDenial,
};

#[test]
fn foreign_branch_occurrence_cannot_enter_owner_issued_coverage() {
    let host = publish_on_first_program();
    let foreign = publish_on_first_program();
    let foreign_branch = foreign.current_world();
    assert!(matches!(
        host.runtime().branches().program_adoption_coverage(
            &[foreign_branch],
            NonZeroUsize::new(1).unwrap()
        ),
        Err(WorthQueryProgramAdoptionCoverageDenial::ForeignOrRetiredTarget { branch })
            if branch == foreign_branch
    ));
}

#[test]
fn caller_order_cannot_duplicate_or_widen_owner_issued_coverage() {
    let host = publish_on_first_program();
    let a = host.current_world();
    let b = fork(&host, a);
    let duplicate = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[a, a], NonZeroUsize::new(2).unwrap());
    assert!(matches!(
        duplicate,
        Err(WorthQueryProgramAdoptionCoverageDenial::DuplicateTarget { branch }) if branch == a
    ));

    let coverage = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[a, b], NonZeroUsize::new(2).unwrap())
        .expect("the owner issues exact A/B coverage");
    let d = fork(&host, a);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    assert!(matches!(
        host.runtime()
            .request(&principal, &scope)
            .on_branches(coverage, &[a, d]),
        Err(WorthQueryProgramAdoptionCoverageDenial::OrderedTargetMismatch)
    ));
}

#[test]
fn covered_branches_advance_in_order_without_absorbing_a_later_branch() {
    let host = publish_on_first_program();
    let a = host.current_world();
    let b = fork(&host, a);
    let c = fork(&host, a);
    let coverage = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[a, b, c], NonZeroUsize::new(3).unwrap())
        .expect("the live owner issues exact A/B/C coverage");
    let d = fork(&host, a);
    let target = target_revision(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let mut adoption = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage, &[a, b, c])
        .expect("the order is an exact permutation of the coverage")
        .programs()
        .adopt(&target)
        .prepare(64)
        .expect("all covered branches preflight before the first effect");

    assert_eq!(adoption.pending_branches().collect::<Vec<_>>(), [a, b, c]);
    for expected in [a, b, c] {
        let progress = adoption
            .advance()
            .expect("performed progress does not block the next branch")
            .expect("one covered branch remains");
        assert_eq!(progress.branch(), expected);
        assert!(matches!(
            progress,
            WorthQueryBranchSetAdoptionProgress::Performed { .. }
        ));
    }
    assert!(adoption.advance().unwrap().is_none());
    let closed = match adoption.close() {
        Ok(closed) => closed,
        Err(_) => panic!("fully performed progress must close"),
    };
    assert_eq!(closed.progress().len(), 3);

    for (ordinal, branch) in [a, b, c].into_iter().enumerate() {
        assert_eq!(
            settle(set_retention(
                &host,
                branch,
                P1_ONLY_RETENTION,
                0x9175_4000 + ordinal as u64
            )),
            RetentionVerdict::Performed(P1_ONLY_RETENTION)
        );
    }
    assert_eq!(
        settle(set_retention(&host, d, P0_ONLY_RETENTION, 0x9175_4010)),
        RetentionVerdict::Performed(P0_ONLY_RETENTION),
        "D was created after coverage and must remain outside the operation"
    );
    assert_eq!(read_retention(host.runtime(), d), P0_ONLY_RETENTION);
}
