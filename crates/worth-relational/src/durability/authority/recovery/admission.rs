//! Side-effect-free recovery admission and terminal rejection evidence.

use crate::capabilities::{RuntimeIdentitySource, SchemaSource, SchemaVersionSource};
use crate::diagnostics::data::{
    DiagnosticsArtifactKind, DiagnosticsScope, RelationalDiagnosticsEntry,
};
use crate::durability::data::{
    DurabilityError, RecoveryAuthorityContinuityMismatch, RecoveryFailureClass, RecoveryPlan,
};
use crate::runtime::RelationalRuntime;

use super::super::authority_continuity::{
    recovery_authority_continuity_diagnostic, validate_recovery_authority_continuity,
};

#[derive(Clone)]
pub(super) struct RecoveryAdmission {
    artifact_kind: DiagnosticsArtifactKind,
    entry: RelationalDiagnosticsEntry,
}

/// Recovery plan that passed the live runtime's complete admission boundary.
pub(in crate::durability::authority) struct AdmittedRecoveryPlan {
    plan: RecoveryPlan,
    admission: RecoveryAdmission,
    prepared_home_schemas: Option<super::super::runtime_rebuild::OriginalSchemaGraph>,
}

impl AdmittedRecoveryPlan {
    pub(super) const fn admission(&self) -> &RecoveryAdmission {
        &self.admission
    }

    pub(in crate::durability::authority) fn into_parts(
        self,
    ) -> (
        RecoveryPlan,
        Option<super::super::runtime_rebuild::OriginalSchemaGraph>,
    ) {
        (self.plan, self.prepared_home_schemas)
    }

    pub(super) fn with_prepared_home_schemas(
        mut self,
        schemas: super::super::runtime_rebuild::OriginalSchemaGraph,
    ) -> Self {
        self.prepared_home_schemas = Some(schemas);
        self
    }
}

impl RecoveryAdmission {
    pub(super) const fn artifact_kind(&self) -> DiagnosticsArtifactKind {
        self.artifact_kind
    }

    pub(super) fn into_entry(self) -> RelationalDiagnosticsEntry {
        self.entry
    }

    pub(super) fn emit(&self, runtime: &RelationalRuntime) {
        runtime.publication_authority().push_bounded_diagnostic(
            DiagnosticsScope::History,
            self.artifact_kind,
            vec![self.entry.clone()],
        );
    }
}

pub(super) struct RecoveryRejection {
    admission: RecoveryAdmission,
    error: DurabilityError,
    record_verification_counters: bool,
    authority_continuity: crate::durability::data::RecoveryAuthorityContinuityCheck,
}

impl RecoveryRejection {
    pub(super) const fn records_verification_counters(&self) -> bool {
        self.record_verification_counters
    }

    pub(super) fn emit(&self, runtime: &RelationalRuntime) {
        self.admission.emit(runtime);
    }

    pub(super) fn into_error(self) -> DurabilityError {
        self.error
    }

    pub(super) const fn authority_continuity(
        &self,
    ) -> &crate::durability::data::RecoveryAuthorityContinuityCheck {
        &self.authority_continuity
    }
}

pub(super) fn admit_recovery(
    runtime: &RelationalRuntime,
    mut plan: RecoveryPlan,
) -> Result<AdmittedRecoveryPlan, RecoveryRejection> {
    plan.home_world_selection = runtime
        .durability
        .medium()
        .and_then(|medium| medium.home_world_selection());
    let rejected = matches!(
        plan.authority_continuity.verification_outcome,
        crate::durability::data::RecoveryVerificationOutcome::Rejected { .. }
    );
    let admission = RecoveryAdmission {
        artifact_kind: if rejected {
            DiagnosticsArtifactKind::Failure
        } else {
            DiagnosticsArtifactKind::MinimalSummary
        },
        entry: recovery_authority_continuity_diagnostic(&plan),
    };
    let result = validate_recovery_authority_continuity(runtime, &plan)
        .and_then(|()| validate_reported_parity(runtime, &plan))
        .and_then(|()| validate_world_source(&plan));
    match result {
        Ok(()) => Ok(AdmittedRecoveryPlan {
            plan,
            admission,
            prepared_home_schemas: None,
        }),
        Err(error) => Err(RecoveryRejection {
            admission,
            error,
            record_verification_counters: rejected,
            authority_continuity: plan.authority_continuity.clone(),
        }),
    }
}

fn validate_reported_parity(
    runtime: &RelationalRuntime,
    plan: &RecoveryPlan,
) -> Result<(), DurabilityError> {
    if !plan.authority_continuity.schema_parity.is_verified() {
        return Err(DurabilityError::new(
            RecoveryFailureClass::SchemaMismatch,
            "recovery schema registry mismatch",
        )
        .with_authority_continuity_mismatch(
            RecoveryAuthorityContinuityMismatch::SchemaRegistryShape {
                expected_primary_schema_version: plan.config.primary_schema_version_id(),
                found_primary_schema_version: runtime.primary_schema_version_id(),
                expected_entity_kind_count: plan.config.schema.registry.entity_kinds.len(),
                found_entity_kind_count: runtime.schema_registry().entity_kinds.len(),
                expected_relation_kind_count: plan.config.schema.registry.relation_kinds.len(),
                found_relation_kind_count: runtime.schema_registry().relation_kinds.len(),
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
                found: format!("{:?}", runtime.runtime_profile()),
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
                found: runtime.runtime_name().to_string(),
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
    if let Some(selected) = selected {
        if !plan
            .home_world_selection
            .as_ref()
            .is_some_and(|actual| actual.require_original_source(selected).is_ok())
        {
            return Err(DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint,
                "original World source eligibility belongs to a different or missing Home admission"));
        }
    }
    Ok(())
}
