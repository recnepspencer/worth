pub use {std::collections::BTreeMap, std::collections::BTreeSet};

pub use {crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleAncestry, crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::RelationEdge, crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, crate::relational_certification::ordinary_context::world::supply_chain::schema::SupplyChainSchema};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedSupplyChainObservation {
    pub schema: SupplyChainSchema,
    pub entities: BTreeMap<EntityKey, EntityRecord>,
    pub relations: BTreeMap<RelationKey, RelationEdge>,
    pub absent_entities: BTreeSet<EntityKey>,
    pub absent_relations: BTreeSet<RelationKey>,
    pub ancestry: OracleAncestry,
}

impl ExpectedSupplyChainObservation {
    pub fn from_branch(branch: &OracleBranch) -> Self {
        Self {
            schema: branch.state.schema.clone(),
            entities: branch.state.entities.clone(),
            relations: branch.state.relations.clone(),
            absent_entities: branch.state.absent_entities.clone(),
            absent_relations: branch.state.absent_relations.clone(),
            ancestry: branch.ancestry.clone(),
        }
    }

    pub fn schema_version(&self) -> SchemaVersion {
        self.schema.version
    }
}