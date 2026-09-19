use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::{
    BulkMutationLocalityFootprint, CreateIntent, EntityMutationIntent, MergedCommitPlan,
    MutationIntent, ProvenanceCompleteBulkMutationBatch, RelationMutationIntent,
};

#[derive(Debug, Clone)]
pub(super) struct BulkMutationPlanTelemetry {
    pub(super) locality: BulkMutationLocalityFootprint,
    pub(super) normalized_client_key_count: usize,
    pub(super) lineage_transition_count: usize,
    pub(super) provenance_record_count: usize,
}

pub(super) fn summarize_bulk_mutation_telemetry(
    merged_plan: &MergedCommitPlan,
    worker_batch_count: usize,
) -> Option<BulkMutationPlanTelemetry> {
    if merged_plan.merged_intents.is_empty() {
        return None;
    }

    let mut footprint = BulkMutationTelemetryAccumulator::default();
    for intent in &merged_plan.merged_intents {
        footprint.record_intent(intent);
    }

    Some(footprint.into_plan_telemetry(worker_batch_count))
}

pub(super) fn telemetry_from_strategy_batch(
    batch: &ProvenanceCompleteBulkMutationBatch,
) -> BulkMutationPlanTelemetry {
    BulkMutationPlanTelemetry {
        locality: batch.planned().locality.clone(),
        normalized_client_key_count: batch.planned().naming.normalized_client_keys.len(),
        lineage_transition_count: batch.planned().lineage.transitions.len(),
        provenance_record_count: batch.planned().provenance.worker_batch_names.len(),
    }
}

pub(super) fn record_bulk_mutation_telemetry(
    runtime: &RelationalPreparationRuntime,
    telemetry: &BulkMutationPlanTelemetry,
) {
    runtime.performance_access().count_bulk_mutation_plan(
        &telemetry.locality,
        telemetry.normalized_client_key_count,
        telemetry.lineage_transition_count,
        telemetry.provenance_record_count,
    );
}

#[derive(Default)]
struct BulkMutationTelemetryAccumulator {
    touched_partitions: std::collections::BTreeSet<crate::identity::data::PartitionId>,
    cross_partition_relation_count: usize,
    entity_target_count: usize,
    relation_target_count: usize,
    normalized_client_key_count: usize,
    lineage_transition_count: usize,
}

impl BulkMutationTelemetryAccumulator {
    fn record_intent(&mut self, intent: &MutationIntent) {
        intent.seed_touched_partitions(&mut self.touched_partitions);
        match intent {
            MutationIntent::Create(CreateIntent::Entity(_)) => {
                self.record_entity_creation(1, 1);
            }
            MutationIntent::Create(CreateIntent::EntityAspects(_)) => {
                self.record_entity_creation(1, 1);
            }
            MutationIntent::Create(CreateIntent::BulkEntities(spec)) => {
                self.record_entity_creation(spec.field_patches.len(), spec.client_keys.len());
            }
            MutationIntent::Create(CreateIntent::Relation(spec)) => {
                self.record_relation_creation(
                    1,
                    1,
                    usize::from(spec.source.partition_id() != spec.target.partition_id()),
                );
            }
            MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
                self.record_relation_creation(
                    1,
                    1,
                    usize::from(spec.source.partition_id() != spec.target.partition_id()),
                );
            }
            MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
                self.record_relation_creation(
                    spec.endpoints.len(),
                    spec.client_keys.len(),
                    spec.endpoints
                        .iter()
                        .filter(|(source, target)| source.partition_id() != target.partition_id())
                        .count(),
                );
            }
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(_))
            | MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(_)) => {
                self.entity_target_count += 1;
            }
            MutationIntent::Entity(EntityMutationIntent::Replace(_)) => {
                self.record_entity_creation(1, 1);
            }
            MutationIntent::Entity(EntityMutationIntent::Delete(_)) => {
                self.entity_target_count += 1;
                self.lineage_transition_count += 1;
            }
            MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(_)) => {
                self.relation_target_count += 1;
            }
            MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(_))
            | MutationIntent::Relation(RelationMutationIntent::Delete(_)) => {
                self.relation_target_count += 1;
                self.lineage_transition_count += 1;
            }
            MutationIntent::Materialization(intent) => match intent.record() {
                crate::transactions::data::RecordRef::Entity(_) => {
                    self.entity_target_count += 1;
                }
                crate::transactions::data::RecordRef::Relation(_) => {
                    self.relation_target_count += 1;
                }
            },
        }
    }

    fn record_entity_creation(&mut self, target_count: usize, client_key_count: usize) {
        self.entity_target_count += target_count;
        self.normalized_client_key_count += client_key_count;
        self.lineage_transition_count += client_key_count;
    }

    fn record_relation_creation(
        &mut self,
        target_count: usize,
        client_key_count: usize,
        cross_partition_count: usize,
    ) {
        self.relation_target_count += target_count;
        self.normalized_client_key_count += client_key_count;
        self.lineage_transition_count += client_key_count;
        self.cross_partition_relation_count += cross_partition_count;
    }

    fn into_plan_telemetry(self, worker_batch_count: usize) -> BulkMutationPlanTelemetry {
        BulkMutationPlanTelemetry {
            locality: BulkMutationLocalityFootprint {
                touched_partitions: self
                    .touched_partitions
                    .into_iter()
                    .collect::<Vec<_>>()
                    .into(),
                cross_partition_relation_count: self.cross_partition_relation_count,
                entity_target_count: self.entity_target_count,
                relation_target_count: self.relation_target_count,
            },
            normalized_client_key_count: self.normalized_client_key_count,
            lineage_transition_count: self.lineage_transition_count,
            provenance_record_count: worker_batch_count,
        }
    }
}
