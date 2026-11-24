mod checkpoint_lineage;
#[cfg(test)]
mod checkpoint_lineage_restore_tests;
mod registry_mismatch;
mod schema_continuity;

use crate::config::data::RelationalRuntimeConfig;
use crate::diagnostics::data::RelationalDiagnosticsEntry;
use crate::durability::data::{
    DurabilityError, RecoveryAuthorityContinuityMismatch, RecoveryFailureClass, RecoveryPlan,
};

use super::diagnostics::recovery_authority_continuity_evaluated;
use registry_mismatch::schema_registry_mismatch;
use schema_continuity::validate_schema_recovery_authority_continuity;

pub(super) fn validate_recovery_authority_continuity(
    config: &RelationalRuntimeConfig,
    plan: &RecoveryPlan,
) -> Result<(), DurabilityError> {
    if plan.config.schema.registry != config.schema.registry {
        return Err(DurabilityError::new(
            RecoveryFailureClass::SchemaMismatch,
            "recovery schema registry mismatch",
        )
        .with_authority_continuity_mismatch(schema_registry_mismatch(
            &plan.config.schema.registry,
            &config.schema.registry,
            plan.config.schema.registry.primary_schema_version_id(),
            config.schema.registry.primary_schema_version_id(),
        )));
    }
    if plan.config.profile != config.profile {
        return Err(DurabilityError::new(
            RecoveryFailureClass::ProfileMismatch,
            "recovery profile mismatch",
        )
        .with_authority_continuity_mismatch(
            RecoveryAuthorityContinuityMismatch::RuntimeProfile {
                expected: format!("{:?}", plan.config.profile),
                found: format!("{:?}", config.profile),
            },
        ));
    }
    if plan.config.execution.runtime_name != config.execution.runtime_name {
        return Err(DurabilityError::new(
            RecoveryFailureClass::RuntimeNameMismatch,
            "recovery runtime name mismatch",
        )
        .with_authority_continuity_mismatch(
            RecoveryAuthorityContinuityMismatch::RuntimeName {
                expected: plan.config.execution.runtime_name.clone(),
                found: config.execution.runtime_name.clone(),
            },
        ));
    }
    validate_schema_recovery_authority_continuity(config, plan)?;
    Ok(())
}

pub(super) fn recovery_authority_continuity_diagnostic(
    plan: &RecoveryPlan,
) -> RelationalDiagnosticsEntry {
    recovery_authority_continuity_evaluated(plan)
}

pub(super) fn validate_checkpoint_lineage_artifact(
    checkpoint: &crate::durability::data::DurableCheckpoint,
) -> Result<(), DurabilityError> {
    checkpoint_lineage::validate_checkpoint_lineage_artifact(checkpoint)
}
