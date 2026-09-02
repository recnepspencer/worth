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
    UiAppearanceStateAxisVersion, UiBackdropDeclarationAuthoring,
    UiBackdropDeclarationAuthoringDenial, UiStaticBackdropDeclaration, UiStaticBackdropExtent,
    UiStaticBackdropMotion, UiStaticBackdropPlacement, UiStaticBackdropPresence,
    UiStaticBackdropScope, UiStaticOverlayRelation, UiStaticOverlayRelationGraph,
    UiStaticOverlayRelationGraphDenial, UiStaticOverlayRelationKind, UiThemeColor,
    UiThemeColorParseDenial, UiThemeCornerRadii, UiThemeOpacity, UiThemeOpacityDenial,
    UiThemeOutline, UiThemeSlotIdentity, UiThemeSolidStroke, UiThemeValue, UiThemeValueKind,
};

pub use worth_ui_runtime::facade::AppearanceRoleRegistrationDenial;
