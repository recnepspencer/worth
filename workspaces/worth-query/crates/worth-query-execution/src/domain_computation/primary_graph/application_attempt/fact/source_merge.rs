use super::WorthQueryApplicationObservedFact as Fact;

impl Fact {
    /// Combines observations of the same dependency at one native revision.
    /// Adjacency endpoints and comparison limits describe each read boundary;
    /// they may differ even when both reads observed the same structural truth.
    pub(in crate::domain_computation::primary_graph) fn merge_same_source_fact(
        &mut self,
        duplicate: Self,
    ) -> bool {
        if self.dependency_key() != duplicate.dependency_key() {
            return false;
        }
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
                *first_limit = (*first_limit).max(second_limit);
                first_endpoints.extend(second_endpoints);
                first_endpoints.sort();
                first_endpoints.dedup();
                true
            }
            (existing, duplicate) => *existing == duplicate,
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
            endpoints,
        }
    }

    #[test]
    fn same_revision_source_adjacency_unions_coverage_and_comparison_allowance() {
        let first = adjacency(4, 2, vec![endpoint(3), endpoint(2)]);
        let second = adjacency(4, 7, vec![endpoint(4), endpoint(3)]);
        let expected = adjacency(4, 7, vec![endpoint(2), endpoint(3), endpoint(4)]);
        let key = first.dependency_key();
        let mut forward = first.clone();
        assert!(forward.merge_same_source_fact(second.clone()));
        assert_eq!(forward, expected);
        assert_eq!(forward.dependency_key(), key);
        let mut reverse = second;
        assert!(reverse.merge_same_source_fact(first));
        assert_eq!(reverse, expected);
        assert!(forward.merge_same_source_fact(expected.clone()));
        assert_eq!(forward, expected);
    }

    #[test]
    fn changed_revision_or_native_locator_denies_without_mutating_source_fact() {
        let original = adjacency(4, 2, vec![endpoint(2)]);
        let mut observed = original.clone();
        assert!(!observed.merge_same_source_fact(adjacency(5, 7, vec![endpoint(3)])));
        assert_eq!(observed, original);
        let mut foreign = adjacency(4, 7, vec![endpoint(3)]);
        if let Fact::SourceAdjacencyRevision { anchor, .. } = &mut foreign {
            *anchor = endpoint(5);
        }
        assert!(!observed.merge_same_source_fact(foreign));
        assert_eq!(observed, original);
        let mut entity = Fact::SourceEntity {
            entity_id: endpoint(1),
        };
        assert!(entity.merge_same_source_fact(entity.clone()));
        assert!(!entity.merge_same_source_fact(Fact::SourceEntity {
            entity_id: endpoint(2)
        }));
        assert_eq!(
            entity,
            Fact::SourceEntity {
                entity_id: endpoint(1)
            }
        );
    }
}
