//! Completed custody follows checkpoint dispositions assigned by this fixture.
//! The commit receipt is real; checkpoint and claim stages are set directly.
use super::super::{
    BoundOutputSource, PreparedOutputRootKind, ReadyCompletion, SourceCustody,
    WorthQueryAcceptedOutputAuthority, WorthQueryCompletedOutputDemand, WorthQueryOutputCheckpoint,
    WorthQueryOutputClaimIdentity, WorthQueryOutputProgress,
};
use super::*;
use crate::equality_type_id::EqualityTypeId as TypeId;

fn completed_stage(claimed_with_ready: bool) {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let (scope, _) = super::source_custody::real_scope_and_occurrence();
    let row_key = key("completed-stage", 5, 1);
    let bound = BoundOutputSource {
        scope,
        identity: row_key.source.clone(),
    };
    let mut state = DemandRegistryState::default();
    state.source_custody.insert(
        commit.clone(),
        SourceCustody {
            cleanup: Default::default(),
            occurrence: receipt.product_branch().occurrence(),
            root_kind: PreparedOutputRootKind::Discovered(TypeId::of::<()>()),
            source: None,
            discovery: None,
            bound_sources: Some(vec![bound.clone()]),
            consumed_sources: vec![bound.identity],
            retired_sources: Vec::new(),
            retired: None,
            token_count: 0,
            completed: true,
        },
    );
    let output = if claimed_with_ready {
        let completion = ReadyCompletion::for_test(WorthQueryCompletedOutputDemand {
            authority: WorthQueryAcceptedOutputAuthority::Committed(receipt),
            readiness: crate::domain_computation::primary_graph::WorthQueryOutputReadinessDeliveryEvidence::without_execution(),
            resources: None,
        });
        let mut progress = WorthQueryOutputProgress::restored(completion);
        progress.claim(WorthQueryOutputClaimIdentity(1));
        progress
    } else {
        WorthQueryOutputProgress::stopped_without_checkpoint(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::NoEffect,
            "finished without an answer",
        ))
    };
    let mut row = record(occurrence(), DemandState::Output(output), 0);
    row.source_commits.push(commit.clone());
    state.records.insert(row_key.clone(), row);
    state.note_custody(&commit);
    state
        .prune_completed_custody(&mut super::super::lifecycle::release::CleanupWalk::new())
        .unwrap();
    assert_eq!(
        state.source_custody.contains_key(&commit),
        claimed_with_ready,
        "an active checkpoint owner retains custody; a finished unowned row does not"
    );
    assert_eq!(
        state.records[&row_key].source_commits.contains(&commit),
        claimed_with_ready
    );
    if claimed_with_ready {
        {
            let DemandState::Output(output) = &mut state.records.get_mut(&row_key).unwrap().state
            else {
                panic!("the actual checkpoint row is retained")
            };
            output.reopen();
            assert!(matches!(
                output.checkpoint,
                Some(WorthQueryOutputCheckpoint::Ready(_))
            ));
        }
        state.note_custody(&commit);
        state
            .prune_completed_custody(&mut super::super::lifecycle::release::CleanupWalk::new())
            .unwrap();
        assert!(
            state.source_custody.is_empty(),
            "the completed claim releases its custody"
        );
        assert!(state.records[&row_key].source_commits.is_empty());
    }
}
#[test]
fn a_stopped_row_without_ready_releases_completed_custody() {
    completed_stage(false);
}
#[test]
fn a_claimed_row_with_ready_keeps_completed_custody_until_its_claim_ends() {
    completed_stage(true);
}
