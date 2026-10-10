use super::*;

impl VisibilityReadContext<'_> {
    /// Enumerates aspect versions under the caller request.
    pub fn entity_aspect_versions(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: crate::identity::data::EntityId,
    ) -> Result<Option<Vec<(AspectKey, u64)>>, super::query_execution::QueryReadExecutionStop> {
        super::query_preparation::check_query_preparation_request(execution)?;
        let Some(partition) = self.runtime.partitions.partition(entity_id.partition_id) else {
            return Ok(None);
        };
        let slot = entity_id.slot_index();
        let Some(slot_view) = partition.entity_arena.get_slot(slot) else {
            return Ok(None);
        };
        if slot_view.generation() != entity_id.generation_value()
            || slot_view.partition_id() != entity_id.partition_id
        {
            return Ok(None);
        }
        let Some(symbolic) = partition.entity_arena.aspect_versions_at(slot) else {
            return Ok(None);
        };
        let versions =
            super::aspect_version_read::read_versions(self.runtime, execution, symbolic, false)?;
        debug_assert!(aspect_versions_are_canonical(&versions));
        Ok(Some(versions))
    }
    /// Enumerates and orders aspect versions under the caller request.
    pub fn relation_aspect_versions(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation_id: crate::identity::data::RelationId,
    ) -> Result<Option<Vec<(AspectKey, u64)>>, super::query_execution::QueryReadExecutionStop> {
        super::query_preparation::check_query_preparation_request(execution)?;
        let Some(partition) = self.runtime.partitions.partition(relation_id.partition_id) else {
            return Ok(None);
        };
        let slot = relation_id.slot_index();
        let Some(slot_view) = partition.relation_arena.get_slot(slot) else {
            return Ok(None);
        };
        if slot_view.generation() != relation_id.generation_value()
            || slot_view.partition_id() != relation_id.partition_id
        {
            return Ok(None);
        }
        let Some(symbolic) = partition.relation_arena.aspect_versions_at(slot) else {
            return Ok(None);
        };
        let versions =
            super::aspect_version_read::read_versions(self.runtime, execution, symbolic, true)?;
        debug_assert!(aspect_versions_are_canonical(&versions));
        Ok(Some(versions))
    }
}

fn aspect_versions_are_canonical(versions: &[(AspectKey, u64)]) -> bool {
    versions.windows(2).all(|window| window[0] <= window[1])
}

#[cfg(test)]
mod tests {
    use crate::facade::config::CascadeDeletePolicy;
    use crate::facade::identity::PartitionId;
    use crate::tests::support::{create_entity, runtime_with_declared_aspect_schema};

    #[test]
    fn aspect_version_enumeration_spends_the_request_and_keeps_native_causes() {
        use crate::execution::RelationalExecutionDenialCause as Cause;
        use crate::transactions::data::CommitExecutionDenialKind;
        use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest};
        use worth_foundational::{
            DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
        };
        let runtime =
            runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
        let entity = create_entity(&runtime, "alpha");
        // The declared fixture has lifecycle and name: two symbolic versions,
        // each costing one visit, cannot fit a one-unit request.
        for (memory, work, expected_work) in [(64 * 1024 * 1024, 1, true), (0, 100, false)] {
            let lease = crate::tests::support::test_execution_authority()
                .request_lease(LeaseRequest {
                    policy: ExecutionRequestPolicy::new(
                        ExecutionPosture::Serial,
                        DeterminismContract::CanonicalBitwise,
                        ExecutionBudget::new(std::num::NonZeroUsize::new(1).unwrap(), memory, work),
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
    fn unresolved_symbolic_aspect_versions_are_not_exposed_as_foundational_keys() {
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        let runtime =
            runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
        let entity = create_entity(&runtime, "alpha");
        {
            let mut writer = runtime.edit_partitions();
            let partition = writer
                .partition_mut(PartitionId::main())
                .expect("main partition");
            let versions = partition
                .entity_arena
                .aspect_versions_at_mut(entity.slot_index())
                .expect("test entity aspect version slot is materialized");
            versions.clear();
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
