//! Public bootstrap seed admission stays inside the installation owner.
use super::super::{WorthQueryApplicationEntitySeed, WorthQueryApplicationRelationSeed};
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphBootstrap<Schema> {
    pub fn bind_entity<Entity>(
        &mut self,
        seed: WorthQueryApplicationEntitySeed<Schema, Entity>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        self.typed_rows.bind_entity(
            &self.graph.layout,
            self.recovered_relational_authority.is_some(),
            seed,
        )
    }
    pub fn bind_relation<Relation, From, To>(
        &mut self,
        seed: WorthQueryApplicationRelationSeed<Schema, Relation, From, To>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        self.typed_rows.bind_relation(
            &self.graph.layout,
            self.recovered_relational_authority.is_some(),
            seed,
        )
    }
}
