//! Ordinary Native consequence mutation shares allocation, symbols and work custody.
use super::*;
use crate::authority::mutation::{
    apply_plan_to_working_state, BranchLocalDeleteAllowance, MutationApplyDenial, MutationEffect,
};
use crate::symbols::data::ClientKey;
use std::collections::BTreeSet;

pub(super) struct Writer<'a, 'request, 'scope, 'kernel, 'authority> {
    pub(super) execution: &'a ExecutionRequest<'request, 'scope>,
    runtime: &'a RelationalPreparationRuntime,
    basis: &'a AdmittedRelationalBranchBasis,
    pub(super) working: &'a mut WorkingState,
    identities:
        &'a mut crate::authority::commit::phases::applied_mutation::PendingMutationIdentities,
    candidate: &'a CanonicalCommitEnvelope,
    context: &'a mut MapKernelContext<'kernel, 'authority>,
    funding: ExecutionMemoryReservation,
    pub(super) effect: MutationEffect,
}
impl<'a, 'request, 'scope, 'kernel, 'authority> Writer<'a, 'request, 'scope, 'kernel, 'authority> {
    pub(super) fn finish(self) -> super::CompletedOwnership {
        super::CompletedOwnership {
            effect: self.effect,
            funding: self.funding,
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        execution: &'a ExecutionRequest<'request, 'scope>,
        runtime: &'a RelationalPreparationRuntime,
        basis: &'a AdmittedRelationalBranchBasis,
        working: &'a mut WorkingState,
        identities: &'a mut crate::authority::commit::phases::applied_mutation::PendingMutationIdentities,
        candidate: &'a CanonicalCommitEnvelope,
        context: &'a mut MapKernelContext<'kernel, 'authority>,
    ) -> Result<Self, TransactionCommitError> {
        let funding = execution
            .in_scope(|lease| ExecutionMemoryReservation::reserve_in_scope(lease, 0))
            .map_err(request)?
            .map_err(|cause| request(crate::execution::denial::lease_denial(cause)))?;
        Ok(Self {
            execution,
            runtime,
            basis,
            working,
            identities,
            candidate,
            context,
            funding,
            effect: Default::default(),
        })
    }
    pub(super) fn checkpoint(&mut self, units: usize) -> Result<(), TransactionCommitError> {
        checkpoint(self.context, units as u64)
    }
    pub(super) fn retain(&mut self, bytes: u64) -> Result<(), TransactionCommitError> {
        let total = self.funding.bytes().checked_add(bytes).ok_or_else(|| {
            request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
        })?;
        self.funding.resize(total).map_err(|cause| {
            request(crate::execution::denial::lease_denial(
                worth_execution::LeaseDenial::MemoryExhausted(cause),
            ))
        })
    }
    pub(super) fn create_entity<T: serde::Serialize>(
        &mut self,
        kind: KindId,
        aspect: &str,
        publication: CommitId,
        value: &T,
        link: Option<(KindId, EntityId)>,
    ) -> Result<EntityId, TransactionCommitError> {
        let bytes = self.encode(value)?;
        self.create_encoded_entity(kind, aspect, publication, bytes, link)
    }
    pub(super) fn create_encoded_entity(
        &mut self,
        kind: KindId,
        aspect: &str,
        publication: CommitId,
        bytes: Vec<u8>,
        link: Option<(KindId, EntityId)>,
    ) -> Result<EntityId, TransactionCommitError> {
        let partition = link.map(|(_, target)| target.partition_id).ok_or_else(|| {
            crate::mvcc::origin_denial("ownership creation requires its publication source")
        })?;
        let key = ClientKey::raw(aspect);
        let created = CreatedEntityRef {
            partition_id: partition,
            kind_id: kind,
            client_key: key.clone(),
        };
        let mut intents = vec![MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: partition,
            kind_id: kind,
            client_key: key,
            fields: fields::encode(aspect, publication, bytes)?,
        }))];
        if let Some((link_kind, target)) = link {
            intents.push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: partition,
                    kind_id: link_kind,
                    client_key: ClientKey::raw(LINK),
                    source: EntityReference::Created(created.clone()),
                    target: EntityReference::Existing(target),
                    fields: fields::encode(LINK, publication, Vec::new())?,
                },
            )));
        }
        let bindings = self.apply(intents)?;
        bindings.resolve(&created).ok_or_else(|| {
            crate::mvcc::origin_denial("Native current ownership allocation lost its binding")
        })
    }
    pub(super) fn create_link(
        &mut self,
        kind: KindId,
        publication: CommitId,
        source: EntityId,
        target: EntityId,
    ) -> Result<(), TransactionCommitError> {
        self.apply(vec![MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: source.partition_id,
                kind_id: kind,
                client_key: ClientKey::raw(LINK),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields: fields::encode(LINK, publication, Vec::new())?,
            },
        ))])
        .map(|_| ())
    }
    pub(super) fn update_entity<T: serde::Serialize>(
        &mut self,
        entity: EntityId,
        aspect: &str,
        publication: CommitId,
        value: &T,
    ) -> Result<(), TransactionCommitError> {
        let bytes = self.encode(value)?;
        self.apply(vec![MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: fields::encode(aspect, publication, bytes)?,
            }),
        )])
        .map(|_| ())
    }
    pub(super) fn delete_entity(&mut self, entity: EntityId) -> Result<(), TransactionCommitError> {
        self.apply(vec![MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: entity },
        ))])
        .map(|_| ())
    }
    pub(super) fn delete_relation(
        &mut self,
        relation: RelationId,
    ) -> Result<(), TransactionCommitError> {
        self.apply(vec![MutationIntent::Relation(
            RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id: relation,
            }),
        )])
        .map(|_| ())
    }
    fn apply(
        &mut self,
        intents: Vec<MutationIntent>,
    ) -> Result<CommitCreatedEntityBindings, TransactionCommitError> {
        self.checkpoint(intents.len())?;
        for intent in &intents {
            self.retain(
                std::mem::size_of::<MutationIntent>() as u64
                    + intent.owned_allocation_capacity_bytes(),
            )?;
        }
        let mut entities = BTreeSet::new();
        let mut relations = BTreeSet::new();
        let mut partitions = BTreeSet::new();
        for intent in &intents {
            match intent {
                MutationIntent::Create(CreateIntent::Entity(spec)) => {
                    partitions.insert(spec.partition_id);
                }
                MutationIntent::Create(CreateIntent::Relation(spec)) => {
                    partitions.insert(spec.partition_id);
                    for endpoint in [&spec.source, &spec.target] {
                        partitions.insert(endpoint.partition_id());
                        if let EntityReference::Existing(id) = endpoint {
                            entities.insert(*id);
                        }
                    }
                }
                MutationIntent::Entity(EntityMutationIntent::UpdateFields(spec)) => {
                    entities.insert(spec.entity_id);
                }
                MutationIntent::Entity(EntityMutationIntent::Delete(spec)) => {
                    entities.insert(spec.entity_id);
                }
                MutationIntent::Relation(RelationMutationIntent::Delete(spec)) => {
                    relations.insert(spec.relation_id);
                }
                _ => {
                    return Err(crate::mvcc::origin_denial(
                        "unsupported Native ownership consequence",
                    ))
                }
            }
        }
        partitions.extend(entities.iter().map(|id| id.partition_id));
        partitions.extend(relations.iter().map(|id| id.partition_id));
        for partition in &partitions {
            self.basis
                .inner
                .root
                .demand_head_partition(self.execution, *partition)
                .map_err(|cause| {
                    crate::authority::commit::preparation::storage_read_denial(self.basis, cause)
                })?;
        }
        let mut held = self.funding.bytes();
        let funding = &mut self.funding;
        self.working.materialize_consequence_records(
            self.basis.inner.root.as_ref(),
            &partitions,
            &entities,
            &relations,
            |units, bytes| {
                checkpoint(self.context, units)?;
                let bytes = bytes.ok_or_else(|| {
                    request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
                })?;
                held = held.checked_add(bytes).ok_or_else(|| {
                    request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
                })?;
                funding.resize(held).map_err(|cause| {
                    request(crate::execution::denial::lease_denial(
                        worth_execution::LeaseDenial::MemoryExhausted(cause),
                    ))
                })
            },
        )?;
        let schema = self.basis.inner.root.schema_authority();
        let config = crate::config::data::MutationConfig {
            cascade_delete_policy: self.runtime.config.storage.cascade_delete_policy,
            adjacency_policy: self.runtime.config.storage.adjacency_policy.clone(),
            cross_context_policy: self.runtime.config.storage.cross_context_policy,
        };
        let applied = apply_plan_to_working_state(
            self.working,
            &AuthoritativeApplyPlan {
                transaction_id: self.candidate.merged_plan.transaction_id,
                version_id: self.candidate.commit.version_id,
                merged_intents: intents,
            },
            &config,
            schema.registry(),
            schema.aspect_plans(),
            &mut self.identities.symbols,
            BranchLocalDeleteAllowance::default(),
            &mut self.identities.records,
            crate::authority::commit::pipeline::request_lease(self.execution),
            self.runtime.commit_work_budget.clone(),
        )
        .map_err(|cause| match cause {
            MutationApplyDenial::Commit(cause) => cause,
            MutationApplyDenial::Symbols(cause) => {
                crate::mvcc::origin_denial(format!("Native ownership symbol admission: {cause:?}"))
            }
        })?;
        crate::authority::mutation::apply_adjacency_deltas(
            self.working,
            &applied.effect.adjacency.deltas,
            self.candidate.commit.version_id,
        );
        self.effect.accumulate(applied.effect);
        Ok(applied.created_entities)
    }
}
