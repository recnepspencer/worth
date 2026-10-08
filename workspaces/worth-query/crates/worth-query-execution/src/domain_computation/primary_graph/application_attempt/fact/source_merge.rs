use super::WorthQueryApplicationObservedFact as Fact;

impl Fact {
    /// Combines observations of the same dependency at one native revision.
    /// Adjacency endpoints and comparison limits describe each read boundary;
    /// they may differ even when both reads observed the same structural truth.
    pub(in crate::domain_computation::primary_graph) fn merge_same_source_fact(
        &mut self,
        duplicate: Self,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        request: Option<
            &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        >,
    ) -> Result<bool, super::super::retained_decision_facts::StoreDenial> {
        if self.dependency_key() != duplicate.dependency_key() {
            return Ok(false);
        }
        self.merge_after_equal_source_key(
            duplicate,
            super::super::retained_decision_facts::StorageControl::new(allocation_policy, request),
        )
    }
    /// Private retained-store callback. The caller has compared the complete
    /// admitted typed key; native revision equality remains checked below.
    pub(in crate::domain_computation::primary_graph) fn merge_after_equal_source_key(
        &mut self,
        duplicate: Self,
        control: super::super::retained_decision_facts::StorageControl<'_, '_>,
    ) -> Result<bool, super::super::retained_decision_facts::StoreDenial> {
        match (self, duplicate) {
            (
                Fact::SourceAdjacencyRevision {
                    native_revision: first_revision,
                    comparison_work_limit: first_limit,
                    endpoints: first_endpoints,
                    ..
                },
                Fact::SourceAdjacencyRevision {
                    native_revision: second_revision,
                    comparison_work_limit: second_limit,
                    endpoints: second_endpoints,
                    ..
                },
            ) if *first_revision == second_revision => {
                let union = first_endpoints.union(&second_endpoints, control)?;
                // No body change until the complete replacement is admitted.
                *first_limit = (*first_limit).max(second_limit);
                *first_endpoints = union;
                Ok(true)
            }
            (existing, duplicate) => Ok(*existing == duplicate),
        }
    }
}

#[cfg(test)]
mod tests {
    use worth_relational::facade::{
        identity::{EntityId, KindId, PartitionId, VersionId},
        runtime::RelationalAdjacencyDirection,
    };

    use super::Fact;

    fn endpoint(slot: u64) -> EntityId {
        EntityId::new(PartitionId::main(), slot, 1)
    }

    fn adjacency(revision: u64, work: usize, endpoints: Vec<EntityId>) -> Fact {
        Fact::SourceAdjacencyRevision {
            relation_kind: KindId::new(120),
            anchor: endpoint(1),
            direction: RelationalAdjacencyDirection::Outgoing,
            native_revision: Some(VersionId(revision)),
            comparison_work_limit: work,
            endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(&endpoints, worth_execution::ExecutionAllocationPolicy::SystemAllocation, None).unwrap(),
        }
    }

    #[test]
    fn same_revision_source_adjacency_unions_coverage_and_comparison_allowance() {
        let first = adjacency(4, 2, vec![endpoint(3), endpoint(2)]);
        let second = adjacency(4, 7, vec![endpoint(4), endpoint(3)]);
        let expected = adjacency(4, 7, vec![endpoint(2), endpoint(3), endpoint(4)]);
        let key = first.dependency_key();
        let mut forward = first.clone();
        assert!(forward
            .merge_same_source_fact(
                second.clone(),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                None
            )
            .unwrap());
        assert_eq!(forward, expected);
        assert_eq!(forward.dependency_key(), key);
        let mut reverse = second;
        assert!(reverse
            .merge_same_source_fact(
                first,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                None
            )
            .unwrap());
        assert_eq!(reverse, expected);
        assert!(forward
            .merge_same_source_fact(
                expected.clone(),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                None
            )
            .unwrap());
        assert_eq!(forward, expected);
    }

    #[test]
    fn changed_revision_or_native_locator_denies_without_mutating_source_fact() {
        let original = adjacency(4, 2, vec![endpoint(2)]);
        let mut observed = original.clone();
        assert!(!observed
            .merge_same_source_fact(
                adjacency(5, 7, vec![endpoint(3)]),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                None
            )
            .unwrap());
        assert_eq!(observed, original);
        let mut foreign = adjacency(4, 7, vec![endpoint(3)]);
        if let Fact::SourceAdjacencyRevision { anchor, .. } = &mut foreign {
            *anchor = endpoint(5);
        }
        assert!(!observed
            .merge_same_source_fact(
                foreign,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                None
            )
            .unwrap());
        assert_eq!(observed, original);
        let mut entity = Fact::SourceEntity {
            entity_id: endpoint(1),
        };
        assert!(entity
            .merge_same_source_fact(
                entity.clone(),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                None
            )
            .unwrap());
        assert!(!entity
            .merge_same_source_fact(
                Fact::SourceEntity {
                    entity_id: endpoint(2)
                },
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                None
            )
            .unwrap());
        assert_eq!(
            entity,
            Fact::SourceEntity {
                entity_id: endpoint(1)
            }
        );
    }
}
