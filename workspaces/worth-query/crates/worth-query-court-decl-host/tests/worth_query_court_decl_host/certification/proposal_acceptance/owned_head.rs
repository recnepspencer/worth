//! Real head ownership composes ordinary candidate work with the existing guards.
use super::*;
use worth_query_host::facade::product::{
    WorthQueryApplicationProductBranchCleanupDenial, WorthQueryExpectedProductBranch,
    WorthQueryExpectedProductBranchAdoptionDenial, WorthQueryOwnedProductHeadCloseFailure,
    WorthQueryProductBranch, WorthQueryProductBranchOwnerCleanupDenial,
};
use worth_query_testkit_document_retention::facade::document_retention_model::host::{
    DocumentRetentionRuntime, DocumentRetentionSchema, RetentionProgramP0,
};
use worth_query_testkit_document_retention::facade::document_retention_model::readback::observe_head;

fn fork(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
    source: WorthQueryProductBranch,
) -> WorthQueryExpectedProductBranch<'_, DocumentRetentionSchema> {
    host.with_host_advancement(|phase| {
        let selected = host.runtime().on_branch(source).select(&phase).unwrap();
        host.runtime()
            .branches()
            .fork_expected(selected)
            .unwrap()
            .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
            .create(&phase)
            .unwrap()
    })
    .unwrap()
}

#[test]
fn owned_head_adopts_actual_candidate_and_preserves_independent_heads() {
    let host = publish_on_first_program();
    let root = host.current_world();
    let mut head = fork(&host, root).into_owned_head();
    let sibling = fork(&host, root).into_owned_head();
    let previous_selector = head.branch();
    let candidate = fork(&host, head.branch());
    let candidate_selector = candidate.branch();
    assert_eq!(
        settle(set_retention(&host, candidate_selector, 3, 0x9182_0101)),
        RetentionVerdict::Performed(3)
    );
    let expected = host
        .with_host_advancement(|phase| {
            host.runtime()
                .on_branch(candidate_selector)
                .select(&phase)
                .unwrap()
                .product()
                .selected_commit()
                .clone()
        })
        .unwrap();
    let previous = host
        .with_host_advancement(|phase| head.adopt(candidate, &phase).unwrap())
        .unwrap();
    assert_eq!(head.branch(), candidate_selector);
    assert_eq!(previous.branch(), previous_selector);
    assert_eq!(read_retention(host.runtime(), head.branch()), 3);
    assert_eq!(
        read_retention(host.runtime(), previous.branch()),
        SEED_RETENTION
    );
    assert_eq!(
        read_retention(host.runtime(), sibling.branch()),
        SEED_RETENTION
    );
    assert_eq!(read_retention(host.runtime(), root), SEED_RETENTION);
    host.with_host_advancement(|phase| {
        let selected = host
            .runtime()
            .on_branch(head.branch())
            .select(&phase)
            .unwrap();
        assert_eq!(selected.product().selected_commit(), &expected);
    })
    .unwrap();
    host.with_host_advancement(|phase| previous.close(&phase).unwrap())
        .unwrap();
    assert_eq!(read_retention(host.runtime(), head.branch()), 3);
    let retained = observe_head(host.runtime(), head.branch());
    let retired_selector = head.branch();
    let failure = host
        .with_host_advancement(|phase| head.close(&phase).unwrap_err())
        .unwrap();
    let WorthQueryOwnedProductHeadCloseFailure::Retired { branch, failure } = failure else {
        panic!("a held reader retains cleanup, not a live head");
    };
    assert_eq!(branch, retired_selector);
    assert!(matches!(
        failure.denial(),
        WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        )
    ));
    assert!(!host.runtime().branches().pending_cleanup().is_empty());
    let still_blocked = failure.into_cleanup().retry().unwrap_err();
    assert!(matches!(
        still_blocked.denial(),
        WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        )
    ));
    drop(retained);
    still_blocked.into_cleanup().retry().unwrap();
    host.with_host_advancement(|phase| sibling.close(&phase).unwrap())
        .unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn owned_head_wrong_source_refusal_preserves_head_and_original_candidate() {
    let host = publish_on_first_program();
    let mut head = fork(&host, host.current_world()).into_owned_head();
    let sibling = fork(&host, host.current_world()).into_owned_head();
    let original = head.branch();
    let candidate = fork(&host, sibling.branch());
    let candidate_selector = candidate.branch();
    assert_eq!(
        settle(set_retention(&host, candidate_selector, 3, 0x9182_0102)),
        RetentionVerdict::Performed(3)
    );
    let failure = host
        .with_host_advancement(|phase| head.adopt(candidate, &phase).unwrap_err())
        .unwrap();
    assert!(matches!(
        failure.denial(),
        WorthQueryExpectedProductBranchAdoptionDenial::HeadReplaced
    ));
    let (candidate, _) = failure.into_parts();
    assert_eq!(head.branch(), original);
    assert_eq!(candidate.branch(), candidate_selector);
    assert_eq!(read_retention(host.runtime(), original), SEED_RETENTION);
    assert_eq!(
        read_retention(host.runtime(), sibling.branch()),
        SEED_RETENTION
    );
    assert_eq!(read_retention(host.runtime(), candidate_selector), 3);
    host.with_host_advancement(|phase| candidate.close(&phase).unwrap())
        .unwrap();
    host.with_host_advancement(|phase| head.close(&phase).unwrap())
        .unwrap();
    host.with_host_advancement(|phase| sibling.close(&phase).unwrap())
        .unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn owned_head_foreign_close_preserves_live_custody_for_its_actual_runtime() {
    let host = publish_on_first_program();
    let foreign = publish_on_first_program();
    let head = fork(&host, host.current_world()).into_owned_head();
    let original = head.branch();
    let failure = foreign
        .with_host_advancement(|phase| head.close(&phase).unwrap_err())
        .unwrap();
    let WorthQueryOwnedProductHeadCloseFailure::Retained { head, denial } = failure else {
        panic!("a foreign phase cannot retire the actual head");
    };
    assert!(matches!(
        denial,
        WorthQueryApplicationProductBranchCloseDenial::ForeignPhase
    ));
    assert_eq!(head.branch(), original);
    assert_eq!(
        read_retention(host.runtime(), head.branch()),
        SEED_RETENTION
    );
    host.with_host_advancement(|phase| head.close(&phase).unwrap())
        .unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
    assert_eq!(
        read_retention(foreign.runtime(), foreign.current_world()),
        SEED_RETENTION
    );
}
