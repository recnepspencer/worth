use super::*;

#[test]
fn generation_routing_respects_declared_branch_scope_without_history_inventory() {
    for branch_scoped in [true, false] {
        let runtime = runtime_with_index_field_aspects();
        let created = create_entity_outcome(&runtime, "alpha");
        let entity = changed_entities(&created)[0];
        let field = aspect_field_locator(aspect_key("name"), field_key("name"));
        let index = runtime.index_authority().register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "entity.name.branch-routing".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: field.clone(),
            },
            branch_scoped,
        });
        let main_build = runtime
            .index_authority()
            .build_for_commit(DerivedIndexBuildRequest {
                source_commit_id: created.commit.commit_id,
                branch_id: created.commit.branch_id.clone(),
                index_ids: vec![index.index_id],
            });
        assert!(main_build.failed_indexes.is_empty());
        let sibling = BranchId("same-root-sibling".into());
        runtime
            .history_authority()
            .fork_branch_from(sibling.clone(), &created.commit.branch_id)
            .unwrap();
        let identity = runtime.branch_identity(&sibling).unwrap();
        let (_, basis) = runtime.observe_branch(&identity).unwrap();
        let foreign_build = runtime.index_authority().build_for_basis(
            DerivedIndexBuildRequest {
                source_commit_id: created.commit.commit_id,
                branch_id: sibling,
                index_ids: vec![index.index_id],
            },
            &basis,
        );
        assert!(foreign_build.failed_indexes.is_empty());
        assert_eq!(
            foreign_build.generations[0].applicability.version_id,
            main_build.generations[0].applicability.version_id
        );
        assert!(
            foreign_build.generations[0].generation_id > main_build.generations[0].generation_id
        );
        let snapshot = created.snapshot.clone();
        let context = runtime.read_truth().query_plan_context(&snapshot).unwrap();
        let packet = crate::facade::query::PlannedQueryPacket {
            label: "same-root-branch-scope".into(),
            context_id: context,
            scope: crate::facade::query::QueryScope::EntityFieldEquals {
                field_locator: field,
                value: string_aspect_value("alpha"),
                partition_scope: None,
            },
            locality: crate::facade::query::QueryLocalityClass::CrossPartitionTraversal,
            ordering: crate::facade::query::QueryOrderingContract::CanonicalEntityIdOrder,
            access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
            execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
            reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
            plan_key: crate::facade::query::DeterministicQueryPlanKey(9_176_042),
            target_count_hint: 1,
        };
        let plan = runtime
            .read_truth()
            .plan_query_packet(&snapshot, packet)
            .unwrap();
        let before = runtime.index_access().generation_selection_counters();
        let observed = runtime
            .index_access()
            .execute_query_plan_with_index_parity(plan, IndexParityMode::CertificationParity)
            .unwrap();
        let expected = if branch_scoped {
            &main_build
        } else {
            &foreign_build
        };
        assert_eq!(
            observed.access_path,
            QueryAccessPath::DerivedIndexGeneration {
                generation_id: expected.generations[0].generation_id
            }
        );
        assert_eq!(observed.execution.result.entities.len(), 1);
        assert_eq!(observed.execution.result.entities[0].entity_id, entity);
        let counters = runtime.index_access().generation_selection_counters();
        assert_eq!(
            counters.generation_payload_reads - before.generation_payload_reads,
            2
        );
        assert_eq!(
            counters.history_inventory_entries - before.history_inventory_entries,
            0
        );
    }
}
