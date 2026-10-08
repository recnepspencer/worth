//! Selected-policy typed validation dependencies publish only after complete admission.
use super::{Denial, RelationalTransactionFootprint, RelationalTransactionReadLocus};
use crate::identity::data::KindId;
use crate::transactions::data::{
    CreateIntent, EntityMutationIntent, EntityReference, MaterializationMutationIntent,
    MutationIntent, RecordRef, RelationMutationIntent,
};

impl RelationalTransactionFootprint {
    pub(crate) fn derive_validation_dependencies(
        &mut self,
        plan: &crate::transactions::data::MergedCommitPlan,
        maximum_loci: usize,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), Denial> {
        allocation_policy.check_live()?;
        let mut candidate = self.clone();
        for intent in &plan.merged_intents {
            allocation_policy.check_live()?;
            candidate.record_validation_intent(intent, allocation_policy)?;
        }
        let partitions = candidate.write_partitions.clone();
        for partition in &partitions {
            allocation_policy.check_live()?;
            candidate.record_read(
                RelationalTransactionReadLocus::ValidationPartition(*partition),
                allocation_policy,
            )?;
        }
        let required_loci = candidate.total_locus_count();
        if required_loci > maximum_loci {
            return Err(Denial::FootprintCapacityExhausted {
                maximum_loci,
                required_loci,
            });
        }
        allocation_policy.check_live()?;
        *self = candidate;
        Ok(())
    }

    fn record_validation_intent(
        &mut self,
        intent: &MutationIntent,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), Denial> {
        match intent {
            MutationIntent::Create(create) => {
                self.record_create_dependencies(create, allocation_policy)?
            }
            MutationIntent::Entity(entity) => {
                let (record, replacement_kind) = match entity {
                    EntityMutationIntent::UpdateFields(intent) => (intent.entity_id, None),
                    EntityMutationIntent::ApplyAspectPatch(intent) => (intent.entity_id, None),
                    EntityMutationIntent::Replace(intent) => {
                        (intent.entity_id, Some(intent.replacement.kind_id))
                    }
                    EntityMutationIntent::Delete(intent) => (intent.entity_id, None),
                    EntityMutationIntent::Revalidate(intent) => (intent.entity_id, None),
                };
                self.record_read(
                    RelationalTransactionReadLocus::Existing(RecordRef::Entity(record)),
                    allocation_policy,
                )?;
                if let Some(kind) = replacement_kind {
                    self.record_read(
                        RelationalTransactionReadLocus::EntitySchema(kind),
                        allocation_policy,
                    )?;
                }
            }
            MutationIntent::Relation(relation) => {
                let record = match relation {
                    RelationMutationIntent::UpdateEndpoints(intent) => {
                        self.record_read(
                            RelationalTransactionReadLocus::RelationSchema(intent.kind_id),
                            allocation_policy,
                        )?;
                        self.record_entity_reference(&intent.source, allocation_policy)?;
                        self.record_entity_reference(&intent.target, allocation_policy)?;
                        intent.relation_id
                    }
                    RelationMutationIntent::ApplyAspectPatch(intent) => intent.relation_id,
                    RelationMutationIntent::Delete(intent) => intent.relation_id,
                };
                self.record_read(
                    RelationalTransactionReadLocus::Existing(RecordRef::Relation(record)),
                    allocation_policy,
                )?;
            }
            MutationIntent::Materialization(intent) => {
                self.record_read(
                    RelationalTransactionReadLocus::Existing(intent.record()),
                    allocation_policy,
                )?;
                match intent {
                    MaterializationMutationIntent::RematerializeEntity(spec) => self.record_read(
                        RelationalTransactionReadLocus::EntitySchema(spec.kind_id),
                        allocation_policy,
                    )?,
                    MaterializationMutationIntent::RematerializeRelation(spec) => {
                        self.record_read(
                            RelationalTransactionReadLocus::RelationSchema(spec.kind_id),
                            allocation_policy,
                        )?;
                        self.record_read(
                            RelationalTransactionReadLocus::Existing(RecordRef::Entity(
                                spec.source,
                            )),
                            allocation_policy,
                        )?;
                        self.record_read(
                            RelationalTransactionReadLocus::Existing(RecordRef::Entity(
                                spec.target,
                            )),
                            allocation_policy,
                        )?;
                    }
                    MaterializationMutationIntent::SuspendEntity(_)
                    | MaterializationMutationIntent::SuspendRelation(_) => {}
                }
            }
        };
        Ok(())
    }

    fn record_create_dependencies(
        &mut self,
        create: &CreateIntent,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), Denial> {
        match create {
            CreateIntent::Entity(intent) => {
                self.record_read(
                    RelationalTransactionReadLocus::EntitySchema(intent.kind_id),
                    allocation_policy,
                )?;
            }
            CreateIntent::EntityAspects(intent) => {
                self.record_read(
                    RelationalTransactionReadLocus::EntitySchema(intent.kind_id),
                    allocation_policy,
                )?;
            }
            CreateIntent::BulkEntities(intent) => {
                self.record_read(
                    RelationalTransactionReadLocus::EntitySchema(intent.kind_id),
                    allocation_policy,
                )?;
            }
            CreateIntent::Relation(intent) => {
                self.record_relation_create(
                    intent.kind_id,
                    &intent.source,
                    &intent.target,
                    allocation_policy,
                )?;
            }
            CreateIntent::RelationAspects(intent) => {
                self.record_relation_create(
                    intent.kind_id,
                    &intent.source,
                    &intent.target,
                    allocation_policy,
                )?;
            }
            CreateIntent::BulkRelations(intent) => {
                self.record_read(
                    RelationalTransactionReadLocus::RelationSchema(intent.kind_id),
                    allocation_policy,
                )?;
                for (source, target) in &intent.endpoints {
                    allocation_policy.check_live()?;
                    self.record_entity_reference(source, allocation_policy)?;
                    self.record_entity_reference(target, allocation_policy)?;
                }
            }
        };
        Ok(())
    }

    fn record_relation_create(
        &mut self,
        kind: KindId,
        source: &EntityReference,
        target: &EntityReference,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), Denial> {
        self.record_read(
            RelationalTransactionReadLocus::RelationSchema(kind),
            allocation_policy,
        )?;
        self.record_entity_reference(source, allocation_policy)?;
        self.record_entity_reference(target, allocation_policy)?;
        Ok(())
    }

    fn record_entity_reference(
        &mut self,
        reference: &EntityReference,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), Denial> {
        let locus = match reference {
            EntityReference::Existing(entity) => {
                RelationalTransactionReadLocus::Existing(RecordRef::Entity(*entity))
            }
            EntityReference::Created(entity) => {
                RelationalTransactionReadLocus::CreatedEntity(entity.clone())
            }
        };
        self.record_read(locus, allocation_policy)?;
        Ok(())
    }
}
