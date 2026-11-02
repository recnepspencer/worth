//! Consuming receipt admission returns the original undelivered custody.
use super::*;
use crate::branch::reference_test_fixture::{real_fixture, RealReferenceFixture};
use crate::facade::*;

type Owner = RuntimeWorldOwner<(), (), (), (), ()>;
struct Clock;
impl RuntimeWorldClockSource for Clock {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(0)
    }
}
fn published() -> (RealReferenceFixture, Owner, ConsumedCompositePublication) {
    let mut fixture = real_fixture(12, 12);
    let inputs = fixture.owner_inputs(
        RuntimeWorldBudgets::without_policy_limits(),
        RuntimeWorldClock::from_source(Clock),
    );
    let owner = RuntimeWorldOwner::builder()
        .with_execution_policy(inputs.execution.request_policy())
        .with_bridge_correspondence(inputs.bridge)
        .with_relational_services(inputs.relational)
        .with_signal_services(inputs.signal)
        .with_signal_definition_publication(inputs.signal_definition_publication)
        .with_budgets(inputs.budgets)
        .with_clock(inputs.clock)
        .build()
        .unwrap();
    let expected = match owner
        .lifecycle_port()
        .bootstrap_root(fixture.bootstrap_intent())
        .unwrap()
    {
        RuntimeWorldBootstrapOutcome::Performed(value) => value.product_branch().clone(),
        other => panic!("actual root: {other:?}"),
    };
    let policy = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&policy);
    let cancel = RuntimeWorldCancellationSource::new();
    let candidate =
        fixture.prepare_relational_owner_candidate(&request, "original-receipt-retirement");
    let prepared = owner
        .publication_port()
        .prepare_without_signal(
            expected,
            CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
                .with_prepared_relational_candidate(candidate),
            &cancel.token(),
            None,
        )
        .unwrap();
    let publication =
        match owner
            .publication_port()
            .execute_without_signal(request, prepared, &cancel.token())
        {
            RuntimeWorldPublicationOutcome::Performed(value) => value.consume(),
            other => panic!("actual publication: {other:?}"),
        };
    (fixture, owner, publication)
}
#[test]
fn prepaid_retirement_refuses_another_undelivered_publication_and_returns_both_original_owners() {
    let (_first_fixture, _first_owner, mut first) = published();
    let (second_fixture, _second_owner, second) = published();
    let identity = second.commit().identity().clone();
    let policy = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&policy);
    let first_history = first
        .take_history_protection()
        .expect("actual first history handoff");
    let prepaid = first.prepare_home_retirement(&request).unwrap();
    let (same_prepaid, mut same_second, cause) = match prepaid.begin(second) {
        Err(refusal) => refusal,
        Ok(_) => panic!("vacant storage cannot retire another pending handoff"),
    };
    assert!(matches!(
        cause,
        ConsumedPublicationRetirementDenial::UndeliveredCustody
    ));
    assert_eq!(same_second.commit().identity(), &identity);
    let second_history = same_second
        .take_history_protection()
        .expect("original history remains owed");
    let mut task = same_prepaid
        .begin(same_second)
        .unwrap_or_else(|_| panic!("actual handoff completed"));
    let cancel = worth_execution::CancellationSource::new();
    let stopped = crate::test_execution_policy::serial_request().with_cancellation(cancel.token());
    cancel.cancel();
    assert!(matches!(
        task.advance(
            &ExecutionRequest::serial(&stopped),
            second_fixture.relational_runtime()
        ),
        Err(ConsumedPublicationRetirementDenial::Execution(
            worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)
        ))
    ));
    assert_eq!(
        task.allocation
            .backing
            .original
            .as_ref()
            .unwrap()
            .commit()
            .identity(),
        &identity
    );
    assert!(
        task.into_retired().is_err(),
        "stopped work cannot claim retirement"
    );
    drop((first_history, second_history));
}
