//! A successor round's creates take its chain's partitions (#119 §2.3 step 4).
//!
//! The round's attempt issues its own mutation partition. A key any round of
//! the chain created keeps the partition the ledger gave it, so its identity
//! is reused. The round's own known keys reveal its attempt partition and the
//! chain's: a new key authored in the round's attempt partition takes the
//! chain's; any other new key keeps the partition its producer chose. A
//! replacement of an entity the chain already replaced takes that
//! replacement's partition. One bulk create cannot span partitions.
use super::super::succession::ledger::ChainCreateLedger;
use crate::mvcc::publication::application_publication::custody;
use contracts::execution::request_stage::checkpoint;
use contracts::identity::data::primitives::PartitionId;
use contracts::lineage::data::{CommitNamingPlan, PendingBirthTarget};
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget;
use contracts::symbols::data::client_key::ClientKey;
use contracts::symbols::data::string_interner::StringInterner;
use contracts::transactions::data::intents::mutation_intent::{
    CreateIntent, EntityMutationIntent, MutationIntent,
};
use contracts::transactions::data::outcomes::transaction_commit_error::{
    origin_denial, TransactionCommitError,
};
use contracts::transactions::data::primitives::{EntityReference, WorkerIntentBatch};
use std::collections::BTreeMap;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port as contracts;

struct Partitions<'a> {
    ledger: &'a ChainCreateLedger,
    symbols: &'a StringInterner,
    /// The round's attempt partition to the chain's, from its known keys.
    attempt: BTreeMap<PartitionId, PartitionId>,
}

impl Partitions<'_> {
    fn text<'k>(&'k self, key: &'k ClientKey) -> Result<&'k str, TransactionCommitError> {
        key.resolve_with_interner(self.symbols)
            .ok_or_else(|| origin_denial("coupled local key has no actual Native symbol meaning"))
    }

    fn known(
        &self,
        relation: bool,
        key: &ClientKey,
    ) -> Result<Option<PartitionId>, TransactionCommitError> {
        Ok(self.ledger.partition(relation, self.text(key)?))
    }

    fn of(
        &self,
        relation: bool,
        key: &ClientKey,
        authored: PartitionId,
    ) -> Result<PartitionId, TransactionCommitError> {
        Ok(match self.known(relation, key)? {
            Some(partition) => partition,
            None => self.attempt.get(&authored).copied().unwrap_or(authored),
        })
    }

    fn bulk(
        &self,
        relation: bool,
        keys: &[ClientKey],
        authored: PartitionId,
    ) -> Result<PartitionId, TransactionCommitError> {
        let mut chosen = None;
        for key in keys {
            let partition = self.of(relation, key, authored)?;
            if chosen.is_some_and(|chosen| chosen != partition) {
                return Err(origin_denial("coupled bulk create spans partitions"));
            }
            chosen = Some(partition);
        }
        Ok(chosen.unwrap_or(authored))
    }

    fn reference(&self, reference: &mut EntityReference) -> Result<(), TransactionCommitError> {
        if let EntityReference::Created(created) = reference {
            created.partition_id = self.of(false, &created.client_key, created.partition_id)?;
        }
        Ok(())
    }
}

/// O(round creates + references + plan targets).
pub fn repartition(
    execution: &ExecutionRequest<'_, '_>,
    ledger: &ChainCreateLedger,
    symbols: &StringInterner,
    batches: &mut [WorkerIntentBatch],
    roles: &mut [RequiredOutputRoleTarget],
    naming: Option<&mut CommitNamingPlan>,
) -> Result<(), TransactionCommitError> {
    let mut partitions = Partitions {
        ledger,
        symbols,
        attempt: BTreeMap::new(),
    };
    execution
        .run_native_with_checkpoint(|context| {
            let mut charge = |units: usize| {
                checkpoint(context, units as u64 + 1).map_err(custody::execution_stop)
            };
            for intent in batches.iter().flat_map(|batch| &batch.intents) {
                if let MutationIntent::Entity(EntityMutationIntent::Replace(spec)) = intent {
                    charge(1)?;
                    if let Some(chain) = ledger.replacement_partition(spec.entity_id) {
                        let authored = spec.replacement.partition_id;
                        partitions.attempt.entry(authored).or_insert(chain);
                    }
                    continue;
                }
                let MutationIntent::Create(create) = intent else {
                    continue;
                };
                let keys: Vec<(bool, &ClientKey, PartitionId)> = match create {
                    CreateIntent::Entity(spec) => {
                        vec![(false, &spec.client_key, spec.partition_id)]
                    }
                    CreateIntent::EntityAspects(spec) => {
                        vec![(false, &spec.client_key, spec.partition_id)]
                    }
                    CreateIntent::BulkEntities(spec) => spec
                        .client_keys
                        .iter()
                        .map(|key| (false, key, spec.partition_id))
                        .collect(),
                    CreateIntent::Relation(spec) => {
                        vec![(true, &spec.client_key, spec.partition_id)]
                    }
                    CreateIntent::RelationAspects(spec) => {
                        vec![(true, &spec.client_key, spec.partition_id)]
                    }
                    CreateIntent::BulkRelations(spec) => spec
                        .client_keys
                        .iter()
                        .map(|key| (true, key, spec.partition_id))
                        .collect(),
                };
                charge(keys.len())?;
                for (relation, key, authored) in keys {
                    if let Some(chain) = partitions.known(relation, key)? {
                        partitions.attempt.entry(authored).or_insert(chain);
                    }
                }
            }
            for intent in batches.iter_mut().flat_map(|batch| &mut batch.intents) {
                charge(1)?;
                if let MutationIntent::Entity(EntityMutationIntent::Replace(spec)) = intent {
                    let authored = spec.replacement.partition_id;
                    spec.replacement.partition_id = ledger
                        .replacement_partition(spec.entity_id)
                        .or_else(|| partitions.attempt.get(&authored).copied())
                        .unwrap_or(authored);
                }
                if let MutationIntent::Create(create) = intent {
                    match create {
                        CreateIntent::Entity(spec) => {
                            spec.partition_id =
                                partitions.of(false, &spec.client_key, spec.partition_id)?
                        }
                        CreateIntent::EntityAspects(spec) => {
                            spec.partition_id =
                                partitions.of(false, &spec.client_key, spec.partition_id)?
                        }
                        CreateIntent::BulkEntities(spec) => {
                            spec.partition_id =
                                partitions.bulk(false, &spec.client_keys, spec.partition_id)?
                        }
                        CreateIntent::Relation(spec) => {
                            spec.partition_id =
                                partitions.of(true, &spec.client_key, spec.partition_id)?
                        }
                        CreateIntent::RelationAspects(spec) => {
                            spec.partition_id =
                                partitions.of(true, &spec.client_key, spec.partition_id)?
                        }
                        CreateIntent::BulkRelations(spec) => {
                            spec.partition_id =
                                partitions.bulk(true, &spec.client_keys, spec.partition_id)?
                        }
                    }
                }
                intent.visit_entity_references_mut(|reference| partitions.reference(reference))?;
            }
            for role in roles {
                charge(1)?;
                partitions.reference(&mut role.target)?;
            }
            if let Some(plan) = naming {
                super::super::part_naming::visit_targets_mut(plan, |target| {
                    charge(1)?;
                    match target {
                        PendingBirthTarget::CreatedEntity(created) => {
                            created.partition_id =
                                partitions.of(false, &created.client_key, created.partition_id)?;
                        }
                        PendingBirthTarget::CreatedRelation(created) => {
                            partitions.reference(&mut created.source)?;
                            partitions.reference(&mut created.target)?;
                            created.partition_id =
                                partitions.of(true, &created.client_key, created.partition_id)?;
                        }
                    }
                    Ok::<_, TransactionCommitError>(())
                })?;
            }
            Ok::<_, TransactionCommitError>(())
        })
        .map_err(custody::execution_stop)?
        .0
}
