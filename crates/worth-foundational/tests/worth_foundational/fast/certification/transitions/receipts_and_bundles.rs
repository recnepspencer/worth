use worth_foundational::{
    FoundationalBoundaryArtifactCategory, FoundationalTransitionIssuanceCause,
};

use super::fixtures::receipt::{
    commit_id, committed_authority_artifact, discard_closeout_cause, discard_receipt,
    no_op_issuance_cause, receipt_authority, receipt_identity,
};

#[test]
fn receipt_issuance_preserves_blind_consumer_authority_and_strategy_meaning() {
    let receipt = committed_authority_artifact("mesh-update")
        .issue_receipt(receipt_identity(51), commit_id(41), receipt_authority())
        .expect("receipt should issue from committed authority");

    assert_eq!(receipt.commit_id().handle().get(), 41);
    assert_eq!(receipt.receipt_identity().handle().get(), 51);
    assert_eq!(receipt.branch_id().as_str(), "main");
    assert_eq!(receipt.parent_basis().basis_id().get(), 401);
    assert_eq!(receipt.delta_evidence().delta_count(), 2);
    assert_eq!(
        receipt.transition_class(),
        worth_foundational::FoundationalAuthorityTransitionClass::Commit
    );
    assert_eq!(
        receipt.strategy_identity().ownership(),
        worth_foundational::FoundationalTransitionStrategyOwnershipClass::CustomRegistered
    );
    assert_eq!(
        receipt.strategy_descriptor_digest().digest_id().bytes(),
        &[77; 32]
    );
    assert_eq!(receipt.transition_basis_identity().basis_id().get(), 73);
    assert_eq!(
        receipt.receipt_claim().category(),
        FoundationalBoundaryArtifactCategory::Receipt
    );
    assert_eq!(
        receipt.issuance_basis().issuance_cause(),
        FoundationalTransitionIssuanceCause::CommitAttested
    );
}

#[test]
fn provenance_rows_remain_structured_and_blind_consumer_readable() {
    let receipt = committed_authority_artifact("mesh-update")
        .issue_receipt(receipt_identity(52), commit_id(42), receipt_authority())
        .expect("receipt should issue");
    let row = &receipt.transition_provenance_rows()[0];

    assert_eq!(row.source_branch().as_str(), "feature/geometry");
    assert_eq!(row.target_branch().as_str(), "main");
    assert_eq!(row.parent_basis().basis_id().get(), 401);
    assert_eq!(row.merge_basis().identity().basis_id().get(), 73);
    assert_eq!(row.observation_basis().basis_id().get(), 31);
    assert_eq!(
        row.comparison_basis()
            .expect("comparison basis remains visible")
            .basis_id()
            .get(),
        43
    );
    assert_eq!(
        row.correspondence_basis()
            .expect("correspondence basis remains visible")
            .basis_id()
            .get(),
        67
    );
    assert_eq!(
        row.remap_basis()
            .expect("remap basis remains visible")
            .basis_id()
            .get(),
        71
    );
    assert_eq!(
        row.issuance_cause(),
        Some(FoundationalTransitionIssuanceCause::CommitAttested)
    );
    assert_eq!(
        row.commit_id()
            .expect("receipt rows carry attested commit id"),
        commit_id(42)
    );
    assert_eq!(
        row.receipt_identity()
            .expect("receipt rows carry attested receipt identity"),
        receipt_identity(52)
    );
}

#[test]
fn transition_bundle_emits_typed_summary_report_and_receipt_without_result_bags() {
    let bundle = committed_authority_artifact("mesh-update")
        .emit_transition_bundle()
        .with_summary()
        .with_merge_report()
        .with_receipt(receipt_identity(53), commit_id(43), receipt_authority())
        .materialize()
        .expect("bundle should materialize");

    assert_eq!(
        bundle.primary().transition_class(),
        bundle.transition_class()
    );
    assert!(bundle.summary().is_some());
    assert!(bundle.merge_report().is_some());
    assert!(bundle.receipt().is_some());
    assert_eq!(bundle.materialization_cost().member_count(), 4);
    assert_eq!(bundle.materialization_cost().provenance_row_count(), 2);
    assert_eq!(bundle.materialization_cost().attested_delta_count(), 2);
}

#[test]
fn report_only_bundle_does_not_fake_receipt_attestation_fields() {
    let bundle = committed_authority_artifact("mesh-update")
        .emit_transition_bundle()
        .with_merge_report()
        .materialize()
        .expect("report-only bundle should materialize");
    let row = &bundle
        .merge_report()
        .expect("report should exist")
        .surface()
        .rows()[0];

    assert_eq!(row.issuance_cause(), None);
    assert_eq!(row.commit_id(), None);
    assert_eq!(row.receipt_identity(), None);
}

#[test]
fn discard_receipts_remain_explicitly_non_authoritative() {
    let discard = discard_receipt("mesh-update");

    assert_eq!(discard.branch_id().as_str(), "feature/geometry");
    assert_eq!(discard.fork_basis().forked_from_branch().as_str(), "main");
    assert_eq!(discard.closeout_cause(), discard_closeout_cause());
    assert!(discard.non_authoritative_residue_report().is_zero_residue());
    assert_eq!(discard.summary().surface().supporting_point_count(), 3);
}

#[test]
fn no_op_issuance_cause_stays_distinct_from_authoritative_commit_attestation() {
    assert_eq!(
        no_op_issuance_cause(),
        FoundationalTransitionIssuanceCause::NoOpAttested
    );
    assert_ne!(
        no_op_issuance_cause(),
        FoundationalTransitionIssuanceCause::CommitAttested
    );
}
