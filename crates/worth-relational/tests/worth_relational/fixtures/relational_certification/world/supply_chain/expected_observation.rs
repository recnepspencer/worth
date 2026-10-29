pub(crate) use std::collections::{BTreeMap, BTreeSet};

pub(crate) use super::oracle::{OracleAncestry, OracleBranch};
pub(crate) use super::schema::{EntityRecord, RelationEdge, SchemaVersion, SupplyChainSchema};
pub(crate) use super::semantic_key::{EntityKey, RelationKey};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExpectedSupplyChainObservation {
    pub(crate) schema: SupplyChainSchema,
    pub(crate) entities: BTreeMap<EntityKey, EntityRecord>,
    pub(crate) relations: BTreeMap<RelationKey, RelationEdge>,
    pub(crate) absent_entities: BTreeSet<EntityKey>,
    pub(crate) absent_relations: BTreeSet<RelationKey>,
    pub(crate) ancestry: OracleAncestry,
}

impl ExpectedSupplyChainObservation {
    pub(crate) fn from_branch(branch: &OracleBranch) -> Self {
        Self {
            schema: branch.state.schema.clone(),
            entities: branch.state.entities.clone(),
            relations: branch.state.relations.clone(),
            absent_entities: branch.state.absent_entities.clone(),
            absent_relations: branch.state.absent_relations.clone(),
            ancestry: branch.ancestry.clone(),
        }
    }

    pub(crate) fn schema_version(&self) -> SchemaVersion {
        self.schema.version
    }
}
