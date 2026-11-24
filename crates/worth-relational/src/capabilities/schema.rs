use crate::config::data::RelationalRuntimeConfig;
use crate::runtime::RelationalRuntime;
use crate::schema::data::{
    AspectContractPlanCatalog, LoweredAspectContractPlan, RelationalSchemaRegistry,
};
#[cfg(test)]
use crate::schema::data::{AspectLoweringTrace, LoweredRelationIntegrityPlan};

pub(crate) trait SchemaSource {
    fn schema_registry(&self) -> &RelationalSchemaRegistry;
}

pub(crate) use crate::schema::aspect_plan_source::AspectPlanSource;

/// Schema truth is read from the configuration this handle holds in force, so
/// registry and contract runtime answers always come from the same installed
/// configuration even while the owner installs another one.
impl SchemaSource for RelationalRuntime {
    fn schema_registry(&self) -> &RelationalSchemaRegistry {
        &self.config.schema.registry
    }
}

impl AspectPlanSource for RelationalRuntime {
    fn aspect_plan_catalog(&self) -> &AspectContractPlanCatalog {
        &self.schema_contract_runtime.aspect_contract_plans
    }

    fn entity_aspect_plan(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        self.schema_contract_runtime
            .aspect_contract_plans
            .entity_plans
            .get(&kind_id)
    }

    fn relation_aspect_plan(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        self.schema_contract_runtime
            .aspect_contract_plans
            .relation_plans
            .get(&kind_id)
    }
}

impl SchemaSource for RelationalSchemaRegistry {
    fn schema_registry(&self) -> &RelationalSchemaRegistry {
        self
    }
}

impl SchemaSource for crate::branch::RelationalBranchRootSchemaAuthority {
    fn schema_registry(&self) -> &RelationalSchemaRegistry {
        self.registry()
    }
}

impl SchemaSource for RelationalRuntimeConfig {
    fn schema_registry(&self) -> &RelationalSchemaRegistry {
        &self.schema.registry
    }
}

impl RelationalRuntime {
    #[cfg(test)]
    pub(crate) fn entity_aspect_plan_trace(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<AspectLoweringTrace> {
        self.schema_contract_runtime
            .aspect_contract_plans
            .entity_plans
            .get(&kind_id)
            .map(LoweredAspectContractPlan::lowering_trace)
    }

    #[cfg(test)]
    pub(crate) fn relation_aspect_plan_trace(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<AspectLoweringTrace> {
        self.schema_contract_runtime
            .aspect_contract_plans
            .relation_plans
            .get(&kind_id)
            .map(LoweredAspectContractPlan::lowering_trace)
    }

    #[cfg(test)]
    pub(crate) fn relation_integrity_plan(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<LoweredRelationIntegrityPlan> {
        use worth_relational_contracts::internal_port::schema::{
            aspect_plans::lower_relation_integrity_plans_for_compilation,
            compilation_control::SchemaCompilationControl,
        };
        use worth_relational_contracts::internal_port::validation::data::catalog::append_relation_integrity_registrations;

        let control = SchemaCompilationControl::Configuration;
        let plan = lower_relation_integrity_plans_for_compilation(self.schema_registry(), control)
            .expect("installed schema lowers")
            .relation_plans
            .remove(&kind_id)?;
        let mut expected = Vec::new();
        append_relation_integrity_registrations(&plan, &mut expected, control)
            .expect("installed native rules lower");
        let installed = &self
            .schema_contract_runtime
            .relation_integrity_registrations;
        assert!(expected.iter().all(|rule| installed.contains(rule)));
        Some(plan)
    }
}
