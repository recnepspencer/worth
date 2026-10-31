//! Typed create effects for a sealed, pre-World checkpoint installation.
use super::{denial, WorthQueryCheckpointTransitionResources};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntitySeed, WorthQueryApplicationRelationSeed,
    WorthQueryPrimaryGraphInstallationDenial,
};
use worth_query_installation::facade::ApplicationSchema;

/// Authors new typed records in the same candidate as the program transition.
/// This stage cannot commit, expose a World, or mutate an existing record.
/// Links currently require endpoints created in this migration batch.
pub struct WorthQueryCheckpointMigrationWriter<'installation, Schema> {
    pub(super) graph:
        &'installation mut super::super::prepared_native::PreparedPrimaryGraphInstallation,
    pub(super) resources: WorthQueryCheckpointTransitionResources,
    work: usize,
    bytes: usize,
    failed: bool,
    _schema: std::marker::PhantomData<fn() -> Schema>,
}

impl<'installation, Schema: ApplicationSchema>
    WorthQueryCheckpointMigrationWriter<'installation, Schema>
{
    pub(super) fn new(
        graph: &'installation mut super::super::prepared_native::PreparedPrimaryGraphInstallation,
        resources: WorthQueryCheckpointTransitionResources,
    ) -> Self {
        Self {
            graph,
            resources,
            work: 0,
            bytes: 0,
            failed: false,
            _schema: std::marker::PhantomData,
        }
    }

    pub fn bind_entity<Entity>(
        &mut self,
        seed: WorthQueryApplicationEntitySeed<Schema, Entity>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        self.charge(seed.migration_cost())?;
        let result = self.graph.typed_rows.bind_entity(
            &self.graph.graph.layout,
            self.graph.recovered_relational_authority.is_some(),
            seed,
        );
        self.failed |= result.is_err();
        result
    }

    pub fn bind_relation<Relation, From, To>(
        &mut self,
        seed: WorthQueryApplicationRelationSeed<Schema, Relation, From, To>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        self.charge(seed.migration_cost())?;
        let result = self.graph.typed_rows.bind_relation(
            &self.graph.graph.layout,
            self.graph.recovered_relational_authority.is_some(),
            seed,
        );
        self.failed |= result.is_err();
        result
    }

    fn charge(
        &mut self,
        (work, bytes): (usize, usize),
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        self.work = self.work.saturating_add(work);
        self.bytes = self.bytes.saturating_add(bytes);
        if self.failed
            || self.work > self.resources.maximum_authored_units
            || self.bytes > self.resources.maximum_authored_bytes
        {
            self.failed = true;
            return Err(denial(
                "checkpoint migration authoring resources exceeded or previously refused",
            ));
        }
        Ok(())
    }

    pub(super) fn finish(self) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if self.failed {
            Err(denial("checkpoint migration authoring previously refused"))
        } else {
            Ok(())
        }
    }
}
