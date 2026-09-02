mod aspect;
mod attachment;
mod canonical;
mod capacity;
mod role;
mod role_authoring;
mod state_partition;
mod state_partition_compiler;
mod theme;

pub use aspect::{
    UiAppearanceAspect, UiAppearanceAspectApplicability, UiAppearanceAspectContract,
    UiAppearanceAspectContractDenial,
};
pub use attachment::{
    UiAppearanceRoleAttachmentDeclaration, UiAppearanceRoleAttachmentDeclarationDenial,
};
pub use capacity::{
    UI_APPEARANCE_BACKDROP_RELATION_CAPACITY, UI_APPEARANCE_ROLE_CAPACITY,
    UI_APPEARANCE_SLOT_USES_PER_ROLE_CAPACITY,
};
pub use role::{UiAppearanceRole, UiAppearanceRoleId};
pub use role::{
    UiAppearanceRoleApplicability, UiAppearanceRoleDeclaration, UiAppearanceRoleDeclarationDenial,
    UiAppearanceRoleIdentity, UiAppearanceRoleRevision, UiAppearanceRoleSchemaVersion,
    UiThemeSlotUse, UiThemeSlotUseDenial,
};
pub use role_authoring::{UiAppearanceRoleAuthoring, UiAppearanceRoleAuthoringDenial};
pub use state_partition::{
    UiAppearanceAxisClass, UiAppearanceAxisDomain, UiAppearanceAxisPredicate,
    UiAppearanceDecisionCell, UiAppearanceDecisionPartition, UiAppearanceDecisionPartitionDenial,
    UiAppearanceDecisionResult, UiAppearanceDecisionRule, UiAppearanceDecisionValue,
    UiAppearanceStateAxis, UiAppearanceStateAxisVersion, UI_APPEARANCE_DECISION_CELL_CAPACITY,
};
pub use state_partition_compiler::{
    UiAppearanceCell, UiAppearanceCellBuilder, UiAppearanceCellBuilderDenial,
    UiAppearanceCellValue, UiAppearancePartitionAuthoring,
};
pub use theme::{
    UiLogicalLength, UiThemeColor, UiThemeColorParseDenial, UiThemeCornerRadii, UiThemeOpacity,
    UiThemeOpacityDenial, UiThemeOutline, UiThemeSlotIdentity, UiThemeSolidStroke, UiThemeValue,
    UiThemeValueKind,
};

#[cfg(test)]
#[path = "state_partition_compiler_tests.rs"]
mod state_partition_compiler_tests;
