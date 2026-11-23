//! Installed index registration and schema routing.
use crate::indexes::data::DerivedIndexDefinition;
use crate::runtime::RelationalRuntime;

pub struct IndexAuthority<'runtime> {
    pub(super) runtime: &'runtime RelationalRuntime,
}

impl RelationalRuntime {
    pub fn index_authority(&self) -> IndexAuthority<'_> {
        IndexAuthority::new(self)
    }
}

impl<'runtime> IndexAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    /// Privately prepare an append-only catalog on this actual receiver.
    pub fn prepare_home_program_definitions(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::runtime::RelationalIndexDefinitionPreparation,
        crate::runtime::RelationalIndexDefinitionPreparationDenial,
    > {
        crate::runtime::RelationalIndexDefinitionPreparation::capture(
            self.runtime.runtime_instance_id(),
            self.runtime.indexes.catalog_binding(),
            request,
        )
    }

    pub fn register(
        &self,
        definition: DerivedIndexDefinition,
    ) -> Result<DerivedIndexDefinition, crate::indexes::data::DerivedIndexRegistrationDenial> {
        let definition = self.runtime.indexes.register_definition(definition)?;
        self.install_known_schema_routing();
        Ok(definition)
    }

    /// cost-lane: installation/recovery. Compile coverage for installed head schemas.
    pub(crate) fn recover_installed_index_routing(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        self.install_known_schema_routing();
        Ok(())
    }

    fn install_known_schema_routing(&self) {
        use crate::capabilities::AspectPlanSource;
        self.runtime.indexes.install_routing(
            self.runtime.schema_contract_runtime.schema_authority_digest,
            self.runtime.aspect_plan_catalog(),
        );
        for root in self.runtime.history.current_root_storage_inventory() {
            self.runtime.indexes.install_routing(
                root.schema_authority().authority_digest(),
                root.schema_authority().aspect_plans(),
            );
        }
    }
}
