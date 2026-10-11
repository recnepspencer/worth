//! Actual publication receipts, not attempted executions, pay for inspection.
use super::super::super::frontier::{
    DependencySchedule, MemberInput, MemberState, RequiredFrontier, RequiredMember,
};
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::execution::commit_receipt;
use crate::domain_computation::primary_graph::{
    application_contribution::{
        WorthQueryProducerApplicability, WorthQueryProducerLifecyclePosture,
    },
    output_lineage::own_write_fixture::with_committed_own_write,
    WorthQueryApplicationCommitOutcome,
};

fn queue() -> RequiredFrontier<()> {
    RequiredFrontier {
        members: vec![RequiredMember {
            ready: MemberInput::Selected(()),
            caller: false,
            caller_successor: false,
            resolved_successor: false,
            queued: None,
            contacts: 0,
            state: MemberState::Prepared,
            dependency_schedule: DependencySchedule::OwnSelection,
        }],
    }
}
#[test]
fn a_caller_refusal_has_no_reinspection_transition() {
    let mut frontier = queue();
    frontier.members[0].caller = true;
    assert!(frontier.queued_member(0).is_none());
}
#[test]
fn a_stale_queue_publication_without_another_publication_is_held() {
    assert_publication_credit(false, false);
}
#[test]
fn another_members_publication_pays_only_one_reinspection() {
    assert_publication_credit(false, true);
}
#[test]
fn a_replayed_receipt_pays_for_no_reinspection() {
    assert_publication_credit(true, true);
}
fn assert_publication_credit(replay: bool, publish: bool) {
    with_committed_own_write(|world, receipt, _, _| {
        let runtime = &world.application;
        let mut admission = runtime.demand_request_admission();
        let (shared, positioned) = super::super::super::selection::select_required_basis(
            runtime,
            runtime.current_world(),
            &mut admission,
        )
        .unwrap();
        let input = SelectedDecisionInput {
            shared: &shared,
            positioned: &positioned,
            runtime,
        };
        let source = WorthQueryObservedSourceEpoch::new(
            [1; 32],
            [2; 32],
            worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                1,
                1,
            ),
            shared
                .selected()
                .product()
                .observation()
                .lifecycle_incarnation(),
            0,
            [3; 32],
        );
        let key = |producer: &str| {
            WorthQueryOutputDemandKey::new(
                "performed-member-test-family",
                producer.to_owned(),
                WorthQueryProducerApplicability::new(
                    "test",
                    WorthQueryProducerLifecyclePosture::Initial,
                ),
                source.clone(),
            )
        };
        let a = key("A");
        let m = key("M");
        let mut record = PerformedMembers::start();
        let inspected = record.inspection();
        let certified = record.certify("Current at this generation");
        let permission = record
            .fresh(
                &a,
                source.clone(),
                &DecisionInput::Selected(&input),
                &mut admission,
            )
            .unwrap_or_else(|_| panic!("A's first preparation is admitted"))
            .unwrap();
        let permission = record
            .attach_successor(permission, &a, &mut admission)
            .unwrap();
        if publish {
            let outcome = if replay {
                WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt)
            } else {
                WorthQueryApplicationCommitOutcome::Committed(receipt)
            };
            let publication = commit_receipt("actual publication", outcome)
                .unwrap_or_else(|_| panic!("receipt retained"));
            record.publication(permission, publication, |member| member.performed());
        }
        assert_eq!(
            certified.into_current(&record),
            (!publish || replay).then_some("Current at this generation"),
            "only a new publication invalidates the certified decision"
        );
        assert!(
            record
                .reinspection(&a, inspected, &mut admission)
                .unwrap()
                .is_none(),
            "a member's own publication pays for no retry"
        );
        let credit = record.reinspection(&m, inspected, &mut admission).unwrap();
        assert_eq!(
            credit.is_some(),
            publish && !replay,
            "only another newly published member supplies the credit"
        );
        let mut frontier = queue();
        assert_eq!(
            frontier.queued_member(0).unwrap().refused(credit),
            publish && !replay
        );
        let inspected_again = record.inspection();
        let credit = record
            .reinspection(&m, inspected_again, &mut admission)
            .unwrap();
        assert!(
            credit.is_none(),
            "one publication cannot pay for two inspections"
        );
        assert!(!frontier.queued_member(0).unwrap().refused(credit));
        assert!(matches!(frontier.members[0].state, MemberState::Pending));
    });
}
