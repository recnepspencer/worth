pub use {std::collections::BTreeMap, std::collections::BTreeSet};

pub use {crate::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey};
pub use {worth_relational::facade::history::RelationalCommitReceipt};
pub use {worth_relational::facade::identity::EntityId, worth_relational::facade::identity::KindId, worth_relational::facade::identity::RelationId};
pub use {worth_relational::facade::snapshots::SnapshotHandle};
pub use {worth_relational::facade::transactions::CommitResult, worth_relational::facade::transactions::CreatedEntityRef, worth_relational::facade::transactions::CreatedRelationRef, worth_relational::facade::transactions::EntityReference, worth_relational::facade::transactions::EntitySpec, worth_relational::facade::transactions::RelationSpec};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityHandle {
    pub semantic: EntityKey,
    pub created: CreatedEntityRef,
    pub id: EntityId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationHandle {
    pub semantic: RelationKey,
    pub created: CreatedRelationRef,
    pub id: RelationId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaselineBranchEnvelope {
    pub runtime_instance_id: u64,
    pub branch_id: worth_relational::facade::history::BranchId,
    pub commit: Option<RelationalCommitReceipt>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplyChainSemanticHandles {
    pub entities: BTreeMap<EntityKey, EntityHandle>,
    pub relations: BTreeMap<RelationKey, RelationHandle>,
    pub snapshot: SnapshotHandle,
    pub branch: BaselineBranchEnvelope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HandleBindingError {
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
    pub fn aurora_voyage(&self) -> &EntityHandle {
        &self.entities
            [&crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, 0)]
    }

    pub fn medical_cargo(&self) -> &EntityHandle {
        &self.entities
            [&crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot, 0)]
    }

    pub fn rewire_port(&self) -> &EntityHandle {
        &self.entities
            [&crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 3)]
    }

    pub fn bind(
        program: &CompiledSupplyChainProgram,
        commit: &CommitResult,
        snapshot: SnapshotHandle,
    ) -> Result<Self, HandleBindingError> {
        if commit.snapshot.runtime_instance_id() != snapshot.runtime_instance_id() {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::ForeignRuntime {
                commit_runtime_instance_id: commit.snapshot.runtime_instance_id(),
                snapshot_runtime_instance_id: snapshot.runtime_instance_id(),
            });
        }
        let entities = bind_entities(program, commit)?;
        let relations = bind_relations(program, commit)?;

        Ok(Self {
            entities,
            relations,
            branch: crate::relational_certification::ordinary_context::world::supply_chain::handles::BaselineBranchEnvelope::from_snapshot(&snapshot, commit),
            snapshot,
        })
    }

    pub fn entity_key(&self, id: EntityId) -> Option<EntityKey> {
        self.entities
            .values()
            .find(|handle| handle.id == id)
            .map(|handle| handle.semantic)
    }

    pub fn relation_key(&self, id: RelationId) -> Option<RelationKey> {
        self.relations
            .values()
            .find(|handle| handle.id == id)
            .map(|handle| handle.semantic)
    }

    pub fn for_snapshot(&self, snapshot: SnapshotHandle) -> Self {
        let mut handles = self.clone();
        handles.branch = BaselineBranchEnvelope {
            runtime_instance_id: snapshot.runtime_instance_id(),
            branch_id: snapshot.branch_id().clone(),
            commit: self.branch.commit.clone(),
        };
        handles.snapshot = snapshot;
        handles
    }

    pub fn for_observation(
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

pub fn bind_entities(
    program: &CompiledSupplyChainProgram,
    commit: &CommitResult,
) -> Result<BTreeMap<EntityKey, EntityHandle>, HandleBindingError> {
    let mut entities = std::collections::BTreeMap::new();
    let mut ids = std::collections::BTreeSet::new();
    for semantic in program.definition().entities.keys().copied() {
        let spec = spec_for_entity(program, semantic)
            .ok_or(crate::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::MissingEntitySpec(semantic))?;
        let created = created_entity(spec);
        let Some(id) = commit.created_entity(&created) else {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::MissingEntity(semantic));
        };
        if !ids.insert(id) {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::DuplicateEntityIdentity(id));
        }
        let expected = crate::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id(semantic.kind);
        if spec.kind_id != expected {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::WrongEntityKind {
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

pub fn bind_relations(
    program: &CompiledSupplyChainProgram,
    commit: &CommitResult,
) -> Result<BTreeMap<RelationKey, RelationHandle>, HandleBindingError> {
    let mut relations = std::collections::BTreeMap::new();
    let mut ids = std::collections::BTreeSet::new();
    let mut references = std::collections::BTreeSet::new();
    for spec in program.relation_specs() {
        if !references.insert(spec.client_key.clone()) {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::DuplicateRelationReference(
                spec.client_key.clone(),
            ));
        }
    }
    for (semantic, edge) in &program.definition().relations {
        let spec = spec_for_relation(program, *semantic)
            .ok_or(crate::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::MissingRelationSpec(*semantic))?;
        let expected_source = created_reference(edge.source);
        let expected_target = created_reference(edge.target);
        if spec.source != expected_source || spec.target != expected_target {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::WrongRelationEndpoints {
                semantic: *semantic,
                expected_source,
                expected_target,
                observed_source: spec.source.clone(),
                observed_target: spec.target.clone(),
            });
        }
        let expected = crate::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::relation_kind_id(semantic.kind);
        if spec.kind_id != expected {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::WrongRelationKind {
                semantic: *semantic,
                expected,
                observed: spec.kind_id,
            });
        }
        let created = created_relation(spec);
        let Some(id) = commit.created_relation(&created) else {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::MissingRelation(*semantic));
        };
        if !ids.insert(id) {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::DuplicateRelationIdentity(id));
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
    pub fn from_snapshot(snapshot: &SnapshotHandle, commit: &CommitResult) -> Self {
        Self {
            runtime_instance_id: snapshot.runtime_instance_id(),
            branch_id: snapshot.branch_id().clone(),
            commit: Some(commit.commit.clone()),
        }
    }
}

pub fn spec_for_entity(
    program: &CompiledSupplyChainProgram,
    key: EntityKey,
) -> Option<&EntitySpec> {
    program
        .entity_specs()
        .iter()
        .find(|spec| spec.client_key == crate::relational_certification::ordinary_context::world::supply_chain::program::entity_client_key(key))
}

pub fn spec_for_relation(
    program: &CompiledSupplyChainProgram,
    key: RelationKey,
) -> Option<&RelationSpec> {
    program
        .relation_specs()
        .iter()
        .find(|spec| spec.client_key == crate::relational_certification::ordinary_context::world::supply_chain::program::relation_client_key(key))
}

pub fn created_reference(key: EntityKey) -> EntityReference {
    worth_relational::facade::transactions::EntityReference::Created(CreatedEntityRef {
        partition_id: crate::relational_certification::ordinary_context::world::supply_chain::program::partition_for_entity_kind(key.kind),
        kind_id: crate::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id(key.kind),
        client_key: crate::relational_certification::ordinary_context::world::supply_chain::program::entity_client_key(key),
    })
}

pub fn created_entity(spec: &EntitySpec) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: spec.partition_id,
        kind_id: spec.kind_id,
        client_key: spec.client_key.clone(),
    }
}

pub fn created_relation(spec: &RelationSpec) -> CreatedRelationRef {
    CreatedRelationRef {
        partition_id: spec.partition_id,
        kind_id: spec.kind_id,
        client_key: spec.client_key.clone(),
        source: spec.source.clone(),
        target: spec.target.clone(),
    }
}