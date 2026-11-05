//! Completed custody follows checkpoint dispositions assigned by this fixture.
//! The commit receipt is real; checkpoint and claim stages are set directly.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_custody::PreparedOutputRootKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::ReadyCompletion;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::SourceCustody;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryAcceptedOutputAuthority;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryCompletedOutputDemand;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputClaimIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;

pub fn completed_stage(claimed_with_ready: bool) {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let (scope, _) = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::source_custody::real_scope_and_occurrence();
    let row_key = key("completed-stage", 5, 1);
    let bound = BoundOutputSource::new_for_test(scope, (*row_key.source_for_test()).clone());
    let mut state = DemandRegistryState::default();
    state.source_custody.insert(commit.clone(), {
        // Bound in the original literal order, passed in the door's.
        let (
            cleanup,
            occurrence,
            root_kind,
            source,
            discovery,
            bound_sources,
            consumed_sources,
            retired_sources,
            retired,
            token_count,
            completed,
        ) = (
            Default::default(),
            receipt.product_branch().occurrence_for_test(),
            PreparedOutputRootKind::Discovered(TypeId::of_for_test::<()>()),
            None,
            None,
            Some(vec![bound.clone()]),
            vec![bound.identity_into_for_test()],
            Vec::new(),
            None,
            0,
            true,
        );
        SourceCustody::new_for_test(
            occurrence,
            root_kind,
            source,
            discovery,
            consumed_sources,
            retired_sources,
            bound_sources,
            retired,
            token_count,
            cleanup,
            completed,
        )
    });
    let output = if claimed_with_ready {
        let completion = ReadyCompletion::for_test(WorthQueryCompletedOutputDemand::new_for_test(WorthQueryAcceptedOutputAuthority::Committed(receipt), worth_query_execution::facade::domain_computation::WorthQueryOutputReadinessDeliveryEvidence::without_execution_for_test()));
        let mut progress = WorthQueryOutputProgress::restored_for_test(completion);
        progress.claim_for_test(WorthQueryOutputClaimIdentity(1));
        progress
    } else {
        WorthQueryOutputProgress::stopped_without_checkpoint_for_test(
            WorthQueryOutputDemandDenial::new_for_test(
                WorthQueryOutputDemandDenialKind::NoEffect,
                "finished without an answer",
            ),
        )
    };
    let mut row = record(occurrence(), DemandState::Output(output), 0);
    (*row.source_commits_mut_for_test()).push(commit.clone());
    (*state.records_mut_for_test()).insert_for_test(row_key.clone(), row);
    state.note_custody_for_test(&commit);
    state.prune_completed_custody().unwrap();
    assert_eq!(
        state.source_custody.contains_key(&commit),
        claimed_with_ready,
        "an active checkpoint owner retains custody; a finished unowned row does not"
    );
    assert_eq!(
        (*(*state.records_for_test())[&row_key].source_commits_for_test()).contains(&commit),
        claimed_with_ready
    );
    if claimed_with_ready {
        {
            let mut record = (*state.records_mut_for_test()).get_mut(&row_key).unwrap();
            let DemandState::Output(output) = &mut (*record.state_mut_for_test()) else {
                panic!("the actual checkpoint row is retained")
            };
            output.reopen_for_test();
            assert!(matches!(
                (*output.checkpoint_for_test()),
                Some(WorthQueryOutputCheckpoint::Ready(_))
            ));
        }
        state.note_custody_for_test(&commit);
        state.prune_completed_custody().unwrap();
        assert!(
            state.source_custody.is_empty(),
            "the completed claim releases its custody"
        );
        assert!((*(*state.records_for_test())[&row_key].source_commits_for_test()).is_empty());
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
