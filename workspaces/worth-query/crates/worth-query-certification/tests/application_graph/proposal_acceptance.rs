//! Exact public fork admission precedes ordinary candidate requests and cleanup.
#[path = "proposal_acceptance/adoption.rs"]
mod adoption;
use crate::document_retention_model::{
    host::{publish_on_first_program, SEED_RETENTION},
    presented_request::set_retention,
    readback::read_retention,
    settled_verdict::{settle, RetentionVerdict},
};
use worth_query_host::facade::{
    primary_graph::{RuntimeWorldBranchAdmissionDenial, RuntimeWorldServiceDenial},
    product::{
        WorthQueryApplicationProductBranchCloseDenial, WorthQueryProductBranchAdmissionDenial,
        WorthQueryProductBranchAdmissionRefusalKind, WorthQueryProductBranchCreateError,
        WorthQueryProductBranchCreationDenial,
    },
};

#[test]
fn expected_source_fork_retains_selection_and_runs_ordinary_candidate_requests() {
    let host = publish_on_first_program();
    let source = host.current_world();
    let selected = host
        .with_host_advancement(|phase| host.runtime().on_branch(source).select(&phase).unwrap())
        .unwrap();
    let expected_commit = selected.product().selected_commit().clone();
    let candidate = host
        .with_host_advancement(|phase| {
            host.runtime()
                .branches()
                .fork_expected(selected)
                .unwrap()
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    assert_ne!(candidate.branch(), source);
    assert_eq!(candidate.expected_source().product_branch(), source);
    assert_eq!(
        candidate.expected_source().selected_commit(),
        &expected_commit
    );
    assert_eq!(
        read_retention(host.runtime(), candidate.branch()),
        SEED_RETENTION
    );
    assert_eq!(
        settle(set_retention(&host, candidate.branch(), 3, 0x9181_1701)),
        RetentionVerdict::Performed(3),
    );
    assert_eq!(read_retention(host.runtime(), candidate.branch()), 3);
    assert_eq!(read_retention(host.runtime(), source), SEED_RETENTION);
    host.with_host_advancement(|phase| candidate.close(&phase).unwrap())
        .unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn expected_source_fork_refuses_movement_between_selection_and_creation() {
    let host = publish_on_first_program();
    let source = host.current_world();
    let selected = host
        .with_host_advancement(|phase| host.runtime().on_branch(source).select(&phase).unwrap())
        .unwrap();
    let fork = host
        .runtime()
        .branches()
        .fork_expected(selected)
        .unwrap()
        .components(|parts| parts.fork_relational().reuse_exact_signal_basis());
    assert_eq!(
        settle(set_retention(&host, source, 3, 0x9181_1702)),
        RetentionVerdict::Performed(3),
    );
    let outcome = host
        .with_host_advancement(|phase| fork.create(&phase))
        .unwrap();
    assert!(matches!(
        outcome,
        Err(WorthQueryProductBranchCreateError::Creation(
            WorthQueryProductBranchCreationDenial::World(RuntimeWorldServiceDenial::Denied(
                RuntimeWorldBranchAdmissionDenial::StaleSourceHead
            ))
        ))
    ));
    assert_eq!(read_retention(host.runtime(), source), 3);
    assert!(host.runtime().branches().pending_cleanup().is_empty());
    // A fresh selection succeeds at the moved source; the old selection was
    // refused for its actual expected basis, not an unusable component plan.
    let candidate = host
        .with_host_advancement(|phase| {
            let selected = host.runtime().on_branch(source).select(&phase).unwrap();
            host.runtime()
                .branches()
                .fork_expected(selected)
                .unwrap()
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    assert_eq!(read_retention(host.runtime(), candidate.branch()), 3);
    host.with_host_advancement(|phase| candidate.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn expected_source_fork_refuses_another_applications_selection() {
    let host = publish_on_first_program();
    let foreign = publish_on_first_program();
    let selected = foreign
        .with_host_advancement(|phase| {
            foreign
                .runtime()
                .on_branch(foreign.current_world())
                .select(&phase)
                .unwrap()
        })
        .unwrap();
    assert!(matches!(
        host.runtime().branches().fork_expected(selected),
        Err(WorthQueryProductBranchAdmissionDenial::Refused(
            WorthQueryProductBranchAdmissionRefusalKind::ForeignOwner
        ))
    ));
    assert_eq!(
        read_retention(host.runtime(), host.current_world()),
        SEED_RETENTION
    );
    assert!(host.runtime().branches().pending_cleanup().is_empty());
    assert!(foreign.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn expected_candidate_close_refusal_preserves_candidate_and_source_custody() {
    let host = publish_on_first_program();
    let foreign = publish_on_first_program();
    let candidate = host
        .with_host_advancement(|phase| {
            let selected = host
                .runtime()
                .on_branch(host.current_world())
                .select(&phase)
                .unwrap();
            host.runtime()
                .branches()
                .fork_expected(selected)
                .unwrap()
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let branch = candidate.branch();
    let expected = candidate.expected_source();
    let failure = foreign
        .with_host_advancement(|phase| match candidate.close(&phase) {
            Ok(_) => panic!("a foreign phase cannot close the candidate"),
            Err(failure) => failure,
        })
        .unwrap();
    assert!(matches!(
        failure.denial(),
        WorthQueryApplicationProductBranchCloseDenial::ForeignPhase
    ));
    let (candidate, _) = failure.into_parts();
    assert_eq!(candidate.branch(), branch);
    assert_eq!(candidate.expected_source(), expected);
    assert_eq!(
        read_retention(host.runtime(), candidate.branch()),
        SEED_RETENTION
    );
    host.with_host_advancement(|phase| candidate.close(&phase).unwrap())
        .unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}
