//! Explicit accounting survives the same installed member boundary as native compute.
use super::*;
use crate::domain_computation::primary_graph::{HandlerFailure, HandlerResult};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

struct Owner {
    accounted: bool,
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
        assert_eq!(checkpoint.charge.is_some(), self.accounted);
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
        let execution = QueryRequestExecution::open(
            worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(policy),
            &scope,
        );
        let installed =
            WorthQueryInstalledManagedComputation::<Schema, Feature, Computation, _>::new(Owner {
                accounted: policy.accounting().is_some(),
            });
        let result = installed
            .decision_plan(&7, HandlerResult::Completed)
            .run(&execution);
        if policy.work_ceiling() == Some(2) {
            let HandlerResult::ExecutionDenied(denial) = result else {
                panic!("an actual accounted exhaustion must refuse: {result:?}");
            };
            assert_eq!(
                denial.downcast::<crate::domain_computation::primary_graph::WorthQueryAdvancementDenial>().unwrap(),
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::WorkExhausted,
                ),
            );
        } else {
            assert!(matches!(result, HandlerResult::Completed(14)), "{result:?}");
        }
    }
}
