pub(crate) use std::collections::{BTreeMap, BTreeSet};

pub(crate) use super::expected_observation::ExpectedSupplyChainObservation;
pub(crate) use super::schema::{EntityRecord, RelationEdge, SchemaError, SchemaVersion};
pub(crate) use super::semantic_key::{BranchLabel, EntityKey, RelationKey, SemanticPath};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObservedSupplyChainState {
    pub(crate) schema: SchemaVersion,
    pub(crate) entities: BTreeMap<EntityKey, EntityRecord>,
    pub(crate) relations: BTreeMap<RelationKey, RelationEdge>,
    pub(crate) relation_vector: Vec<RelationEdge>,
    pub(crate) absent_entities: BTreeSet<EntityKey>,
    pub(crate) absent_relations: BTreeSet<RelationKey>,
    pub(crate) branch: BranchLabel,
    pub(crate) parent: Option<BranchLabel>,
    pub(crate) lineage: Vec<BranchLabel>,
    pub(crate) accepted: Vec<super::scenario_delta_vocabulary::DeltaId>,
    pub(crate) history: Vec<super::oracle::AcceptedDelta>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ComparisonMismatch {
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
        expected: Vec<super::scenario_delta_vocabulary::DeltaId>,
        observed: Vec<super::scenario_delta_vocabulary::DeltaId>,
    },
    AcceptedHistory {
        expected: Vec<super::oracle::AcceptedDelta>,
        observed: Vec<super::oracle::AcceptedDelta>,
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
pub(crate) struct ComparisonFailure {
    pub(crate) mismatch: ComparisonMismatch,
}

impl ObservedSupplyChainState {
    pub(crate) fn from_expected(expected: &ExpectedSupplyChainObservation) -> Self {
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

    pub(crate) fn remove_entity(&mut self, key: EntityKey) {
        self.entities.remove(&key);
        self.absent_entities.insert(key);
    }

    pub(crate) fn remove_relation(&mut self, key: RelationKey) {
        self.relations.remove(&key);
        self.relation_vector.retain(|edge| edge.key != key);
        self.absent_relations.insert(key);
    }

    pub(crate) fn replace_relation(&mut self, edge: RelationEdge) {
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

    pub(crate) fn duplicate_relation(&mut self, key: RelationKey) {
        if let Some(edge) = self.relations.get(&key).copied() {
            self.relation_vector.push(edge);
        }
    }

    pub(crate) fn remove_relation_from_vector(&mut self, key: RelationKey) {
        self.relation_vector.retain(|edge| edge.key != key);
    }

    pub(crate) fn append_relation_to_vector(&mut self, edge: RelationEdge) {
        self.relation_vector.push(edge);
    }

    pub(crate) fn repoint_relation_vector(
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

    pub(crate) fn repoint_relation(&mut self, key: RelationKey, target: EntityKey) {
        if let Some(mut edge) = self.relations.get(&key).copied() {
            edge.target = target;
            self.replace_relation(edge);
        }
    }

    pub(crate) fn rebase_relation(&mut self, key: RelationKey, source: EntityKey) {
        if let Some(mut edge) = self.relations.get(&key).copied() {
            edge.source = source;
            self.replace_relation(edge);
        }
    }

    pub(crate) fn set_branch(&mut self, branch: BranchLabel) {
        self.branch = branch;
    }

    pub(crate) fn set_parent(&mut self, parent: Option<BranchLabel>) {
        self.parent = parent;
    }

    pub(crate) fn set_lineage(&mut self, lineage: Vec<BranchLabel>) {
        self.lineage = lineage;
    }

    pub(crate) fn set_accepted(
        &mut self,
        accepted: Vec<super::scenario_delta_vocabulary::DeltaId>,
    ) {
        self.accepted = accepted;
    }

    pub(crate) fn set_history(&mut self, history: Vec<super::oracle::AcceptedDelta>) {
        self.history = history;
    }

    pub(crate) fn set_entity(&mut self, key: EntityKey, value: EntityRecord) {
        self.entities.insert(key, value);
        self.absent_entities.remove(&key);
    }
}
