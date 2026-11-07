use crate::basis_lifecycle::BasisFamily;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::inventory::{
    EffectLifecycleFamilyInventoryRow, EffectLifecycleFamilyKey, EffectLifecyclePublicSurfaceRow,
    EffectLoweredArtifactKind, EffectPublicSurfaceAvailability, EffectPublicSurfaceKind,
    EffectReceiptArtifactKind,
};
use super::planning::EffectAuthorityOwner;
use super::support_matrix::EffectSupportPosture;

pub(super) fn effect_lifecycle_family_rows() -> Result<
    Vec<EffectLifecycleFamilyInventoryRow>,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    Ok(vec![
        EffectLifecycleFamilyInventoryRow::new(
            EffectLifecycleFamilyKey::Mutation,
            EffectAuthorityOwner::WorthRelational,
            vec![
                BasisFamily::CurrentHead,
                BasisFamily::BranchHead,
                BasisFamily::TenantScoped,
                BasisFamily::PolicyScoped,
            ],
            EffectLoweredArtifactKind::LoweredMutationIntentDeclaration,
            EffectReceiptArtifactKind::WorthQueryIntentExecution,
            EffectSupportPosture::Denied,
            EffectSupportPosture::Unsupported,
        )?,
        EffectLifecycleFamilyInventoryRow::new(
            EffectLifecycleFamilyKey::Merge,
            EffectAuthorityOwner::WorthRelational,
            vec![BasisFamily::CurrentHead, BasisFamily::BranchHead],
            EffectLoweredArtifactKind::LoweredMergeWorkflowDeclaration,
            EffectReceiptArtifactKind::WorthQueryIntentExecution,
            EffectSupportPosture::Denied,
            EffectSupportPosture::Unsupported,
        )?,
        EffectLifecycleFamilyInventoryRow::new(
            EffectLifecycleFamilyKey::Writeback,
            EffectAuthorityOwner::WorthRuntimeBridge,
            vec![
                BasisFamily::CurrentHead,
                BasisFamily::BranchHead,
                BasisFamily::TenantScoped,
                BasisFamily::PolicyScoped,
            ],
            EffectLoweredArtifactKind::QueryWritebackDeclaration,
            EffectReceiptArtifactKind::WorthQueryWriteReceipt,
            EffectSupportPosture::Denied,
            EffectSupportPosture::Deferred,
        )?,
        EffectLifecycleFamilyInventoryRow::new(
            EffectLifecycleFamilyKey::OrderedBatch,
            EffectAuthorityOwner::WorthRelational,
            vec![
                BasisFamily::CurrentHead,
                BasisFamily::BranchHead,
                BasisFamily::TenantScoped,
                BasisFamily::PolicyScoped,
            ],
            EffectLoweredArtifactKind::LoweredEffectBatchExecutionPlan,
            EffectReceiptArtifactKind::WorthQueryBatchWriteReceipt,
            EffectSupportPosture::Denied,
            EffectSupportPosture::Unsupported,
        )?,
    ])
}

pub(super) fn effect_lifecycle_public_surface_rows() -> Result<
    Vec<EffectLifecyclePublicSurfaceRow>,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    Ok(vec![
        EffectLifecyclePublicSurfaceRow::new(
            EffectPublicSurfaceKind::CommonPathIntentAuthoring,
            Some(
                "normalize_raw_effect_intent(...) -> evaluate_effect_eligibility(...) -> admit_effect_intent(...)",
            ),
            Some(EffectReceiptArtifactKind::WorthQueryIntentExecution),
            EffectPublicSurfaceAvailability::Implemented,
            true,
        )?,
        EffectLifecyclePublicSurfaceRow::new(
            EffectPublicSurfaceKind::WritebackCommonPath,
            Some(
                "normalize_raw_effect_intent(...) -> evaluate_effect_eligibility(...) -> admit_effect_intent(...) -> scope_admitted_effect_plan(...).lower().execute()",
            ),
            Some(EffectReceiptArtifactKind::WorthQueryWriteReceipt),
            EffectPublicSurfaceAvailability::Implemented,
            true,
        )?,
        EffectLifecyclePublicSurfaceRow::new(
            EffectPublicSurfaceKind::InspectableLoweredPlan,
            Some("scope_admitted_effect_plan(...).lower()"),
            Some(EffectReceiptArtifactKind::WorthQueryIntentExecution),
            EffectPublicSurfaceAvailability::Implemented,
            true,
        )?,
        EffectLifecyclePublicSurfaceRow::new(
            EffectPublicSurfaceKind::SupportDiscovery,
            Some("discover_effect_lifecycle_support(...)"),
            None,
            EffectPublicSurfaceAvailability::Implemented,
            true,
        )?,
        EffectLifecyclePublicSurfaceRow::new(
            EffectPublicSurfaceKind::DenialOrRebind,
            Some("evaluate_effect_eligibility(...)"),
            None,
            EffectPublicSurfaceAvailability::Implemented,
            true,
        )?,
        EffectLifecyclePublicSurfaceRow::new(
            EffectPublicSurfaceKind::BatchExecution,
            Some("effect_batch().using_basis(...).admit().lower().execute()"),
            Some(EffectReceiptArtifactKind::WorthQueryBatchWriteReceipt),
            EffectPublicSurfaceAvailability::Implemented,
            true,
        )?,
        EffectLifecyclePublicSurfaceRow::new(
            EffectPublicSurfaceKind::DiagnosticsEnvelope,
            Some(
                "execute_receipt_with(...).transition_rules() / receipt.effect_envelope() / receipt.materialize_diagnostics(...)",
            ),
            Some(EffectReceiptArtifactKind::SelfDescribingEffectEnvelope),
            EffectPublicSurfaceAvailability::Implemented,
            true,
        )?,
        EffectLifecyclePublicSurfaceRow::new(
            EffectPublicSurfaceKind::ProductionCertification,
            Some("certify_effect_execution_pipeline()"),
            None,
            EffectPublicSurfaceAvailability::Implemented,
            true,
        )?,
        EffectLifecyclePublicSurfaceRow::new(
            EffectPublicSurfaceKind::HiddenLowerRuntimeTypes,
            None,
            None,
            EffectPublicSurfaceAvailability::Implemented,
            true,
        )?,
    ])
}
