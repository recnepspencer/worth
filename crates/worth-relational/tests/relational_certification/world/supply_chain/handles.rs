use std::collections::{BTreeMap, BTreeSet};

use super::program::CompiledSupplyChainProgram;
use super::semantic_key::{EntityKey, RelationKey};
use worth_relational::facade::history::RelationalCommitReceipt;
use worth_relational::facade::identity::{EntityId, KindId, RelationId};
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_relational::facade::transactions::{
    CommitResult, CreatedEntityRef, CreatedRelationRef, EntityReference, EntitySpec, RelationSpec,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EntityHandle {
    pub(crate) semantic: EntityKey,
    pub(crate) created: CreatedEntityRef,
    pub(crate) id: EntityId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RelationHandle {
    pub(crate) semantic: RelationKey,
    pub(crate) created: CreatedRelationRef,
    pub(crate) id: RelationId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BaselineBranchEnvelope {
    pub(crate) runtime_instance_id: u64,
    pub(crate) branch_id: worth_relational::facade::history::BranchId,
    pub(crate) commit: Option<RelationalCommitReceipt>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SupplyChainSemanticHandles {
    pub(crate) entities: BTreeMap<EntityKey, EntityHandle>,
    pub(crate) relations: BTreeMap<RelationKey, RelationHandle>,
    pub(crate) snapshot: SnapshotHandle,
    pub(crate) branch: BaselineBranchEnvelope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum HandleBindingError {
    ForeignRuntime {
        commit_runtime_instance_id: u64,
        snapshot_runtime_instance_id: u64,
    },
    MissingEntity(EntityKey),
    MissingEntitySpec(EntityKey),
    MissingRelation(RelationKey),
    MissingRelationSpec(RelationKey),
    DuplicateRelationReference(worth_relational::facade::symbols::ClientKey),
    DuplicateEntityIdentity(EntityId),
    DuplicateRelationIdentity(RelationId),
    WrongEntityKind {
        semantic: EntityKey,
        expected: KindId,
        observed: KindId,
    },
    WrongRelationKind {
        semantic: RelationKey,
        expected: KindId,
        observed: KindId,
    },
    WrongRelationEndpoints {
        semantic: RelationKey,
        expected_source: EntityReference,
        expected_target: EntityReference,
        observed_source: EntityReference,
        observed_target: EntityReference,
    },
}

impl SupplyChainSemanticHandles {
    pub(crate) fn aurora_voyage(&self) -> &EntityHandle {
        &self.entities
            [&super::semantic_key::EntityKey::new(super::semantic_key::EntityKind::Voyage, 0)]
    }

    pub(crate) fn medical_cargo(&self) -> &EntityHandle {
        &self.entities
            [&super::semantic_key::EntityKey::new(super::semantic_key::EntityKind::CargoLot, 0)]
    }

    pub(crate) fn rewire_port(&self) -> &EntityHandle {
        &self.entities
            [&super::semantic_key::EntityKey::new(super::semantic_key::EntityKind::Port, 3)]
    }

    pub(crate) fn bind(
        program: &CompiledSupplyChainProgram,
        commit: &CommitResult,
        snapshot: SnapshotHandle,
    ) -> Result<Self, HandleBindingError> {
        if commit.snapshot.runtime_instance_id() != snapshot.runtime_instance_id() {
            return Err(HandleBindingError::ForeignRuntime {
                commit_runtime_instance_id: commit.snapshot.runtime_instance_id(),
                snapshot_runtime_instance_id: snapshot.runtime_instance_id(),
            });
        }
        let entities = bind_entities(program, commit)?;
        let relations = bind_relations(program, commit)?;

        Ok(Self {
            entities,
            relations,
            branch: BaselineBranchEnvelope::from_snapshot(&snapshot, commit),
            snapshot,
        })
    }

    pub(crate) fn entity_key(&self, id: EntityId) -> Option<EntityKey> {
        self.entities
            .values()
            .find(|handle| handle.id == id)
            .map(|handle| handle.semantic)
    }

    pub(crate) fn relation_key(&self, id: RelationId) -> Option<RelationKey> {
        self.relations
            .values()
            .find(|handle| handle.id == id)
            .map(|handle| handle.semantic)
    }

    pub(crate) fn for_snapshot(&self, snapshot: SnapshotHandle) -> Self {
        let mut handles = self.clone();
        handles.branch = BaselineBranchEnvelope {
            runtime_instance_id: snapshot.runtime_instance_id(),
            branch_id: snapshot.branch_id().clone(),
            commit: self.branch.commit.clone(),
        };
        handles.snapshot = snapshot;
        handles
    }

    pub(crate) fn for_observation(
        &self,
        observation: &worth_relational::facade::branch::RelationalBranchObservation,
    ) -> Self {
        let mut handles = self.clone();
        handles.branch = BaselineBranchEnvelope {
            runtime_instance_id: observation.identity().runtime_instance_id(),
            branch_id: observation.identity().branch_id().clone(),
            commit: self.branch.commit.clone(),
        };
        handles
    }
}

fn bind_entities(
    program: &CompiledSupplyChainProgram,
    commit: &CommitResult,
) -> Result<BTreeMap<EntityKey, EntityHandle>, HandleBindingError> {
    let mut entities = BTreeMap::new();
    let mut ids = BTreeSet::new();
    for semantic in program.definition().entities.keys().copied() {
        let spec = spec_for_entity(program, semantic)
            .ok_or(HandleBindingError::MissingEntitySpec(semantic))?;
        let created = created_entity(spec);
        let Some(id) = commit.created_entity(&created) else {
            return Err(HandleBindingError::MissingEntity(semantic));
        };
        if !ids.insert(id) {
            return Err(HandleBindingError::DuplicateEntityIdentity(id));
        }
        let expected = super::program::entity_kind_id(semantic.kind);
        if spec.kind_id != expected {
            return Err(HandleBindingError::WrongEntityKind {
                semantic,
                expected,
                observed: spec.kind_id,
            });
        }
        entities.insert(
            semantic,
            EntityHandle {
                semantic,
                created,
                id,
            },
        );
    }
    Ok(entities)
}

fn bind_relations(
    program: &CompiledSupplyChainProgram,
    commit: &CommitResult,
) -> Result<BTreeMap<RelationKey, RelationHandle>, HandleBindingError> {
    let mut relations = BTreeMap::new();
    let mut ids = BTreeSet::new();
    let mut references = BTreeSet::new();
    for spec in program.relation_specs() {
        if !references.insert(spec.client_key.clone()) {
            return Err(HandleBindingError::DuplicateRelationReference(
                spec.client_key.clone(),
            ));
        }
    }
    for (semantic, edge) in &program.definition().relations {
        let spec = spec_for_relation(program, *semantic)
            .ok_or(HandleBindingError::MissingRelationSpec(*semantic))?;
        let expected_source = created_reference(edge.source);
        let expected_target = created_reference(edge.target);
        if spec.source != expected_source || spec.target != expected_target {
            return Err(HandleBindingError::WrongRelationEndpoints {
                semantic: *semantic,
                expected_source,
                expected_target,
                observed_source: spec.source.clone(),
                observed_target: spec.target.clone(),
            });
        }
        let expected = super::program::relation_kind_id(semantic.kind);
        if spec.kind_id != expected {
            return Err(HandleBindingError::WrongRelationKind {
                semantic: *semantic,
                expected,
                observed: spec.kind_id,
            });
        }
        let created = created_relation(spec);
        let Some(id) = commit.created_relation(&created) else {
            return Err(HandleBindingError::MissingRelation(*semantic));
        };
        if !ids.insert(id) {
            return Err(HandleBindingError::DuplicateRelationIdentity(id));
        }
        relations.insert(
            *semantic,
            RelationHandle {
                semantic: *semantic,
                created,
                id,
            },
        );
    }
    Ok(relations)
}

impl BaselineBranchEnvelope {
    fn from_snapshot(snapshot: &SnapshotHandle, commit: &CommitResult) -> Self {
        Self {
            runtime_instance_id: snapshot.runtime_instance_id(),
            branch_id: snapshot.branch_id().clone(),
            commit: Some(commit.commit.clone()),
        }
    }
}

fn spec_for_entity(program: &CompiledSupplyChainProgram, key: EntityKey) -> Option<&EntitySpec> {
    program
        .entity_specs()
        .iter()
        .find(|spec| spec.client_key == super::program::entity_client_key(key))
}

fn spec_for_relation(
    program: &CompiledSupplyChainProgram,
    key: RelationKey,
) -> Option<&RelationSpec> {
    program
        .relation_specs()
        .iter()
        .find(|spec| spec.client_key == super::program::relation_client_key(key))
}

fn created_reference(key: EntityKey) -> EntityReference {
    EntityReference::Created(CreatedEntityRef {
        partition_id: super::program::partition_for_entity_kind(key.kind),
        kind_id: super::program::entity_kind_id(key.kind),
        client_key: super::program::entity_client_key(key),
    })
}

fn created_entity(spec: &EntitySpec) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: spec.partition_id,
        kind_id: spec.kind_id,
        client_key: spec.client_key.clone(),
    }
}

fn created_relation(spec: &RelationSpec) -> CreatedRelationRef {
    CreatedRelationRef {
        partition_id: spec.partition_id,
        kind_id: spec.kind_id,
        client_key: spec.client_key.clone(),
        source: spec.source.clone(),
        target: spec.target.clone(),
    }
}
