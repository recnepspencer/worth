//! Admission-owned access to actual retained observations for cancellation proofs.
use super::*;
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};
impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation) fn assert_fact_dispatch_stop_oracle(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
    ) {
        use crate::domain_computation::primary_graph::application_attempt::consult_observation::Door;
        use crate::domain_computation::primary_graph::application_attempt::consult_observation::ReadConsultObservationForTest;
        use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind as Kind;
        use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationDecisionFact as Fact;
        let Some(WorthQueryRetainedAuthorizationDecisionFacts::Abilities {
            principal,
            decisions,
        }) = &self.authorization
        else {
            panic!("conventional retained observations");
        };
        assert_eq!(decisions.len(), 1);
        let facts = [
            Fact::principal(principal.clone()),
            Fact::authorization(0, decisions[0].clone()),
        ];
        let read = |request: &ExecutionRequest<'_, '_>| {
            facts.iter().try_fold(true, |equal, fact| {
                if equal {
                    fact.remains_equal_in(request, runtime, snapshot)
                } else {
                    Ok(false)
                }
            })
        };
        let count = {
            let probe = ReadConsultObservationForTest::install_on_this_thread_for_test(
                Door::Equality,
                None,
            );
            assert!(read(execution).unwrap());
            probe.consults()
        };
        assert!(count > 1);
        println!("retained equality dispatch: {count} consults");
        for k in 1..=count {
            let cancel = CancellationSource::new();
            let serial = worth_execution::SerialRequest::from_memory(
                worth_execution::SerialMemoryBudget::new(None),
                cancel.token(),
                None,
            );
            let request = ExecutionRequest::serial(&serial);
            let probe = ReadConsultObservationForTest::install_callback_on_this_thread_for_test(
                Door::Equality,
                Some(k),
                move || cancel.cancel(),
            );
            assert_eq!(read(&request).unwrap_err().kind(), Kind::Cancelled);
            assert!(probe.consults() >= k);
            drop(probe);
            let serial = worth_execution::SerialRequest::from_memory(
                worth_execution::SerialMemoryBudget::new(None),
                worth_execution::CancellationToken::new(),
                None,
            );
            let request = ExecutionRequest::serial(&serial);
            let probe = ReadConsultObservationForTest::install_with_work_on_this_thread_for_test(
                Door::Equality,
                Some(k),
                move |context| {
                    assert_eq!(
                        {
                            let context = context.expect("the caller's active scope");
                            context.checkpoint(
                                context
                                    .remaining_work()
                                    .expect("this adversarial scope selected a finite ceiling")
                                    + 1,
                            )
                        },
                        Err(worth_execution::MapKernelStop::WorkCeiling)
                    );
                },
            );
            let (answer, _) = request
                .run(
                    worth_execution::ExecutionWorkCeiling::new(Some(u64::MAX - 1)),
                    |_| read(&request),
                )
                .unwrap();
            assert!(
                probe.consults() >= k,
                "the retained equality read reached consult {k}"
            );
            assert_eq!(answer.unwrap_err().kind(), Kind::ExecutionRequest(crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource { denial: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::WorkExhausted, partition_identity: None, policy_ancestor: None }));
        }
    }
    pub(in crate::domain_computation) fn assert_cancelled_retained_activation_read(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        source: &CancellationSource,
    ) {
        let Some(WorthQueryRetainedAuthorizationDecisionFacts::Abilities { decisions, .. }) =
            &self.authorization
        else {
            panic!("ViewAccount must retain conventional observations");
        };
        assert_eq!(decisions.len(), 1);
        decisions[0]
            .assert_retained_activation_read_cancelled(runtime, execution, snapshot, source);
    }
}
