use crate::diagnostics::data::DiagnosticCode;
use crate::transactions::data::ExistingRecordTarget;

use super::entity_authoritative_aspect_state_presentation::entity_authoritative_aspect_state_denial_detail;
use super::record_aspect_patch_denial::denial_detail as record_aspect_patch_denial_detail;
use super::ConflictClass;

impl ConflictClass {
    pub fn code(&self) -> DiagnosticCode {
        match self {
            Self::StaleTarget { .. } => DiagnosticCode::StaleHandle,
            Self::InvalidRelationEndpoint { .. } => DiagnosticCode::InvalidRelationEndpoint,
            Self::DuplicateRelationIdentity { .. } => DiagnosticCode::DuplicateRelationIdentity,
            Self::InvariantViolation { code, .. } => *code,
            Self::KindSchemaMismatch { .. } => DiagnosticCode::InvariantViolation,
            Self::MutationStateInconsistency { .. } => DiagnosticCode::StorageInconsistencyDetected,
            Self::RecordAllocationDenied { .. } => DiagnosticCode::StorageInconsistencyDetected,
            Self::EntityUpdateStateInconsistency { .. } => {
                DiagnosticCode::StorageInconsistencyDetected
            }
            Self::EntityFieldUpdateStateInconsistency { .. } => {
                DiagnosticCode::StorageInconsistencyDetected
            }
            Self::RelationEndpointUpdateStateInconsistency { .. }
            | Self::RelationEndpointUpdateKindMismatch { .. }
            | Self::BulkImportDomainMismatch { .. } => DiagnosticCode::StorageInconsistencyDetected,
            Self::EntityAuthoritativeAspectStateDenied { .. } => {
                DiagnosticCode::StorageInconsistencyDetected
            }
            Self::RecordAspectPatchDenied { .. } => DiagnosticCode::InvariantViolation,
            Self::AspectDeltaFailure { .. } => DiagnosticCode::AspectDeltaFailure,
            Self::ConflictingIntent { .. } => DiagnosticCode::ConflictingIntent,
            Self::InvalidSavepoint { .. } => DiagnosticCode::InvalidSavepoint,
            Self::InvalidMergeParent { .. } => DiagnosticCode::InvalidMergeParent,
            Self::StaleValidationBasis { .. } => DiagnosticCode::StaleHandle,
            Self::ForeignRuntime { .. } => DiagnosticCode::StaleHandle,
            Self::TransactionFootprintBudgetExceeded { .. }
            | Self::TransactionOverlayBudgetExceeded { .. }
            | Self::TransactionSavepointBudgetExceeded { .. }
            | Self::TransactionSavepointFootprintBudgetExceeded { .. }
            | Self::TransactionSavepointIdentityExhausted
            | Self::MaterializationAuthorityRequired
            | Self::MaterializationModeMismatch => DiagnosticCode::PreparationFailure,
            Self::MergeConflictOverlap { .. } => DiagnosticCode::MergeConflictOverlap,
            Self::MissingMergeBase { .. } => DiagnosticCode::MissingMergeBase,
            Self::UndeclaredSchemaTransition { .. }
            | Self::DescriptorSemanticsVersionUnsupported { .. }
            | Self::InvalidSchemaTransitionSourceBasis { .. }
            | Self::InvalidSchemaTransitionTargetBasis { .. }
            | Self::MissingSchemaBasisForTransition { .. }
            | Self::UnsupportedBridgeDescriptor { .. }
            | Self::HistoricalReinterpretationViolation { .. }
            | Self::TypeContinuityDeniedSchemaTransition { .. }
            | Self::StructuralContinuityDeniedSchemaTransition { .. }
            | Self::DirectionalityMismatchUnderCanonicalReconciliation { .. }
            | Self::InvalidSchemaTransitionShape { .. } => {
                DiagnosticCode::SchemaContinuityViolation
            }
        }
    }

    pub fn detail(&self) -> String {
        match self {
            Self::StaleTarget { target, context } => stale_target_detail(target, context),
            Self::InvalidRelationEndpoint { detail }
            | Self::DuplicateRelationIdentity { detail }
            | Self::KindSchemaMismatch { detail }
            | Self::MutationStateInconsistency { detail, .. }
            | Self::AspectDeltaFailure { detail, .. }
            | Self::InvalidMergeParent { detail }
            | Self::StaleValidationBasis { detail }
            | Self::MergeConflictOverlap { detail }
            | Self::MissingMergeBase { detail }
            | Self::UnsupportedBridgeDescriptor { detail }
            | Self::HistoricalReinterpretationViolation { detail }
            | Self::TypeContinuityDeniedSchemaTransition { detail }
            | Self::StructuralContinuityDeniedSchemaTransition { detail }
            | Self::DirectionalityMismatchUnderCanonicalReconciliation { detail } => detail.clone(),
            Self::ForeignRuntime {
                expected_runtime_instance_id,
                actual_runtime_instance_id,
            } => format!(
                "transaction belongs to Relational runtime {actual_runtime_instance_id}, not supplied runtime {expected_runtime_instance_id}"
            ),
            Self::TransactionFootprintBudgetExceeded {
                maximum_loci,
                required_loci,
            } => format!(
                "transaction footprint requires {required_loci} loci but the owner limit is {maximum_loci}"
            ),
            Self::TransactionOverlayBudgetExceeded {
                maximum_bytes,
                required_bytes,
            } => format!(
                "transaction overlay requires {required_bytes} bytes but the owner limit is {maximum_bytes}"
            ),
            Self::TransactionSavepointBudgetExceeded { maximum_savepoints } => format!(
                "transaction savepoints reached the owner limit of {maximum_savepoints}"
            ),
            Self::TransactionSavepointFootprintBudgetExceeded {
                maximum_loci,
                required_loci,
            } => format!(
                "transaction savepoint footprints require {required_loci} loci but the owner limit is {maximum_loci}"
            ),
            Self::TransactionSavepointIdentityExhausted => {
                "transaction savepoint identity space exhausted".to_owned()
            }
            Self::MaterializationAuthorityRequired => {
                "materialization transition requires the Relational owner's sealed port".to_owned()
            }
            Self::MaterializationModeMismatch => {
                "materialization transaction contains an intent outside its sealed mode".to_owned()
            }
            Self::RecordAllocationDenied { denial } => denial.detail(),
            Self::InvariantViolation { detail, .. } => detail.clone(),
            Self::InvalidSchemaTransitionShape { detail } => detail.clone(),
            Self::EntityUpdateStateInconsistency { missing, .. } => format!(
                "entity update requires a retained {} after stale-target validation",
                missing.label()
            ),
            Self::EntityFieldUpdateStateInconsistency { missing, .. } => format!(
                "entity field update requires a retained {} after stale-target validation",
                missing.label()
            ),
            Self::RelationEndpointUpdateStateInconsistency { missing, .. } => format!(
                "relation endpoint update requires retained {} after stale-target validation",
                missing.label()
            ),
            Self::RelationEndpointUpdateKindMismatch {
                intent_kind_id,
                authoritative_kind_id,
                ..
            } => format!(
                "relation endpoint update intent kind {} does not match authoritative relation kind {}",
                intent_kind_id.0, authoritative_kind_id.0
            ),
            Self::BulkImportDomainMismatch {
                expected,
                actual,
                stage,
            } => format!(
                "{} expected {} rows but staged {} rows",
                stage.label(),
                expected.label(),
                actual.label()
            ),
            Self::EntityAuthoritativeAspectStateDenied { kind_id, denial } => format!(
                "entity field declarations for kind {} could not be admitted as authoritative aspect state: {}",
                kind_id.0,
                entity_authoritative_aspect_state_denial_detail(denial)
            ),
            Self::RecordAspectPatchDenied { target, denial } => {
                record_aspect_patch_denial_detail(*target, denial)
            }
            Self::ConflictingIntent { target } => conflicting_intent_detail(target),
            Self::InvalidSavepoint { savepoint_id } => {
                format!("savepoint {:?} does not exist", savepoint_id)
            }
            Self::UndeclaredSchemaTransition {
                previous_schema_version,
                current_schema_version,
                previous_descriptor_semantics_version,
                current_descriptor_semantics_version,
            } => format!(
                "schema continuity violation: branch head schema {:?} cannot continue into {:?} without an explicit schema transition (descriptor semantics {:?} -> {:?})",
                previous_schema_version,
                current_schema_version,
                previous_descriptor_semantics_version,
                current_descriptor_semantics_version
            ),
            Self::DescriptorSemanticsVersionUnsupported {
                previous_descriptor_semantics_version,
                current_descriptor_semantics_version,
            } => format!(
                "descriptor semantics version mismatch: branch head uses {:?} but runtime requires {:?}",
                previous_descriptor_semantics_version,
                current_descriptor_semantics_version
            ),
            Self::InvalidSchemaTransitionSourceBasis {
                declared_schema_id,
                declared_schema_version,
                expected_schema_id,
                expected_schema_version,
            } => format!(
                "declared schema transition source {:?}/{:?} does not match authoritative prior schema basis {:?}/{:?}",
                declared_schema_id,
                declared_schema_version,
                expected_schema_id,
                expected_schema_version
            ),
            Self::InvalidSchemaTransitionTargetBasis {
                declared_schema_id,
                declared_schema_version,
                expected_schema_id,
                expected_schema_version,
            } => format!(
                "declared schema transition target {:?}/{:?} does not match authoritative runtime schema basis {:?}/{:?}",
                declared_schema_id,
                declared_schema_version,
                expected_schema_id,
                expected_schema_version
            ),
            Self::MissingSchemaBasisForTransition { role } => {
                format!("declared schema transition requires a non-empty {role} schema basis")
            }
        }
    }
}

fn stale_target_detail(target: &ExistingRecordTarget, context: &str) -> String {
    match target {
        ExistingRecordTarget::Entity(entity_id) => format!(
            "entity {:?} changed before authoritative apply ({context})",
            entity_id
        ),
        ExistingRecordTarget::Relation(relation_id) => format!(
            "relation {:?} changed before authoritative apply ({context})",
            relation_id
        ),
    }
}

fn conflicting_intent_detail(target: &ExistingRecordTarget) -> String {
    match target {
        ExistingRecordTarget::Entity(entity_id) => {
            format!(
                "conflicting entity intent for slot {}",
                entity_id.local_slot.0
            )
        }
        ExistingRecordTarget::Relation(relation_id) => {
            format!(
                "conflicting relation intent for slot {}",
                relation_id.local_slot.0
            )
        }
    }
}
