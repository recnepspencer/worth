use contracts::transactions::data::intents::mutation_intent::CreateIntent;
use contracts::transactions::data::intents::mutation_intent::EntityMutationIntent;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::intents::mutation_intent::RelationMutationIntent;
use contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use contracts::transactions::data::primitives::BulkMutationLocalityFootprint;
use worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use worth_relational_contracts::internal_port as contracts;

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

pub(super) fn record_bulk_mutation_telemetry(
    instrumentation: &RuntimeInstrumentation,
    telemetry: &BulkMutationPlanTelemetry,
) {
    instrumentation.count_bulk_mutation_plan(
        &telemetry.locality,
        telemetry.normalized_client_key_count,
        telemetry.lineage_transition_count,
        telemetry.provenance_record_count,
    );
}

#[derive(Default)]
struct BulkMutationTelemetryAccumulator {
    touched_partitions:
        std::collections::BTreeSet<contracts::identity::data::primitives::PartitionId>,
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
            | MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(_))
            | MutationIntent::Entity(EntityMutationIntent::Revalidate(_)) => {
                self.entity_target_count += 1;
            }
            MutationIntent::Entity(EntityMutationIntent::Replace(_)) => {
                self.record_entity_creation(1, 1);
            }
            MutationIntent::Entity(EntityMutationIntent::Delete(_)) => {
                self.entity_target_count += 1;
                self.lineage_transition_count += 1;
            }
            MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(_))
            | MutationIntent::Relation(RelationMutationIntent::Revalidate(_)) => {
                self.relation_target_count += 1;
            }
            MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(_))
            | MutationIntent::Relation(RelationMutationIntent::Delete(_)) => {
                self.relation_target_count += 1;
                self.lineage_transition_count += 1;
            }
            MutationIntent::Materialization(intent) => match intent.record() {
                contracts::transactions::data::primitives::RecordRef::Entity(_) => {
                    self.entity_target_count += 1;
                }
                contracts::transactions::data::primitives::RecordRef::Relation(_) => {
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
