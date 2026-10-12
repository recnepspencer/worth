use super::*;
use crate::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use crate::identity::data::EntityId;
use crate::query::data::SnapshotPinnedQueryPlan;
use worth_foundational::facade::{AspectFieldLocator, CanonicalFieldPath, LocatorAuthority};

pub(super) struct World {
    pub runtime: RelationalRuntime,
    pub snapshot: SnapshotHandle,
    pub entities: [EntityId; 3],
    pub relations: [RelationId; 2],
    pub name: AspectFieldLocator,
    pub label: AspectFieldLocator,
}

impl World {
    pub(super) fn new(request: &ExecutionRequest<'_, '_>, indexed: bool) -> Self {
        let runtime =
            runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
        let entities = ["a", "b", "c"]
            .map(|name| create_entity_in_partition(request, &runtime, name, PartitionId(7)));
        let relations = [
            create_relation_in_partition(
                request,
                &runtime,
                entities[0],
                entities[1],
                "ab",
                PartitionId(7),
            ),
            create_relation_in_partition(
                request,
                &runtime,
                entities[1],
                entities[2],
                "bc",
                PartitionId(7),
            ),
        ];
        let name = AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            aspect_key("name"),
            CanonicalFieldPath::single(field_key("name")),
        );
        let label = AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            aspect_key("label"),
            CanonicalFieldPath::single(field_key("label")),
        );
        let definition = runtime.index_authority().register(DerivedIndexDefinition {
            index_id: DerivedIndexId(17),
            name: "read-law-name".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: name.clone(),
            },
            branch_scoped: false,
        });
        if indexed {
            let built = runtime.index_authority().build_for_commit(
                request,
                DerivedIndexBuildRequest {
                    source_commit_id: runtime.history().latest_commit().unwrap().commit_id,
                    branch_id: BranchId("main".into()),
                    index_ids: vec![definition.index_id],
                },
            );
            assert!(built.failed_indexes.is_empty());
            assert_eq!(built.generations.len(), 1);
        }
        let snapshot = runtime.visibility_authority().snapshot();
        Self {
            runtime,
            snapshot,
            entities,
            relations,
            name,
            label,
        }
    }

    pub(super) fn plan(&self, case: &cases::Case) -> SnapshotPinnedQueryPlan {
        let context_id = self
            .runtime
            .read_truth()
            .query_plan_context(&self.snapshot)
            .unwrap();
        self.runtime
            .read_truth()
            .plan_query_packet(
                &self.snapshot,
                PlannedQueryPacket {
                    label: case.name.into(),
                    context_id,
                    scope: case.scope.clone(),
                    locality: QueryLocalityClass::SinglePartition {
                        partition_id: PartitionId(7),
                    },
                    ordering: case.ordering,
                    access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                    execution_shape: QueryExecutionShape::BulkPacketized,
                    reduction: ReductionDiscipline::DeterministicMerge,
                    plan_key: DeterministicQueryPlanKey(9_176_500),
                    target_count_hint: 0,
                },
            )
            .unwrap()
    }
}
