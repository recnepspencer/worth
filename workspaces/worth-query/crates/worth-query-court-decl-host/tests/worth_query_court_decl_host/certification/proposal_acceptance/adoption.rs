//! Application head replacement consumes the retained exact-source continuation.
use super::*;
use worth_query_host::facade::product::WorthQueryExpectedProductBranchAdoptionDenial as Denial;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
};

#[test]
fn expected_candidate_adoption_replaces_the_actual_head_after_ordinary_requests() {
    let host = publish_on_first_program();
    let source = host.current_world();
    let mut head = source;
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
    let branch = candidate.branch();
    assert_eq!(
        settle(set_retention(&host, branch, 3, 0x9181_1710)),
        RetentionVerdict::Performed(3)
    );
    let previous = host
        .with_host_advancement(|phase| candidate.adopt(&mut head, &phase).unwrap())
        .unwrap();
    assert_eq!(head, branch);
    assert_eq!(previous, source);
    assert_eq!(read_retention(host.runtime(), head), 3);
    assert_eq!(read_retention(host.runtime(), previous), SEED_RETENTION);
    host.with_host_advancement(|phase| host.runtime().on_branch(previous).close(&phase).unwrap())
        .unwrap();
    assert_eq!(read_retention(host.runtime(), head), 3);
    host.with_host_advancement(|phase| host.runtime().on_branch(head).close(&phase).unwrap())
        .unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn expected_candidate_adoption_refuses_source_movement_and_same_payload_aba() {
    let host = publish_on_first_program();
    let source = host.current_world();
    let mut head = source;
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
    let branch = candidate.branch();
    assert_eq!(
        settle(set_retention(&host, branch, 4, 0x9181_1711)),
        RetentionVerdict::Performed(4)
    );
    assert_eq!(
        settle(set_retention(&host, source, 3, 0x9181_1712)),
        RetentionVerdict::Performed(3)
    );
    let failure = host
        .with_host_advancement(|phase| candidate.adopt(&mut head, &phase).unwrap_err())
        .unwrap();
    assert!(matches!(
        failure.denial(),
        Denial::SourceBasis(_) | Denial::HeadMoved
    ));
    assert_eq!(head, source);
    assert_eq!(read_retention(host.runtime(), head), 3);
    let (candidate, _) = failure.into_parts();
    assert_eq!(candidate.branch(), branch);
    assert_eq!(
        settle(set_retention(&host, source, SEED_RETENTION, 0x9181_1713)),
        RetentionVerdict::Performed(SEED_RETENTION)
    );
    let failure = host
        .with_host_advancement(|phase| candidate.adopt(&mut head, &phase).unwrap_err())
        .unwrap();
    assert!(matches!(
        failure.denial(),
        Denial::SourceBasis(_) | Denial::HeadMoved
    ));
    assert_eq!(head, source);
    assert_eq!(read_retention(host.runtime(), head), SEED_RETENTION);
    let (candidate, _) = failure.into_parts();
    assert_eq!(read_retention(host.runtime(), candidate.branch()), 4);
    host.with_host_advancement(|phase| candidate.close(&phase).unwrap())
        .unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn expected_candidate_adoption_refuses_a_replaced_model_slot_and_foreign_phase() {
    let host = publish_on_first_program();
    let foreign = publish_on_first_program();
    let source = host.current_world();
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
    let branch = candidate.branch();
    let mut head = branch;
    let failure = host
        .with_host_advancement(|phase| candidate.adopt(&mut head, &phase).unwrap_err())
        .unwrap();
    assert!(matches!(failure.denial(), Denial::HeadReplaced));
    assert_eq!(head, branch);
    let (candidate, _) = failure.into_parts();
    head = source;
    let failure = foreign
        .with_host_advancement(|phase| candidate.adopt(&mut head, &phase).unwrap_err())
        .unwrap();
    assert!(matches!(failure.denial(), Denial::ForeignPhase));
    assert_eq!(head, source);
    let (candidate, _) = failure.into_parts();
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let mut candidate = Some(candidate);
    let stopped = host
        .runtime()
        .with_application_advancement(&scope, |phase| {
            cancellation.cancel();
            let failure = candidate
                .take()
                .unwrap()
                .adopt(&mut head, &phase)
                .unwrap_err();
            assert!(matches!(
                failure.denial(),
                Denial::Stopped(WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                ))
            ));
            assert_eq!(head, source);
            candidate = Some(failure.into_parts().0);
        });
    stopped.expect("the callback handled the typed adoption interruption");
    let previous = host
        .with_host_advancement(|phase| candidate.unwrap().adopt(&mut head, &phase).unwrap())
        .unwrap();
    assert_eq!(previous, source);
    assert_eq!(head, branch);
    host.with_host_advancement(|phase| host.runtime().on_branch(head).close(&phase).unwrap())
        .unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn expected_candidate_adoption_refuses_a_retired_candidate_without_moving_the_head() {
    let host = publish_on_first_program();
    let source = host.current_world();
    let mut head = source;
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
    host.with_host_advancement(|phase| {
        host.runtime()
            .on_branch(candidate.branch())
            .close(&phase)
            .unwrap()
    })
    .unwrap();
    let failure = host
        .with_host_advancement(|phase| candidate.adopt(&mut head, &phase).unwrap_err())
        .unwrap();
    assert!(matches!(
        failure.denial(),
        Denial::CandidateAdmission(WorthQueryProductBranchAdmissionDenial::Refused(
            WorthQueryProductBranchAdmissionRefusalKind::RetiredBranch
        ))
    ));
    assert_eq!(head, source);
    assert_eq!(read_retention(host.runtime(), head), SEED_RETENTION);
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}
