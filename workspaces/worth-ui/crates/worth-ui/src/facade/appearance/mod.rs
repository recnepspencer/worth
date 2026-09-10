//! Public Gate 1 appearance declaration and finite-partition contracts.

pub use worth_ui_dsl::{
    UiAppearanceAspect, UiAppearanceAspectApplicability, UiAppearanceAspectContract,
    UiAppearanceAspectContractDenial, UiAppearanceAxisClass, UiAppearanceAxisDomain,
    UiAppearanceAxisPredicate, UiAppearanceCell, UiAppearanceCellBuilder,
    UiAppearanceCellBuilderDenial, UiAppearanceCellValue, UiAppearanceDecisionCell,
    UiAppearanceDecisionPartition, UiAppearanceDecisionPartitionDenial, UiAppearanceDecisionResult,
    UiAppearanceDecisionRule, UiAppearanceDecisionValue, UiAppearancePartitionAuthoring,
    UiAppearanceRole, UiAppearanceRoleApplicability, UiAppearanceRoleAttachmentDeclaration,
    UiAppearanceRoleAttachmentDeclarationDenial, UiAppearanceRoleAuthoring,
    UiAppearanceRoleAuthoringDenial, UiAppearanceRoleDeclaration,
    UiAppearanceRoleDeclarationDenial, UiAppearanceRoleId, UiAppearanceRoleIdentity,
    UiAppearanceRoleRevision, UiAppearanceRoleSchemaVersion, UiAppearanceStateAxis,
    UiAppearanceStateAxisVersion, UiBackdropDeclaration, UiBackdropDeclarationDenial,
    UiBackdropExtentBasis, UiBackdropIdentity, UiBackdropMotionBasis, UiBackdropPlacement,
    UiBackdropPresenceBasis, UiBackdropScope, UiDslComponentReference, UiLogicalLength,
    UiMosaicRegionDeclarationIdentity, UiOverlayAnchor, UiOverlayPortalParticipant,
    UiOverlayRelation, UiOverlayRelationAdmissionDenial, UiOverlayRelationGraph,
    UiOverlayRelationKind, UiPortalDeclarationId, UiSemanticSurfaceDeclarationIdentity,
    UiThemeColor, UiThemeColorParseDenial, UiThemeCornerRadii, UiThemeOpacity,
    UiThemeOpacityDenial, UiThemeOutline, UiThemeSlotIdentity, UiThemeSolidStroke, UiThemeValue,
    UiThemeValueKind,
};

pub use worth_ui_runtime::facade::registry::descriptor::{
    FrozenAppearanceThemeCapabilities, FrozenAppearanceThemeCapabilitiesDenial, UiThemeDefinition,
    UiThemeDefinitionDenial, UiThemeDefinitionIdentity, UiThemeSlotCatalog,
    UiThemeSlotCatalogDenial, UiThemeSlotDeclaration, UiThemeSlotDisclosure,
    UiThemeSlotSuccessorCompatibility,
};
pub use worth_ui_runtime::facade::{
    AppearanceRoleRegistrationDenial, UiAppearanceInspectionGenerationSuccessionDenial,
};
