//! Side-effect-free recovery admission and terminal rejection evidence.

use crate::durability::data::RecoveryPlan;
use contracts::config::data::runtime_config::RelationalRuntimeConfig;
use contracts::durability::data::recovery_errors::{
    DurabilityError, RecoveryAuthorityContinuityMismatch, RecoveryFailureClass,
};
use contracts::schema::data::RelationalSchemaRegistryFieldsPort as _;
use worth_relational_contracts::internal_port as contracts;

use super::super::authority_continuity::validate_recovery_authority_continuity;

/// Recovery plan that passed the live runtime's complete admission boundary.
pub struct AdmittedRecoveryPlan {
    plan: RecoveryPlan,
    prepared_home_schemas:
        Option<super::super::runtime_rebuild::original_schemas::OriginalSchemaGraph>,
}

impl AdmittedRecoveryPlan {
    pub fn into_parts(
        self,
    ) -> (
        RecoveryPlan,
        Option<super::super::runtime_rebuild::original_schemas::OriginalSchemaGraph>,
    ) {
        (self.plan, self.prepared_home_schemas)
    }

    pub fn with_prepared_home_schemas(
        mut self,
        schemas: super::super::runtime_rebuild::original_schemas::OriginalSchemaGraph,
    ) -> Self {
        self.prepared_home_schemas = Some(schemas);
        self
    }
}

pub struct RecoveryRejection {
    error: DurabilityError,
    record_verification_counters: bool,
    authority_continuity: crate::durability::data::RecoveryAuthorityContinuityCheck,
}

impl RecoveryRejection {
    pub const fn records_verification_counters(&self) -> bool {
        self.record_verification_counters
    }

    pub fn into_error(self) -> DurabilityError {
        self.error
    }

    pub const fn authority_continuity(
        &self,
    ) -> &crate::durability::data::RecoveryAuthorityContinuityCheck {
        &self.authority_continuity
    }
}

pub fn admit_recovery(
    config: &RelationalRuntimeConfig,
    plan: RecoveryPlan,
) -> Result<AdmittedRecoveryPlan, RecoveryRejection> {
    let rejected = matches!(
        plan.authority_continuity.verification_outcome,
        crate::durability::data::RecoveryVerificationOutcome::Rejected { .. }
    );
    let result = validate_recovery_authority_continuity(config, &plan)
        .and_then(|()| validate_reported_parity(config, &plan))
        .and_then(|()| validate_world_source(&plan));
    match result {
        Ok(()) => Ok(AdmittedRecoveryPlan {
            plan,
            prepared_home_schemas: None,
        }),
        Err(error) => Err(RecoveryRejection {
            error,
            record_verification_counters: rejected,
            authority_continuity: plan.authority_continuity.clone(),
        }),
    }
}

fn validate_reported_parity(
    config: &RelationalRuntimeConfig,
    plan: &RecoveryPlan,
) -> Result<(), DurabilityError> {
    if !plan.authority_continuity.schema_parity.is_verified() {
        return Err(DurabilityError::new(
            RecoveryFailureClass::SchemaMismatch,
            "recovery schema registry mismatch",
        )
        .with_authority_continuity_mismatch(
            RecoveryAuthorityContinuityMismatch::SchemaRegistryShape {
                expected_primary_schema_version: plan
                    .config
                    .schema
                    .registry
                    .primary_schema_version_id(),
                found_primary_schema_version: config.schema.registry.primary_schema_version_id(),
                expected_entity_kind_count: (*plan.config.schema.registry.owner_entity_kinds())
                    .len(),
                found_entity_kind_count: (*config.schema.registry.owner_entity_kinds()).len(),
                expected_relation_kind_count: (*plan.config.schema.registry.owner_relation_kinds())
                    .len(),
                found_relation_kind_count: (*config.schema.registry.owner_relation_kinds()).len(),
            },
        ));
    }
    if !plan.authority_continuity.profile_parity.is_verified() {
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
    if !plan.authority_continuity.runtime_name_parity.is_verified() {
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
    Ok(())
}

fn validate_world_source(plan: &RecoveryPlan) -> Result<(), DurabilityError> {
    let selected = plan
        .original_source
        .as_ref()
        .and_then(|source| source.world_selection.as_ref());
    if let Some(selected) = selected
        && !plan
            .home_world_selection
            .as_ref()
            .is_some_and(|actual| actual.require_original_source(selected).is_ok())
    {
        return Err(DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            "original World source eligibility belongs to a different or missing Home admission",
        ));
    }
    Ok(())
}
