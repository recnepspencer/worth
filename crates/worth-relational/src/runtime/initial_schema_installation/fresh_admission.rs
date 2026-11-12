//! Same-runtime installation validation consumes no reconstruction authority.
use super::*;
use crate::durability::authority::FreshRelationalSourceAdmission;
use worth_execution::ExecutionRequest;
impl RelationalRuntime {
    pub fn validate_fresh_initial_schema_installation(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        fresh: &FreshRelationalSourceAdmission,
        expected: &RelationalInitialSchemaInstallationReceipt,
    ) -> Result<(), RelationalInitialSchemaInstallationDenial> {
        fresh
            .validate_fresh_source_runtime(execution, self)
            .map_err(|cause| {
                RelationalInitialSchemaInstallationDenial::new(
                    match cause {
                        crate::durability::data::RelationalRecoveryStop::Request(cause) => {
                            RelationalInitialSchemaInstallationDenialKind::Request(cause)
                        }
                        _ => RelationalInitialSchemaInstallationDenialKind::FreshSourceMismatch,
                    },
                    "fresh source is not this original initial Native state",
                )
            })?;
        // The private receipt came from this SAME issuer and the one sealed
        // installation generation; no registry/handler copies or remint occur.
        if self.runtime_instance_id() != expected.runtime_instance_id
            || !self
                .schema_contract_runtime
                .initial_custom_invariants_sealed
            || self.schema_contract_runtime.custom_invariant_generation
                != expected.custom_invariant_generation
            || !fresh.matches_installation(
                expected.custom_invariant_generation,
                &expected.retained_schema_authority_digest,
                expected.retained_entity_kind_count,
                expected.retained_relation_kind_count,
            )
        {
            return Err(RelationalInitialSchemaInstallationDenial::new(
                RelationalInitialSchemaInstallationDenialKind::FreshSourceMismatch,
                "fresh source differs from the original initial installation",
            ));
        }
        Ok(())
    }
}
