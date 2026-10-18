use super::*;
#[test]
fn post_effect_rebase_door_preserves_stops_at_every_consult() {
    let world = installed_authorization_world(true);
    world.application.with_host_advancement(|phase| {
        let (entity, locator) = account_note(&phase, &world, "account-1");
        let kind = world.application.runtime.primary_graph().unwrap().layout.entity_kind(AccountIdentity::reference().entity()).unwrap();
        let fact = WorthQueryApplicationObservedFact::AbsentField { entity_id: entity, kind, locator: planned(&locator) };
        crate::test_read_stops::every_consult_under(&phase.borrowed_request_for_test(),|execution| match rebase_result_within(&phase, execution, &world, vec![fact.clone()], 64) {
            RebasedSourceFacts::Exact(_) => Ok(true),
            RebasedSourceFacts::VerificationRequired { reason: RebaseVerificationReason::Execution(cause), .. } => Err(cause),
            other => panic!("a native stop must keep its cause: {other:?}"),
        }, worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled, worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted);
    }).unwrap();
}

#[test]
fn selected_native_rebase_preserves_stops_at_every_consult() {
    use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
    use crate::domain_computation::primary_graph::application_checkpoint::facts::NativeFactPayloadBuilder;
    let world = installed_authorization_world(true);
    world.application.with_host_advancement(|phase| {
        let (entity, locator) = account_note(&phase, &world, "account-1");
        let kind = world.application.runtime.primary_graph().unwrap().layout.entity_kind(AccountIdentity::reference().entity()).unwrap();
        let control = StorageControl::new(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None);
        let mut facts = NativeFactPayloadBuilder::allocate(2, control).unwrap();
        facts.push(WorthQueryApplicationObservedFact::AbsentField { entity_id: entity, kind, locator: planned(&locator) }).unwrap();
        // A later real source question exercises cancellation while the first
        // pre-funded action slot is being filled, before that source is asked.
        facts.push(WorthQueryApplicationObservedFact::SourceEntity { entity_id: entity }).unwrap();
        let facts = facts.seal().unwrap();
        let selected = world.selected_product(&phase.test_host_request());
        let snapshot = selected.application_basis().snapshot_handle();
        crate::test_read_stops::every_consult_under(&phase.borrowed_request_for_test(), |execution| {
            let prepared = match super::super::selected::SelectedRebase::prepare_full(
                facts.clone(), Arc::from([]), StorageControl::new(execution.allocation_policy(), None),
            ) {
                Ok(prepared) => prepared,
                Err(super::super::PreparedRebaseDenial::Retention(
                    crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::Allocation(cause))) => {
                    use worth_execution::{ExecutionAllocationDenialClass as Class, WorkCeilingDenial as Work, MapKernelStop as Stop};
                    return Err(match cause.classify() {
                        Class::Request(Work::Stopped(Stop::Cancelled)) => worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled,
                        Class::Request(Work::Stopped(Stop::WorkCeiling)) => worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted,
                        other => panic!("unexpected preparation refusal: {other:?}"),
                    });
                }
                Err(cause) => panic!("unexpected preparation refusal: {cause:?}"),
            };
            let result = world.application.runtime.primary_graph().unwrap().integration_handle().with_runtime(execution, |runtime| {
                prepared.rebase(execution, runtime, snapshot, &BTreeSet::new(), true, 64,
                    &mut world.application.primary_provider.graph.source_owner.invalidation_owner.edit_admission(),
                    StorageControl::new(execution.allocation_policy(), None))
            });
            match result {
                RebasedSourceFacts::Cached { facts, .. } => {
                    assert_eq!(facts.len(), 2);
                    assert!(matches!(&facts[0], WorthQueryApplicationObservedFact::SourceFieldRevision { .. }));
                    Ok(true)
                }
                RebasedSourceFacts::VerificationRequired { reason: RebaseVerificationReason::Execution(cause), .. } => Err(cause),
                RebasedSourceFacts::VerificationRequired { reason: RebaseVerificationReason::AdmissionDenied(CompanionPreflightStop::Execution(cause)), .. } => Err(cause),
                RebasedSourceFacts::VerificationRequired { reason: RebaseVerificationReason::AdmissionDenied(CompanionPreflightStop::Interrupted(event)), .. } => Err(match event.interruption() {
                    worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled,
                    worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => worth_relational::facade::transactions::RelationalExecutionDenialCause::DeadlineElapsed,
                }),
                RebasedSourceFacts::VerificationRequired { reason: RebaseVerificationReason::NativePayloadStorage { class: worth_execution::ExecutionAllocationDenialClass::Request(worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::WorkCeiling)), .. }, .. } => Err(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted),
                other => panic!("selected native resolution must preserve the original request stop: {other:?}"),
            }
        }, worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled,
           worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted);
    }).unwrap();
}
