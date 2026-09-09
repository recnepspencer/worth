mod appearance;
#[cfg(test)]
#[path = "appearance_fact_index_test_support.rs"]
pub(crate) mod appearance_fact_index_test_support;
mod artifact;
mod aspect_contract;
mod closeout;
mod component_reference;
mod declaration_handoff;
mod declared_posture;
mod family;
mod inspection;
mod intent;
mod measurement_dependency;
mod overlay;
#[cfg(any(test, feature = "certification-support"))]
mod rust_authored_declaration_fixture;
mod service;
mod structural_semantics;
mod support;

#[allow(
    unused_imports,
    reason = "Gate 0 freezes the declaration-side appearance contract"
)]
pub use appearance::{
    UiAppearanceAspect, UiAppearanceAspectContract, UiAppearanceAxisClass, UiAppearanceAxisDomain,
    UiAppearanceAxisPredicate, UiAppearanceCell, UiAppearanceCellBuilder,
    UiAppearanceCellBuilderDenial, UiAppearanceCellValue, UiAppearanceDecisionCell,
    UiAppearanceDecisionPartition, UiAppearanceDecisionPartitionDenial, UiAppearanceDecisionResult,
    UiAppearanceDecisionRule, UiAppearanceDecisionValue, UiAppearancePartitionAuthoring,
    UiAppearanceRole, UiAppearanceRoleAuthoring, UiAppearanceRoleAuthoringDenial,
    UiAppearanceRoleDeclaration, UiAppearanceRoleDeclarationDenial, UiAppearanceRoleId,
    UiAppearanceRoleIdentity, UiAppearanceRoleRevision, UiAppearanceRoleSchemaVersion,
    UiAppearanceStateAxis, UiAppearanceStateAxisVersion, UiThemeSlotUse,
};
#[allow(
    unused_imports,
    reason = "Gate 0 retains non-current attachment and pointer declarations"
)]
pub(crate) use appearance::{
    UiAppearanceRoleAttachment, UiAppearanceRoleAttachmentDenial, UiPointerAffordance,
};
pub(crate) use artifact::ui_declaration_lowering::UiDeclarationLowering;
pub(crate) use artifact::{
    authored_source_provenance_digest, stable_text_digest, UiDeclarationArtifactInput,
};
pub use artifact::{
    UiDeclarationArtifact, UiDeclarationArtifactDigest, UiDeclarationAspectDigest,
    UiDeclarationDigestProjection, UiDeclarationEquivalenceContract, UiDeclarationFamilyDigest,
    UiDeclarationIdentity, UiDeclarationIdentityDigest, UiDeclarationPostureDigest,
    UiDeclarationProvenance, UiDeclarationStructuralDigest, UiDeclarationSupportDigest,
};
pub(crate) use aspect_contract::UiAspectContractAdmission;
pub use aspect_contract::{
    UiAspectContract, UiAspectContractAdmissionDenial, UiAspectCoverageEntry,
    UiAspectCoverageReport, UiAspectFamily, UiAspectName, UiAspectSemanticSlice,
    UiConsumedAspectContract, UiPublishedAspectContract,
};
pub use closeout::{
    UiDeclarationClosedSemanticLane, UiDeclarationCloseoutGuarantee, UiDeclarationCloseoutNonGoal,
    UiDeclarationCloseoutReport,
};
pub(crate) use component_reference::admit_component_reference;
pub use component_reference::UiDeclarationComponentReferenceDenial;
pub use declaration_handoff::{UiDeclarationGraphHandoff, UiDeclarationGraphHandoffDenial};
pub(crate) use declaration_handoff::{
    UiDeclaredAspectPayload, UiDeclaredPosturePayload, UiStructuralDeclarationPayload,
};
pub(crate) use declared_posture::UiDeclaredPostureAdmission;
pub use declared_posture::{
    UiDeclaredHostCapabilityPosture, UiDeclaredMeasurementBasisSource,
    UiDeclaredMeasurementConstraintModifier, UiDeclaredMeasurementEvidenceRequirement,
    UiDeclaredMeasurementMode, UiDeclaredMeasurementOwnershipPosture,
    UiDeclaredMeasurementPolicyPosture, UiDeclaredPostureAdmissionDenial,
    UiDeclaredPostureApplicability, UiDeclaredPostureContract, UiDeclaredPostureLane,
    UiDeclaredPostureLaneKind, UiDeclaredQueryBindingPosture, UiDeclaredServiceUsagePosture,
    UiDeclaredTouchMeaningPosture,
};
pub(crate) use family::UiDeclarationFamilyAdmission;
pub use family::{
    UiDeclarationFamily, UiDeclarationFamilyAdmissionDenial, UiDeclarationFamilyCatalog,
    UiDeclarationFamilyKind,
};
pub(crate) use inspection::{UiDeclarationAuthoredEvidenceIndex, UiDeclarationEvidenceRecord};
pub(crate) use intent::{
    prepare_authored_intent_material, UiCanonicalIntentDeclaration, UiIntentCatalog,
    UiIntentCatalogCommandRoute, UiIntentCatalogResolvedRoute, UiIntentCatalogSemanticComparison,
    UiIntentSingleProductRouteDenial, UiResolvedIntentApplicationSource,
    UiResolvedIntentConfirmationContract, UiResolvedIntentConfirmationSource,
    UiResolvedIntentMutabilitySource, UiResolvedIntentPayloadBinding,
    UiResolvedIntentPayloadSource, UiResolvedIntentProjectionSource,
    UiResolvedIntentReadinessSource, WorthUiAuthoredIntentDeclaration,
    WorthUiAuthoredIntentMaterial, WorthUiAuthoredIntentRoute,
};
pub use intent::{
    UiIntentApplicationFact, UiIntentApplicationFactIdentityError,
    UiIntentApplicationFactRegistrationError, UiIntentCatalogMetrics,
    UiIntentCatalogPreparationDenial, UiIntentConcurrencyScope, UiIntentConfirmationContract,
    UiIntentConfirmationContractIdentityError, UiIntentConfirmationRouteBinding,
    UiIntentConsequenceContract, UiIntentDeclaration, UiIntentDeclarationConcurrencyBound,
    UiIntentDeclarationConcurrencyMissing, UiIntentDeclarationConfirmationBound,
    UiIntentDeclarationConfirmationMissing, UiIntentDeclarationConsequencesBound,
    UiIntentDeclarationConsequencesMissing, UiIntentDeclarationConstructionError,
    UiIntentDeclarationIdentity, UiIntentDeclarationOperabilityBound,
    UiIntentDeclarationOperabilityMissing, UiIntentInteractionPayloadSourceKind,
    UiIntentMutabilitySource, UiIntentOperabilityContract,
    UiIntentOperabilityContractIdentityError, UiIntentOperabilityDependencyAxis,
    UiIntentPayloadSource, UiIntentPolicySource, UiIntentReadinessSource, UiIntentRouteBinding,
    UiIntentRouteResolutionCost,
};
pub(crate) use intent::{
    UiIntentApplicationFactPlan, UiIntentApplicationFactSlot, UiIntentApplicationFactValue,
};
pub(crate) use measurement_dependency::declared_measurement_basis_requirements;
pub(crate) use measurement_dependency::declared_query_measurement_dependencies;
pub use measurement_dependency::{
    UiDeclaredMeasurementBasisRequirementSet, UiDeclaredMeasurementQueryDependencySet,
};
#[allow(
    unused_imports,
    reason = "Gate 0 freezes non-current backdrop and overlay declarations"
)]
pub use overlay::{
    UiBackdropDeclaration, UiBackdropDeclarationDenial, UiBackdropExtentBasis, UiBackdropIdentity,
    UiBackdropMotionBasis, UiBackdropPlacement, UiBackdropPresenceBasis, UiBackdropScope,
    UiMosaicRegionDeclarationIdentity, UiOverlayAnchor, UiOverlayPortalParticipant,
    UiOverlayRelation, UiOverlayRelationAdmissionDenial, UiOverlayRelationGraph,
    UiOverlayRelationKind, UiPortalDeclarationId, UiSemanticSurfaceDeclarationIdentity,
};
#[cfg(any(test, feature = "certification-support"))]
pub(crate) use rust_authored_declaration_fixture::WorthUiRustAuthoredDeclarationFixture;
pub(crate) use service::UiDeclaredPortalPlacementGeometry;
pub(crate) use service::UiServicePolicyDefaults;
pub use service::{
    UiCommandRoutingPolicy, UiFocusPolicy, UiFocusScopePolicy, UiMotionPolicy,
    UiNormalizedServicePolicyPlan, UiPortalPolicy, UiPortalPolicyKind, UiReducedMotionBehavior,
    UiScrollAnchorBehavior, UiScrollPolicy, UiScrollRevealAlignment, UiSelectionMode,
    UiSelectionPolicy,
};
pub use structural_semantics::{
    UiDeclarationContainmentIntent, UiDeclarationOrderingGuarantee,
    UiDeclarationPlanningOperatorKind, UiDeclarationRepetitionPosture,
    UiDeclarationSlotParticipationIntent, UiDeclarationStructuralRole,
    UiDeclarationStructuralSemantics, UiDeclarationStructuralSemanticsAdmissionDenial,
};
pub(crate) use structural_semantics::{
    UiDeclarationStructuralSemanticsAdmission, UiDeclarationStructuralSemanticsInput,
};
pub(crate) use support::{
    derive_declaration_inspection_support_projection, UiDeclarationInspectionSupportProjection,
    UiDeclarationSupportSnapshotAdmission,
};
pub use support::{
    UiDeclarationSupportMilestoneExpectation, UiDeclarationSupportRow,
    UiDeclarationSupportRowSchemaKind, UiDeclarationSupportSnapshot,
    UiDeclarationSupportSnapshotAdmissionDenial, UiDeclarationUnsupportedPosture,
};

#[cfg(test)]
mod declaration_measurement_registration_tests;
#[cfg(test)]
mod declared_measurement_posture_tests;
#[cfg(test)]
mod declared_posture_tests;
#[cfg(test)]
mod structural_operator_tests;
#[cfg(test)]
mod support_inspection_tests;
#[cfg(test)]
mod support_measurement_tests;
#[cfg(test)]
mod support_tests;
#[cfg(test)]
mod tests;
