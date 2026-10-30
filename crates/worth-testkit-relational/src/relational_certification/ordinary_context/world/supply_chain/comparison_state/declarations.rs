pub use {std::collections::BTreeMap, std::collections::BTreeSet};

pub use {crate::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::RelationEdge, crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError, crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservedSupplyChainState {
    pub schema: SchemaVersion,
    pub entities: BTreeMap<EntityKey, EntityRecord>,
    pub relations: BTreeMap<RelationKey, RelationEdge>,
    pub relation_vector: Vec<RelationEdge>,
    pub absent_entities: BTreeSet<EntityKey>,
    pub absent_relations: BTreeSet<RelationKey>,
    pub branch: BranchLabel,
    pub parent: Option<BranchLabel>,
    pub lineage: Vec<BranchLabel>,
    pub accepted: Vec<crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId>,
    pub history: Vec<crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AcceptedDelta>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComparisonMismatch {
    MissingEntity(EntityKey),
    MissingRelation(RelationKey),
    EntityValue(SemanticPath),
    RelationTarget(RelationKey),
    RelationSource(RelationKey),
    EntityAbsence(EntityKey),
    RelationAbsence(RelationKey),
    SiblingFactLeak {
        expected: BranchLabel,
        observed: BranchLabel,
    },
    FloatingBranchSelection(BranchLabel),
    WrongAncestry {
        expected: BranchLabel,
        observed: Option<BranchLabel>,
    },
    AcceptedDeltaOrder {
        expected: Vec<crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId>,
        observed: Vec<crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId>,
    },
    AcceptedHistory {
        expected: Vec<crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AcceptedDelta>,
        observed: Vec<crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AcceptedDelta>,
    },
    DuplicateRelation(RelationKey),
    MissingRelationVector(RelationKey),
    UnexpectedRelationVector(RelationKey),
    RelationVectorValue(RelationKey),
    IllegalEndpoint(SchemaError),
    SchemaMeaning {
        expected: SchemaVersion,
        observed: SchemaVersion,
    },
    UnexpectedEntity(EntityKey),
    UnexpectedRelation(RelationKey),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComparisonFailure {
    pub mismatch: ComparisonMismatch,
}

impl ObservedSupplyChainState {
    pub fn from_expected(expected: &ExpectedSupplyChainObservation) -> Self {
        Self {
            schema: expected.schema.version,
            entities: expected.entities.clone(),
            relations: expected.relations.clone(),
            relation_vector: expected.relations.values().copied().collect(),
            absent_entities: expected.absent_entities.clone(),
            absent_relations: expected.absent_relations.clone(),
            branch: expected.ancestry.branch,
            parent: expected.ancestry.parent,
            lineage: expected.ancestry.lineage.clone(),
            accepted: expected.ancestry.accepted.clone(),
            history: expected.ancestry.history.clone(),
        }
    }

    pub fn remove_entity(&mut self, key: EntityKey) {
        self.entities.remove(&key);
        self.absent_entities.insert(key);
    }

    pub fn remove_relation(&mut self, key: RelationKey) {
        self.relations.remove(&key);
        self.relation_vector.retain(|edge| edge.key != key);
        self.absent_relations.insert(key);
    }

    pub fn replace_relation(&mut self, edge: RelationEdge) {
        self.relations.insert(edge.key, edge);
        if let Some(current) = self
            .relation_vector
            .iter_mut()
            .find(|current| current.key == edge.key)
        {
            *current = edge;
        } else {
            self.relation_vector.push(edge);
        }
        self.absent_relations.remove(&edge.key);
    }

    pub fn duplicate_relation(&mut self, key: RelationKey) {
        if let Some(edge) = self.relations.get(&key).copied() {
            self.relation_vector.push(edge);
        }
    }

    pub fn remove_relation_from_vector(&mut self, key: RelationKey) {
        self.relation_vector.retain(|edge| edge.key != key);
    }

    pub fn append_relation_to_vector(&mut self, edge: RelationEdge) {
        self.relation_vector.push(edge);
    }

    pub fn repoint_relation_vector(
        &mut self,
        key: RelationKey,
        source: Option<EntityKey>,
        target: Option<EntityKey>,
    ) {
        if let Some(edge) = self.relation_vector.iter_mut().find(|edge| edge.key == key) {
            if let Some(source) = source {
                edge.source = source;
            }
            if let Some(target) = target {
                edge.target = target;
            }
        }
    }

    pub fn repoint_relation(&mut self, key: RelationKey, target: EntityKey) {
        if let Some(mut edge) = self.relations.get(&key).copied() {
            edge.target = target;
            self.replace_relation(edge);
        }
    }

    pub fn rebase_relation(&mut self, key: RelationKey, source: EntityKey) {
        if let Some(mut edge) = self.relations.get(&key).copied() {
            edge.source = source;
            self.replace_relation(edge);
        }
    }

    pub fn set_branch(&mut self, branch: BranchLabel) {
        self.branch = branch;
    }

    pub fn set_parent(&mut self, parent: Option<BranchLabel>) {
        self.parent = parent;
    }

    pub fn set_lineage(&mut self, lineage: Vec<BranchLabel>) {
        self.lineage = lineage;
    }

    pub fn set_accepted(
        &mut self,
        accepted: Vec<crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId>,
    ) {
        self.accepted = accepted;
    }

    pub fn set_history(
        &mut self,
        history: Vec<crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AcceptedDelta>,
    ) {
        self.history = history;
    }

    pub fn set_entity(&mut self, key: EntityKey, value: EntityRecord) {
        self.entities.insert(key, value);
        self.absent_entities.remove(&key);
    }
}
