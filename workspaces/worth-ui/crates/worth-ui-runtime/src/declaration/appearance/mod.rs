mod aspect_contract;
mod attachment;
mod decision_cell;
mod decision_partition;
#[allow(
    dead_code,
    reason = "Gate 0 freezes pointer affordance vocabulary without an icon mechanic"
)]
mod pointer_affordance;
mod state_axis;
mod theme_slot_use;

pub use aspect_contract::{UiAppearanceAspect, UiAppearanceAspectContract};
pub(crate) use attachment::{UiAppearanceRoleAttachment, UiAppearanceRoleAttachmentDenial};
pub use decision_cell::{
    UiAppearanceAxisPredicate, UiAppearanceDecisionCell, UiAppearanceDecisionResult,
    UiAppearanceDecisionRule,
};
pub use decision_partition::{UiAppearanceDecisionPartition, UiAppearanceDecisionPartitionDenial};
pub(crate) use pointer_affordance::UiPointerAffordance;
pub use state_axis::{
    UiAppearanceAxisClass, UiAppearanceAxisDomain, UiAppearanceStateAxis,
    UiAppearanceStateAxisVersion,
};
pub use theme_slot_use::UiThemeSlotUse;
pub use worth_ui_dsl::{
    UiAppearanceCell, UiAppearanceCellBuilder, UiAppearanceCellBuilderDenial,
    UiAppearanceCellValue, UiAppearanceDecisionValue, UiAppearancePartitionAuthoring,
    UiAppearanceRole, UiAppearanceRoleAuthoring, UiAppearanceRoleAuthoringDenial,
    UiAppearanceRoleDeclaration, UiAppearanceRoleDeclarationDenial, UiAppearanceRoleId,
    UiAppearanceRoleIdentity, UiAppearanceRoleRevision, UiAppearanceRoleSchemaVersion,
};
