//! Actual publication receipts, not attempted executions, pay for inspection.
use worth_query_execution::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::frontier::DependencySchedule;
use worth_query_execution::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::frontier::selection::MemberInput;
use worth_query_execution::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::frontier::MemberState;
use worth_query_execution::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::frontier::RequiredFrontier;
use worth_query_execution::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::frontier::RequiredMember;
use worth_query_execution::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::ordinary::DecisionInput;
use worth_query_execution::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::PerformedMembers;
use worth_query_execution::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::SelectedDecisionInput;
use worth_query_execution::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceEpoch;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::demand_key::WorthQueryOutputDemandKey;
use worth_query_execution::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::caller;
use worth_query_execution::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::frontier;
use worth_query_execution::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::publication;
use worth_query_execution::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::queued;
use worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::outcome::commit_receipt;
use worth_query_execution::facade::application_contribution::WorthQueryProducerApplicability;
use worth_query_execution::facade::application_contribution::WorthQueryProducerLifecyclePosture;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::own_write_fixture::with_committed_own_write;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;

pub fn queue() -> RequiredFrontier<()> {
    RequiredFrontier {
        members: vec![RequiredMember {
            ready: MemberInput::native(()),
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
pub fn assert_publication_credit(replay: bool, publish: bool) {
    with_committed_own_write(|world, receipt, _, _| {
        world
            .application
            .with_host_advancement(|phase| {
                let execution = phase
                    .execution_request_for(&world.application.product_runtime)
                    .unwrap();
                let runtime = &world.application;

                let (shared, positioned) = worth_query_execution::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::selection::select_required_basis(
                    &execution,
                    runtime,
                    runtime.current_world(),
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
                let permission = record
                    .fresh(
                        &execution,
                        &a,
                        source.clone(),
                        &DecisionInput::Selected(&input),
                    )
                    .unwrap_or_else(|_| panic!("A's first preparation is admitted"))
                    .unwrap();
                let permission = record.attach_successor(permission, &a).unwrap();
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
                assert!(
                    record.reinspection(&a, inspected).unwrap().is_none(),
                    "a member's own publication pays for no retry"
                );
                let credit = record.reinspection(&m, inspected).unwrap();
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
                let credit = record.reinspection(&m, inspected_again).unwrap();
                assert!(
                    credit.is_none(),
                    "one publication cannot pay for two inspections"
                );
                assert!(!frontier.queued_member(0).unwrap().refused(credit));
                assert!(matches!(frontier.members[0].state, MemberState::Pending));
            })
            .expect("the owner proof owns its request");
    });
}
