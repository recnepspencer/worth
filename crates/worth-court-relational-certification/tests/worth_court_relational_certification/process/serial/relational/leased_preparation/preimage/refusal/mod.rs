// A fully materialized proposal still belongs to its original current branch.
pub(crate) use {worth_execution::CancellationToken, crate::leased_preparation::isolated_context::preimage::source::Dependencies, crate::leased_preparation::isolated_context::preimage::source::Source, worth_relational::facade::transactions::TransactionCommitError, crate::leased_preparation::ordinary_context::authority, crate::leased_preparation::isolated_context::preimage::check_snapshot, crate::leased_preparation::ordinary_context::lease_request, crate::leased_preparation::ordinary_context::serial, crate::leased_preparation::isolated_context::preimage::source, crate::leased_preparation::ordinary_context::support};
pub(crate) use {worth_relational::facade::transactions::CommitConflict, worth_relational::facade::transactions::ConflictClass};

#[test]
fn preimage_fallback_proposal_refuses_intervening_commit_and_retries() {
    if worth_testkit_relational::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = serial();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let seed_execution = worth_execution::ExecutionRequest::serial(&host);
    let source = crate::leased_preparation::isolated_context::preimage::source::Source::seed(&seed_execution, 9, 1);
    let lease = authority()
        .request_lease(lease_request(32 * 1024 * 1024, worth_execution::CancellationToken::new()))
        .unwrap();
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let proposal = source
        .transaction(&execution, crate::leased_preparation::isolated_context::preimage::source::Dependencies::Endpoints)
        .validate(&execution, &source.runtime)
        .unwrap();
    let (intervening, _) = crate::leased_preparation::ordinary_context::support::create_entity(&execution, &source.runtime, "intervening");
    let before_refusal = source
        .runtime
        .observe_branch(&source.runtime.main_branch_identity())
        .unwrap()
        .0;
    let denial = source
        .runtime
        .prepare_validated_proposal(&execution, proposal)
        .unwrap_err();
    assert!(
        matches!(
            denial,
            worth_relational::facade::transactions::TransactionCommitError::Conflict {
                error: CommitConflict {
                    class: worth_relational::facade::transactions::ConflictClass::StaleValidationBasis { .. },
                    ..
                },
                ..
            }
        ),
        "{denial:?}"
    );
    assert_eq!(
        source
            .runtime
            .observe_branch(&source.runtime.main_branch_identity())
            .unwrap()
            .0,
        before_refusal
    );
    check_snapshot(&source, &execution, &intervening.snapshot, "original", 0);
    let retried = source
        .transaction(&execution, crate::leased_preparation::isolated_context::preimage::source::Dependencies::Endpoints)
        .commit(&execution, &source.runtime)
        .unwrap();
    assert_eq!(retried.commit.parents, vec![intervening.commit.commit_id]);
    check_snapshot(&source, &execution, &retried.snapshot, "published", 8);
    check_snapshot(
        &source,
        &execution,
        &source.original.snapshot,
        "original",
        0,
    );
    for snapshot in [
        &retried.snapshot,
        &intervening.snapshot,
        &source.original.snapshot,
    ] {
        source
            .runtime
            .snapshots()
            .release_snapshot(snapshot)
            .unwrap();
    }
}
