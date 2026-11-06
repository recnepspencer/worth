use crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::tests::*;
#[test]
fn post_effect_rebase_door_preserves_stops_at_every_consult() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world.application.with_host_advancement(|phase| {
        let (entity, locator) = account_note(&phase, &world, "account-1");
        let kind = (*(*world.application.runtime_for_test()).primary_graph_for_test().unwrap().layout_for_test()).entity_kind_for_test(AccountIdentity::reference().entity()).unwrap();
        let fact = WorthQueryApplicationObservedFact::AbsentField { entity_id: entity, kind, locator: planned(&locator) };
        worth_query_execution::test_read_stops::every_consult_under(&phase.borrowed_request_for_test(),|execution| match rebase_result_at_current(&phase, execution, &world, vec![fact.clone()]) {
            RebasedSourceFacts::Exact(_) => Ok(true),
            RebasedSourceFacts::VerificationRequired { reason, .. } => match reason.into_reason_for_test() {
                RebaseVerificationReason::Execution(cause) => Err(cause),
                other => panic!("a native stop must keep its cause: {other:?}"),
            },
            other => panic!("a native stop must keep its cause: {other:?}"),
        }, worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled, worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted);
    }).unwrap();
}

#[test]
fn selected_native_rebase_preserves_stops_at_every_consult() {
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::native::NativeFactPayloadBuilder;
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world.application.with_host_advancement(|phase| {
        let (entity, locator) = account_note(&phase, &world, "account-1");
        let kind = (*(*world.application.runtime_for_test()).primary_graph_for_test().unwrap().layout_for_test()).entity_kind_for_test(AccountIdentity::reference().entity()).unwrap();
        let control = StorageControl::new_for_test(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None);
        let mut facts = NativeFactPayloadBuilder::allocate(2, control).unwrap();
        facts.push(WorthQueryApplicationObservedFact::AbsentField { entity_id: entity, kind, locator: planned(&locator) }).unwrap();
        // A later real source question exercises cancellation while the first
        // pre-funded action slot is being filled, before that source is asked.
        facts.push(WorthQueryApplicationObservedFact::SourceEntity { entity_id: entity }).unwrap();
        let facts = facts.seal().unwrap();
        let selected = world.selected_product(&phase.test_host_request_for_test());
        let snapshot = selected.application_basis_for_test().snapshot_handle();
        worth_query_execution::test_read_stops::every_consult_under(&phase.borrowed_request_for_test(), |execution| {
            let prepared = match worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::selected::SelectedRebase::prepare_full_for_test(
                facts.clone(), Arc::from([]), StorageControl::new_for_test(execution.allocation_policy(), None),
            ) {
                Ok(prepared) => prepared,
                Err(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::PreparedRebaseDenial::Retention(
                    worth_query_execution::facade::application_inspection::StoreDenial::Allocation(cause))) => {
                    use worth_execution::ExecutionAllocationDenialClass as Class;
                    use worth_execution::WorkCeilingDenial as Work;
                    use worth_execution::MapKernelStop as Stop;
                    return Err(match cause.classify() {
                        Class::Request(Work::Stopped(Stop::Cancelled)) => worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled,
                        Class::Request(Work::Stopped(Stop::WorkCeiling)) => worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted,
                        other => panic!("unexpected preparation refusal: {other:?}"),
                    });
                }
                Err(cause) => panic!("unexpected preparation refusal: {cause:?}"),
            };
let result = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap().integration_handle_for_test().with_runtime(execution, |runtime| {
                prepared.rebase_for_test(execution, runtime, snapshot, &BTreeSet::new(), true, StorageControl::new_for_test(execution.allocation_policy(), None))
            });


            match result {
                RebasedSourceFacts::Cached { facts, .. } => {
                    assert_eq!(facts.len(), 2);
                    assert!(matches!(&facts[0], WorthQueryApplicationObservedFact::SourceFieldRevision { .. }));
                    Ok(true)
                }
                RebasedSourceFacts::VerificationRequired { reason, .. } => match reason.into_reason_for_test() {
                RebaseVerificationReason::Execution(cause) => Err(cause),
                RebaseVerificationReason::AdmissionDenied(CompanionPreflightStop::Execution(cause)) => Err(cause),
                RebaseVerificationReason::AdmissionDenied(CompanionPreflightStop::Interrupted(event)) => Err(match event.interruption() {
                    worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled,
                    worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => worth_relational::facade::transactions::RelationalExecutionDenialCause::DeadlineElapsed,
                }),
                RebaseVerificationReason::NativePayloadStorage { class: worth_execution::ExecutionAllocationDenialClass::Request(worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::WorkCeiling)), .. } => Err(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted),
                    other => panic!("selected native refusal must preserve its request stop: {other:?}"),
                },
                other => panic!("selected native resolution must preserve the original request stop: {other:?}"),
            }
        }, worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled,
           worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted);
    }).unwrap();
}
