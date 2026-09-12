use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct EntityIdentityProjection {
    entity_id: crate::facade::identity::EntityId,
}

impl EntityRecordProjection for EntityIdentityProjection {
    const KIND: KindId = KindId(1);

    fn from_record(record: crate::facade::runtime::EntityProjectionRecord<'_>) -> Option<Self> {
        Some(Self {
            entity_id: record.entity_id(),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct MaterializationWaveRule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MaterializationWaveScope {
    visible_entities: usize,
    visible_relations: usize,
    traversed_entities: usize,
    traversed_relations: usize,
    touched_partitions: usize,
}

impl CustomInvariantRule for MaterializationWaveRule {
    type Scope = MaterializationWaveScope;

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("perf.materialization.wave"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Perf Materialization Wave"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(4096).unwrap(),
                access: crate::validation::data::CustomInvariantAccessContract {
                    read_entity_kinds: vec![crate::identity::data::KindId(1)],
                    read_relation_kinds: vec![crate::identity::data::KindId(2)],
                    affected_entity_kinds: vec![crate::identity::data::KindId(1)],
                    affected_relation_kinds: vec![crate::identity::data::KindId(2)],
                },
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        let touched = planner.touched();
        let traversal = planner
            .traversal()
            .walk_outgoing_from(touched.visible_entity_ids(), 2)?;
        Ok(MaterializationWaveScope {
            visible_entities: touched.visible_entity_ids().len(),
            visible_relations: touched.visible_relation_ids().len(),
            traversed_entities: traversal.visited_entities().len(),
            traversed_relations: traversal.traversed_relations().len(),
            touched_partitions: touched.touched_partitions().len(),
        })
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let counts = context.counts();
        let traversal = context
            .traversal()
            .walk_outgoing_from(context.touched().visible_entity_ids(), 2)?;
        if counts.visible_entity_count() == scope.visible_entities
            && counts.visible_relation_count() == scope.visible_relations
            && counts.touched_partition_count() == scope.touched_partitions
            && traversal.visited_entities().len() == scope.traversed_entities
            && traversal.traversed_relations().len() == scope.traversed_relations
        {
            Ok(CustomInvariantVerdict::Pass)
        } else {
            Ok(CustomInvariantVerdict::Violation)
        }
    }
}

pub(super) fn runtime_with_test_schema_profile_and_custom_invariant(
    profile: RelationalRuntimeProfile,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(profile)
        .schema_registry(test_schema_registry())
        .custom_invariant(CustomInvariantRegistration::new(MaterializationWaveRule).unwrap())
        .build()
}

pub(super) fn fresh_diagnostics_metrics(
    runtime: &RelationalRuntime,
    diagnostics_start: usize,
) -> (usize, usize) {
    let publication = runtime.publication();
    let diagnostics = publication.diagnostic_artifacts();
    let fresh_artifacts = &diagnostics[diagnostics_start..];
    let detailed_trace_entries = fresh_artifacts
        .iter()
        .filter(|artifact| {
            artifact.kind == crate::facade::diagnostics::DiagnosticsArtifactKind::DetailedTrace
        })
        .map(|artifact| artifact.entries.len())
        .sum::<usize>();
    (fresh_artifacts.len(), detailed_trace_entries)
}

pub(super) fn dense_patch_record_count(runtime: &RelationalRuntime) -> usize {
    runtime
        .publication()
        .latest_patch()
        .map(|patch| {
            patch
                .authoritative_record_patches
                .iter()
                .filter(|record| {
                    matches!(
                        record.detail,
                        crate::publication::patch::data::PatchDetail::DenseBitset(_)
                    )
                })
                .count()
        })
        .unwrap_or(0)
}

pub(super) fn entity_name_index_packet(
    runtime: &RelationalRuntime,
    snapshot: &crate::facade::snapshots::SnapshotHandle,
    label: &str,
    value: &str,
) -> PlannedQueryPacket {
    let context = runtime
        .read_truth()
        .query_plan_context(snapshot)
        .expect("query plan context");
    PlannedQueryPacket {
        label: label.to_string(),
        context_id: context,
        scope: QueryScope::EntityFieldEquals {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            value: string_aspect_value(value),
            partition_scope: None,
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(1901),
        target_count_hint: 0,
    }
}
