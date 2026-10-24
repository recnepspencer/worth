//! Exact native backing and stop semantics with actually retained query descriptors.
use super::*;
impl WorthQueryRetainedQueryDependencies {
    pub(in crate::domain_computation::primary_graph) fn assert_storage_custody_for_test(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> WorthQueryRetainedQueryDependencies {
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
            std::alloc::Layout::array::<RetainedQuerySource>(count)
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
                        inherited.max_workers(),
                        Some(quote - 1),
                        inherited.work_ceiling(),
                    ),
                ),
                deadline: None,
                cancellation: worth_execution::CancellationToken::new(),
            })
            .unwrap();
        let refusal = retain_descriptors(
            std::iter::once(self.sources.elements()),
            count,
            &worth_execution::ExecutionRequest::leased(&narrowed),
        )
        .err()
        .unwrap();
        assert!(
            matches!(refusal, WorthQueryRequiredSetCurrentnessDenial::Source(ref cause) if cause.allocation_denial().is_some())
        );
        assert_eq!(self.sources.len(), count);
        let retained =
            retain_descriptors(std::iter::once(self.sources.elements()), count, &execution)
                .unwrap();
        assert_eq!(retained.sources.len(), count);
        assert_eq!(retained.sources.charged_payload_bytes(), Some(quote));
        let serial_stop = CancellationSource::new();
        let serial = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(Some(quote)),
            serial_stop.token(),
            None,
        );
        let stopped_parts =
            std::iter::once(self.sources.elements()).inspect(|_| serial_stop.cancel());
        let refused = retain_descriptors(
            stopped_parts,
            count,
            &worth_execution::ExecutionRequest::serial(&serial),
        )
        .err()
        .unwrap();
        assert!(
            matches!(refused, WorthQueryRequiredSetCurrentnessDenial::Output(ref cause)
            if cause.kind() == WorthQueryOutputDemandDenialKind::ExecutionRequest(
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Interrupted(
                    crate::domain_computation::primary_graph::WorthQueryManagedComputationInterruption::Cancelled)))
        );
        assert_eq!(self.sources.len(), count);
        let fresh_serial = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(Some(quote)),
            worth_execution::CancellationToken::new(),
            None,
        );
        let serial_retained = retain_descriptors(
            std::iter::once(self.sources.elements()),
            count,
            &worth_execution::ExecutionRequest::serial(&fresh_serial),
        )
        .expect("a fresh serial request retains the same complete original descriptors");
        assert_eq!(serial_retained.sources.len(), count);
        let stop = CancellationSource::new();
        let child = parent.controlled_child(stop.token(), None);
        let mut building =
            ExecutionArrayBuilder::allocate(count, ExecutionAllocationPolicy::Execution(&child))
                .unwrap();
        building.push(self.sources.elements()[0].shared()).unwrap();
        stop.cancel();
        assert!(building.push(self.sources.elements()[1].shared()).is_err());
        assert!(building.seal().is_err());
        assert_eq!(
            self.sources.len(),
            count,
            "failed prefix owns only attempted shared copies"
        );
        drop(retained);
        serial_retained
    }
}
