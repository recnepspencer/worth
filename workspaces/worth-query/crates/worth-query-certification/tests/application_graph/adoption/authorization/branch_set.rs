//! Fresh branch-local admission preserves completed prefix and pending custody.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::{
    WorthQueryBranchSetAdoptionAdvanceDenial, WorthQueryBranchSetAdoptionPreparationDenial,
    WorthQueryBranchSetAdoptionProgress,
};
use worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial;
pub(super) fn fork(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
) -> worth_query_host::facade::product::WorthQueryProductBranch {
    host.with_host_advancement(|phase| {
        host.runtime()
            .branches()
            .fork(host.current_world())
            .components(|components| components.fork_relational().reuse_exact_signal_basis())
            .create(&phase)
            .unwrap()
    })
    .unwrap()
}
#[test]
fn authorized_branch_set_adopts_both_real_branches() {
    let host = publish_on_first_program();
    let a = host.current_world();
    let b = fork(&host);
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let coverage = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[a, b], NonZeroUsize::new(2).unwrap())
        .unwrap();
    let mut adoption = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage, &[a, b])
        .unwrap()
        .programs()
        .adopt(&target)
        .unwrap()
        .prepare(64)
        .unwrap();
    for branch in [a, b] {
        assert!(
            matches!(adoption.advance(host.runtime(), &principal, &scope).unwrap().unwrap(), WorthQueryBranchSetAdoptionProgress::Performed { branch: actual, .. } if *actual == branch)
        );
        assert_eq!(
            host.runtime()
                .request(&principal, &scope)
                .on_branch(branch)
                .programs()
                .inspect()
                .unwrap()
                .revision(),
            &target
        );
    }
    assert!(adoption.close().is_ok());
}
#[test]
fn unauthorized_branch_set_adopt_and_prepare_publish_nothing() {
    let host = publish_on_first_program();
    let a = host.current_world();
    let b = fork(&host);
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let coverage = || {
        host.runtime()
            .branches()
            .program_adoption_coverage(&[a, b], NonZeroUsize::new(2).unwrap())
            .unwrap()
    };
    let to_prepare = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage(), &[a, b])
        .unwrap()
        .programs()
        .adopt(&target)
        .unwrap();
    change_grant_status(&host, b, "revoked", 91764002);
    let before_a = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(a)
        .programs()
        .inspect()
        .unwrap();
    let before_b = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(b)
        .programs()
        .inspect()
        .unwrap();
    let at_adopt = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage(), &[a, b])
        .unwrap()
        .programs()
        .adopt(&target)
        .err()
        .unwrap();
    let at_prepare = to_prepare.prepare(64).err().unwrap();
    for denial in [at_adopt, at_prepare] {
        match denial {
            WorthQueryBranchSetAdoptionPreparationDenial::Adoption {
                branch,
                denial: WorthQueryBranchAdoptionPreparationDenial::Authorization(cause),
            } => {
                assert_eq!(branch, b);
                permission_denied(cause);
            }
            other => panic!("branch capability must refuse: {other:?}"),
        }
    }
    assert_eq!(
        host.runtime()
            .request(&principal, &scope)
            .on_branch(a)
            .programs()
            .inspect()
            .unwrap()
            .revision(),
        before_a.revision()
    );
    assert_eq!(
        host.runtime()
            .request(&principal, &scope)
            .on_branch(b)
            .programs()
            .inspect()
            .unwrap()
            .revision(),
        before_b.revision()
    );
    assert_eq!(read_retention(host.runtime(), a), 7);
    assert_eq!(read_retention(host.runtime(), b), 7);
}
#[test]
fn revocation_before_next_publication_preserves_prefix_and_exact_pending_branch() {
    let host = publish_on_first_program();
    let a = host.current_world();
    let b = fork(&host);
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let coverage = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[a, b], NonZeroUsize::new(2).unwrap())
        .unwrap();
    let mut adoption = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage, &[a, b])
        .unwrap()
        .programs()
        .adopt(&target)
        .unwrap()
        .prepare(64)
        .unwrap();
    assert!(
        matches!(adoption.advance(host.runtime(), &principal, &scope).unwrap().unwrap(), WorthQueryBranchSetAdoptionProgress::Performed { branch, .. } if *branch == a)
    );
    change_grant_status(&host, b, "revoked", 91764003);
    match adoption
        .advance(host.runtime(), &principal, &scope)
        .err()
        .unwrap()
    {
        WorthQueryBranchSetAdoptionAdvanceDenial::Authorization { branch, cause } => {
            assert_eq!(branch, b);
            permission_denied(cause);
        }
        other => panic!("fresh publication must refuse: {other:?}"),
    }
    assert_eq!(adoption.pending_branches().collect::<Vec<_>>(), [b]);
    assert_eq!(adoption.progress().len(), 1);
    assert_eq!(
        host.runtime()
            .request(&principal, &scope)
            .on_branch(a)
            .programs()
            .inspect()
            .unwrap()
            .revision(),
        &target
    );
    assert_eq!(
        host.runtime()
            .request(&principal, &scope)
            .on_branch(b)
            .programs()
            .inspect()
            .unwrap()
            .revision(),
        host.owned_revision()
    );
    assert_eq!(read_retention(host.runtime(), b), 7);
    assert!(adoption.cancel().is_ok());
}
