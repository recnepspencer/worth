#[cfg(test)]
mod tests {
    use crate::facade::config::CascadeDeletePolicy;
    use crate::facade::identity::PartitionId;
    use crate::tests::support::{create_entity, runtime_with_declared_aspect_schema};

    #[test]
    fn aspect_version_enumeration_spends_the_request_and_keeps_native_causes() {
        let host_request = crate::execution::test_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        use crate::execution::RelationalExecutionDenialCause as Cause;
        use crate::transactions::data::CommitExecutionDenialKind;
        use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest};
        use worth_foundational::{
            DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
        };
        let runtime =
            runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
        let entity = create_entity(execution, &runtime, "alpha");
        // The declared fixture has lifecycle and name: two symbolic versions,
        // each costing one visit, cannot fit a one-unit request.
        for (memory, work, expected_work) in [(64 * 1024 * 1024, 1, true), (0, 100, false)] {
            let lease = crate::tests::support::test_execution_authority()
                .request_lease(LeaseRequest {
                    policy: ExecutionRequestPolicy::new(
                        ExecutionPosture::Serial,
                        DeterminismContract::CanonicalBitwise,
                        ExecutionBudget::new(
                            std::num::NonZeroUsize::new(1).unwrap(),
                            Some(memory),
                            Some(work),
                        ),
                    ),
                    deadline: None,
                    cancellation: CancellationToken::new(),
                })
                .unwrap();
            let stop = runtime
                .read_truth()
                .entity_aspect_versions(&ExecutionRequest::leased(&lease), entity)
                .unwrap_err();
            let kind = stop.execution_denial().unwrap().kind;
            if expected_work {
                assert_eq!(kind, CommitExecutionDenialKind::Cause(Cause::WorkExhausted));
            } else {
                assert!(matches!(
                    kind,
                    CommitExecutionDenialKind::Cause(Cause::PolicyMemoryExhausted {
                        admitted: 0,
                        ..
                    })
                ));
            }
        }
    }

    #[test]
    fn relation_aspect_version_enumeration_spends_the_request_and_keeps_native_causes() {
        let host_request = crate::execution::test_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        use crate::execution::RelationalExecutionDenialCause as Cause;
        use crate::transactions::data::CommitExecutionDenialKind;
        use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest};
        use worth_foundational::{
            DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
        };
        let runtime =
            runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
        let source = create_entity(execution, &runtime, "alpha");
        let target = create_entity(execution, &runtime, "beta");
        let relation =
            crate::tests::support::create_relation(execution, &runtime, source, target, "edge");
        // The declared fixture has lifecycle and label: two symbolic versions,
        // each costing one visit, cannot fit a one-unit request.
        for (memory, work, expected_work) in [(64 * 1024 * 1024, 1, true), (0, 100, false)] {
            let lease = crate::tests::support::test_execution_authority()
                .request_lease(LeaseRequest {
                    policy: ExecutionRequestPolicy::new(
                        ExecutionPosture::Serial,
                        DeterminismContract::CanonicalBitwise,
                        ExecutionBudget::new(
                            std::num::NonZeroUsize::new(1).unwrap(),
                            Some(memory),
                            Some(work),
                        ),
                    ),
                    deadline: None,
                    cancellation: CancellationToken::new(),
                })
                .unwrap();
            let stop = runtime
                .read_truth()
                .relation_aspect_versions(&ExecutionRequest::leased(&lease), relation)
                .unwrap_err();
            let kind = stop.execution_denial().unwrap().kind;
            if expected_work {
                assert_eq!(kind, CommitExecutionDenialKind::Cause(Cause::WorkExhausted));
            } else {
                assert!(matches!(
                    kind,
                    CommitExecutionDenialKind::Cause(Cause::PolicyMemoryExhausted {
                        admitted: 0,
                        ..
                    })
                ));
            }
        }
    }

    #[test]
    fn unresolved_symbolic_aspect_versions_are_not_exposed_as_foundational_keys() {
        let host_request = crate::execution::test_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        let runtime =
            runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
        let entity = create_entity(execution, &runtime, "alpha");
        {
            let mut writer = runtime.edit_partitions();
            let partition = writer
                .partition_mut(PartitionId::main())
                .expect("main partition");
            let versions = partition
                .entity_arena
                .aspect_versions_at_mut(entity.slot_index())
                .expect("test entity aspect version slot is materialized");
            *versions = Default::default();
            versions.insert(crate::symbols::data::Symbol(41), 7);
        }

        let observed = runtime
            .read_truth()
            .entity_aspect_versions(execution, entity)
            .expect("aspect-version request remains live")
            .expect("entity versions");

        assert!(observed.is_empty());
    }
}
