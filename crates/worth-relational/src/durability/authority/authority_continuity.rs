mod checkpoint_lineage;
#[cfg(test)]
mod checkpoint_lineage_restore_tests;
mod registry_mismatch;
mod schema_continuity;

use crate::capabilities::{
    RuntimeConfigSource, RuntimeIdentitySource, SchemaSource, SchemaVersionSource,
};
use crate::diagnostics::data::RelationalDiagnosticsEntry;
use crate::durability::data::{
    DurabilityError, RecoveryAuthorityContinuityCheck, RecoveryAuthorityContinuityMismatch,
    RecoveryFailureClass, RecoveryPlan,
};
use crate::runtime::RelationalRuntime;

use super::diagnostics::recovery_authority_continuity_evaluated;
use registry_mismatch::schema_registry_mismatch;
use schema_continuity::validate_schema_recovery_authority_continuity;

pub(super) fn validate_recovery_authority_continuity(
    runtime: &(impl SchemaSource + RuntimeIdentitySource + SchemaVersionSource + RuntimeConfigSource),
    plan: &RecoveryPlan,
) -> Result<(), DurabilityError> {
    if plan.config.schema.registry != *runtime.schema_registry() {
        return Err(DurabilityError::new(
            RecoveryFailureClass::SchemaMismatch,
            "recovery schema registry mismatch",
        )
        .with_authority_continuity_mismatch(schema_registry_mismatch(
            &plan.config.schema.registry,
            runtime.schema_registry(),
            plan.config.primary_schema_version_id(),
            runtime.primary_schema_version_id(),
        )));
    }
    if plan.config.profile != runtime.runtime_profile() {
        return Err(DurabilityError::new(
            RecoveryFailureClass::ProfileMismatch,
            "recovery profile mismatch",
        )
        .with_authority_continuity_mismatch(
            RecoveryAuthorityContinuityMismatch::RuntimeProfile {
                expected: format!("{:?}", plan.config.profile),
                found: format!("{:?}", runtime.runtime_profile()),
            },
        ));
    }
    if plan.config.execution.runtime_name != runtime.runtime_name() {
        return Err(DurabilityError::new(
            RecoveryFailureClass::RuntimeNameMismatch,
            "recovery runtime name mismatch",
        )
        .with_authority_continuity_mismatch(
            RecoveryAuthorityContinuityMismatch::RuntimeName {
                expected: plan.config.execution.runtime_name.clone(),
                found: runtime.runtime_name().to_string(),
            },
        ));
    }
    validate_schema_recovery_authority_continuity(runtime, plan)?;
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

pub(super) fn record_recovery_verification_counters(
    runtime: &RelationalRuntime,
    authority_continuity: &RecoveryAuthorityContinuityCheck,
) {
    let layer = match authority_continuity.verification_outcome {
        crate::durability::data::RecoveryVerificationOutcome::VerifiedAtLayer(layer) => layer,
        crate::durability::data::RecoveryVerificationOutcome::Rejected { layer, .. } => layer,
    };
    runtime
        .performance_access()
        .count_replay_verification_layer(layer);
    if matches!(
        authority_continuity.first_mismatch,
        Some(RecoveryAuthorityContinuityMismatch::DescriptorSemanticsVersion { .. })
            | Some(RecoveryAuthorityContinuityMismatch::DescriptorCanonicalBasisVersion { .. })
    ) {
        runtime
            .performance_access()
            .count_descriptor_version_mismatch();
    }
}
