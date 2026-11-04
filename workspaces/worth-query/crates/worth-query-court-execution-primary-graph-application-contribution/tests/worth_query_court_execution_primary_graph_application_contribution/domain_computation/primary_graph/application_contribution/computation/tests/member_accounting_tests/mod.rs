//! Explicit accounting survives the same installed member boundary as native compute.
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationOwner;
use crate::domain_computation::primary_graph::application_contribution::computation::tests::*;
use worth_foundational::DeterminismContract;
use worth_foundational::ExecutionBudget;
use worth_foundational::ExecutionPosture;
use worth_foundational::ExecutionRequestPolicy;
use worth_query_execution::facade::domain_computation::HandlerFailure;
use worth_query_execution::facade::domain_computation::HandlerResult;

pub struct Owner {
    pub accounted: bool,
}
impl WorthQueryManagedComputationOwner<Schema, Feature, Computation> for Owner {
    type Prepared = u64;
    type Computed = u64;
    type Output = u64;
    type Stopped = HandlerFailure<&'static str>;

    fn prepare(&self, input: &u64) -> Result<u64, Self::Stopped> {
        Ok(*input)
    }
    fn compute(
        &self,
        prepared: &u64,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<Self::Stopped>> {
        assert_eq!((*checkpoint.charge_for_test()).is_some(), self.accounted);
        checkpoint.poll()?;
        if self.accounted {
            checkpoint.advance(3)?;
        }
        Ok(prepared * 2)
    }
    fn complete(&self, prepared: u64, computed: u64) -> Result<u64, Self::Stopped> {
        assert_eq!(computed, prepared * 2);
        Ok(computed)
    }
}

#[test]
fn installed_member_preserves_explicit_accounting_and_worker_only_polling() {
    let native = ExecutionRequestPolicy::for_workers(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        std::num::NonZeroUsize::MIN,
    );
    let accounted = |ceiling| {
        ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, ceiling),
        )
    };
    for policy in [native, accounted(None), accounted(Some(2))] {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(None, cancellation.token());
        let execution = QueryRequestExecution::open_for_test(
            worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(policy),
            &scope,
        );
        let installed =
            WorthQueryInstalledManagedComputation::<Schema, Feature, Computation, _>::new_for_test(Owner {
                accounted: policy.accounting().is_some(),
            });
        let result = installed
            .decision_plan(&7, HandlerResult::Completed)
            .run_for_test(&execution);
        if policy.work_ceiling() == Some(2) {
            let HandlerResult::ExecutionDenied(denial) = result else {
                panic!("an actual accounted exhaustion must refuse: {result:?}");
            };
            assert_eq!(
                denial.downcast::<worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial>().unwrap(),
                worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::WorkExhausted,
                ),
            );
        } else {
            assert!(matches!(result, HandlerResult::Completed(14)), "{result:?}");
        }
    }
}
