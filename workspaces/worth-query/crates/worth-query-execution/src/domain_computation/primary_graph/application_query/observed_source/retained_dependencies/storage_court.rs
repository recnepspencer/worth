//! Exact native backing and stop semantics with actually retained query descriptors.
use super::*;
impl WorthQueryRetainedQueryDependencies {
    pub(in crate::domain_computation::primary_graph) fn assert_storage_custody_for_test(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) {
        use worth_execution::{CancellationSource, ExecutionAllocationPolicy, LeaseRequest};
        use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};
        let ExecutionAllocationPolicy::Execution(parent) = execution.allocation_policy() else {
            panic!("the query court must use its actual managed request lease")
        };
        let count = self.sources.len();
        assert!(
            count > 1,
            "the complete real pairing includes selection and projected rows"
        );
        let quote = u64::try_from(
            std::alloc::Layout::array::<WorthQueryObservedSource<()>>(count)
                .unwrap()
                .size(),
        )
        .unwrap();
        assert_eq!(self.sources.charged_payload_bytes(), Some(quote));
        let inherited = parent.policy();
        let narrowed = parent
            .child(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    inherited.posture(),
                    inherited.determinism(),
                    ExecutionBudget::new(
                        inherited.budget().max_workers(),
                        quote - 1,
                        inherited.budget().work_ceiling(),
                    ),
                ),
                deadline: None,
                cancellation: worth_execution::CancellationToken::new(),
            })
            .unwrap();
        let refusal = retain_descriptors(
            self.sources.elements().iter(),
            count,
            ExecutionAllocationPolicy::Execution(&narrowed),
        )
        .err()
        .unwrap();
        assert!(
            matches!(refusal, WorthQueryRequiredSetCurrentnessDenial::Source(ref cause) if cause.allocation_denial().is_some())
        );
        assert_eq!(self.sources.len(), count);
        let retained = retain_descriptors(
            self.sources.elements().iter(),
            count,
            execution.allocation_policy(),
        )
        .unwrap();
        assert_eq!(retained.sources.len(), count);
        assert_eq!(retained.sources.charged_payload_bytes(), Some(quote));
        let stop = CancellationSource::new();
        let child = parent.controlled_child(stop.token(), None);
        let mut building =
            ExecutionArrayBuilder::allocate(count, ExecutionAllocationPolicy::Execution(&child))
                .unwrap();
        building
            .push(self.sources.elements()[0].retyped::<()>())
            .unwrap();
        stop.cancel();
        assert!(building
            .push(self.sources.elements()[1].retyped::<()>())
            .is_err());
        assert!(building.seal().is_err());
        assert_eq!(
            self.sources.len(),
            count,
            "failed prefix owns only attempted shared copies"
        );
        drop(retained);
    }
}
