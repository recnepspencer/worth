//! Ordinary Native consequence mutation shares allocation, symbols and work custody.
use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::publication::ownership::read_request::{checkpoint, request};
use crate::contracts::transactions::data::aspect_field_patch::AspectFieldPatch;
use crate::contracts::transactions::data::intents::mutation_intent::{
    CreateIntent, DeleteEntityIntent, DeleteRelationIntent, EntityMutationIntent, MutationIntent,
    MutationIntentOwnerPort as _, RelationMutationIntent, UpdateEntityFieldsIntent,
};
use crate::contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use crate::contracts::transactions::data::outcomes::plan_artifacts::AuthoritativeApplyPlan;
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::contracts::transactions::data::primitives::{
    CreatedEntityRef, EntityReference, EntitySpec, RelationSpec,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelContext};
use worth_relational_branch::internal_port::branch::basis::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _,
};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBody;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;
const LINK: &str = crate::contracts::schema::current_publication_ownership::LINK;
pub mod fields;
pub struct CompletedCurrentRecords {
    pub effect: MutationEffect,
    pub funding: ExecutionMemoryReservation,
}
use crate::authority::mutation::{
    apply_plan_to_working_state, BranchLocalDeleteAllowance, MutationApplyDenial, MutationEffect,
};
use crate::contracts::symbols::data::client_key::ClientKey;
use std::collections::BTreeSet;

pub struct Writer<'a, 'request, 'scope, 'kernel, 'authority> {
    pub(crate) execution: &'a ExecutionRequest<'request, 'scope>,
    config: &'a crate::contracts::config::data::sections::StorageConfig,
    basis: &'a AdmittedRelationalBranchBasis,
    pub working: &'a mut WorkingState,
    identities: &'a mut super::pending_identities::PendingMutationIdentities,
    candidate: &'a CanonicalCommitEnvelopeBody,
    context: &'a mut MapKernelContext<'kernel, 'authority>,
    funding: ExecutionMemoryReservation,
    pub(crate) effect: MutationEffect,
}
impl<'a, 'request, 'scope, 'kernel, 'authority> Writer<'a, 'request, 'scope, 'kernel, 'authority> {
    pub fn finish(self) -> CompletedCurrentRecords {
        CompletedCurrentRecords {
            effect: self.effect,
            funding: self.funding,
        }
    }
    #[allow(clippy::too_many_arguments)]
    /// Mutate within the supplied current-ownership checkpoint. The enclosing
    /// preparation stage alone settles work; this writer receives no dispatcher.
    pub fn in_stage(
        execution: &'a ExecutionRequest<'request, 'scope>,
        config: &'a crate::contracts::config::data::sections::StorageConfig,
        basis: &'a AdmittedRelationalBranchBasis,
        working: &'a mut WorkingState,
        identities: &'a mut super::pending_identities::PendingMutationIdentities,
        candidate: &'a CanonicalCommitEnvelopeBody,
        context: &'a mut MapKernelContext<'kernel, 'authority>,
    ) -> Result<Self, TransactionCommitError> {
        let funding = execution
            .in_scope(|lease| ExecutionMemoryReservation::reserve_in_scope(lease, 0))
            .map_err(request)?
            .map_err(|cause| request(crate::contracts::execution::denial::lease_denial(cause)))?;
        Ok(Self {
            execution,
            config,
            basis,
            working,
            identities,
            candidate,
            context,
            funding,
            effect: Default::default(),
        })
    }
    pub fn context(&mut self) -> &mut MapKernelContext<'kernel, 'authority> {
        self.context
    }
    pub fn checkpoint(&mut self, units: usize) -> Result<(), TransactionCommitError> {
        checkpoint(self.context, units as u64)
    }
    pub fn retain(&mut self, bytes: u64) -> Result<(), TransactionCommitError> {
        let total = self.funding.bytes().checked_add(bytes).ok_or_else(|| {
            request(crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow)
        })?;
        self.funding.resize(total).map_err(|cause| {
            request(crate::contracts::execution::denial::lease_denial(
                worth_execution::LeaseDenial::MemoryExhausted(cause),
            ))
        })
    }
    pub(crate) fn current_links(
        &mut self,
        entity: EntityId,
        kind: KindId,
    ) -> Result<Vec<RelationId>, TransactionCommitError> {
        let view = crate::validation::engine::state_view::InvariantStateView::new(
            self.basis,
            self.candidate.commit.version_id,
        );
        view.outgoing_relations_for_entity_observed(entity, || self.checkpoint(1))
            .map(|links| {
                links
                    .into_iter()
                    .filter(|id| {
                        view.relation_metadata(*id)
                            .is_some_and(|row| row.kind_id == kind)
                    })
                    .collect()
            })
    }
    pub fn create_entity<T: serde::Serialize>(
        &mut self,
        kind: KindId,
        aspect: &str,
        publication: CommitId,
        value: &T,
        link: Option<(KindId, EntityId, CommitId)>,
    ) -> Result<EntityId, TransactionCommitError> {
        let bytes = self.encode(value)?;
        self.create_encoded_entity(kind, aspect, publication, bytes, link)
    }
    pub fn create_encoded_entity(
        &mut self,
        kind: KindId,
        aspect: &str,
        publication: CommitId,
        bytes: Vec<u8>,
        link: Option<(KindId, EntityId, CommitId)>,
    ) -> Result<EntityId, TransactionCommitError> {
        let partition = link
            .map(|(_, target, _)| target.partition_id)
            .ok_or_else(|| {
                crate::contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "ownership creation requires its publication source",
                )
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
        if let Some((link_kind, target, link_publication)) = link {
            intents.push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: partition,
                    kind_id: link_kind,
                    client_key: ClientKey::raw(LINK),
                    source: EntityReference::Created(created.clone()),
                    target: EntityReference::Existing(target),
                    fields: fields::encode(LINK, link_publication, Vec::new())?,
                },
            )));
        }
        let bindings = self.apply(intents)?;
        bindings.resolve(&created).ok_or_else(|| {
            crate::contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "Native current ownership allocation lost its binding",
            )
        })
    }
    pub fn create_link(
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
    pub fn update_entity<T: serde::Serialize>(
        &mut self,
        entity: EntityId,
        aspect: &str,
        publication: CommitId,
        value: &T,
    ) -> Result<(), TransactionCommitError> {
        let bytes = self.encode(value)?;
        self.update_encoded_entity(entity, aspect, publication, bytes)
    }
    pub fn update_encoded_entity(
        &mut self,
        entity: EntityId,
        aspect: &str,
        publication: CommitId,
        bytes: Vec<u8>,
    ) -> Result<(), TransactionCommitError> {
        self.apply(vec![MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: fields::encode(aspect, publication, bytes)?,
            }),
        )])
        .map(|_| ())
    }
    pub fn delete_entity(&mut self, entity: EntityId) -> Result<(), TransactionCommitError> {
        self.apply(vec![MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: entity },
        ))])
        .map(|_| ())
    }
    pub fn delete_relation(&mut self, relation: RelationId) -> Result<(), TransactionCommitError> {
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
                _ => return Err(
                    crate::contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "unsupported Native ownership consequence",
                    ),
                ),
            }
        }
        partitions.extend(entities.iter().map(|id| id.partition_id));
        partitions.extend(relations.iter().map(|id| id.partition_id));
        for partition in &partitions {
            self.basis
                .demand_head_partition(self.execution, *partition)
                .map_err(|cause| {
                    worth_relational_branch::internal_port::authority::commit::preparation::storage_denial::storage_read_denial(
                        self.basis, cause,
                    )
                })?;
        }
        let mut held = self.funding.bytes();
        let funding = &mut self.funding;
        self.working.materialize_consequence_records(
            self.basis,
            &partitions,
            &entities,
            &relations,
            |units, bytes| {
                checkpoint(self.context, units)?;
                let bytes = bytes.ok_or_else(|| {
                    request(crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow)
                })?;
                held = held.checked_add(bytes).ok_or_else(|| {
                    request(crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow)
                })?;
                funding.resize(held).map_err(|cause| {
                    request(crate::contracts::execution::denial::lease_denial(
                        worth_execution::LeaseDenial::MemoryExhausted(cause),
                    ))
                })
            },
        )?;
        let schema = self.basis.owner_inner().root.schema_authority();
        let config = crate::contracts::config::data::policies::MutationConfig {
            cascade_delete_policy: self.config.cascade_delete_policy,
            adjacency_policy: self.config.adjacency_policy.clone(),
            cross_context_policy: self.config.cross_context_policy,
        };
        let applied = apply_plan_to_working_state(
            self.execution,
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
            // The borrowed context belongs to the enclosing ownership stage.
            // Nested mutation reports charge that stage, never redispatch its account.
            None,
        )
        .map_err(|cause| match cause {
            MutationApplyDenial::Commit(cause) => cause,
            MutationApplyDenial::Symbols(cause) => {
                crate::contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    format!("Native ownership symbol admission: {cause:?}"),
                )
            }
        })?;
        crate::authority::mutation::apply_adjacency_deltas_for_request(
            self.execution,
            self.working,
            &applied.effect.adjacency.deltas,
            self.candidate.commit.version_id,
        )?;
        self.effect.accumulate(applied.effect);
        Ok(applied.created_entities)
    }
}
