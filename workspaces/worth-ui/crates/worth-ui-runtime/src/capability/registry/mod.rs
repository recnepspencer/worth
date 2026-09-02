#[allow(
    dead_code,
    reason = "Gate 0 freezes the non-current appearance-role registry contract"
)]
mod appearance_role;
mod command;
mod command_projection;
mod component;
mod family_names;
mod icon;
mod intent;
mod mosaic_placement;
mod mosaic_region;
mod mosaic_sizing;
mod mosaic_state;
mod native_capability;
mod plugin_slot;
mod runtime_outcome_projection;
mod runtime_service;
mod settings;
mod surface;
mod task_presentation;
#[allow(
    dead_code,
    reason = "Gate 0 freezes the non-current appearance-theme registry contract"
)]
mod theme;
mod theme_token;
mod view_binding;

pub use appearance_role::AppearanceRoleRegistrationDenial;
pub use appearance_role::FrozenAppearanceRoleCapabilities;
pub(crate) use appearance_role::{AppearanceRoleAcceptedRegistrationProof, AppearanceRoleRegistry};
pub(crate) use command::CommandAcceptedRegistrationProof;
pub(crate) use command::CommandRegistry;
pub use command::{
    CommandCategory, CommandDescriptor, FrozenCommandCapabilities, UiCommandContextConsumption,
    UiCommandKeyCode, UiCommandLogicalKey, UiCommandModifierSet, UiCommandPhysicalKey,
    UiCommandRegistrationGeneration, UiCommandRegistrationOwner,
    UiCommandRegistrationOwnerIdentity, UiCommandRepeatPolicy, UiCommandRouteDeclaration,
    UiCommandRouteDestination, UiCommandRoutePriority, UiCommandRouteScope,
    UiCommandRouteScopeIdentity, UiCommandShortcutKey, UiCommandShortcutPlatform,
    UiCommandShortcutSequence, UiCommandShortcutStroke, UiCommandTextInputPolicy,
};
pub(crate) use command_projection::{
    CommandProjectionAcceptedRegistrationProof, CommandProjectionRegistry,
};
pub use command_projection::{
    CommandProjectionCommandReference, CommandProjectionDescriptor, CommandProjectionGrouping,
    CommandProjectionIconLabelPolicy, CommandProjectionKey, CommandProjectionMeaningOverride,
    CommandProjectionMosaicScope, CommandProjectionOrdering, CommandProjectionOverflowBehavior,
    CommandProjectionReadinessDisplayPolicy, CommandProjectionShortcutVisibility,
    CommandProjectionSurface, FrozenCommandProjectionCapabilities, FrozenCommandProjectionEntry,
};
pub(crate) use component::{
    ComponentAcceptedRegistrationProof, ComponentAppearanceAspectContractDenial, ComponentRegistry,
};
pub use component::{
    ComponentAccessibilitySupport, ComponentAllocationMeasurementContract,
    ComponentCanvasSpatialContract, ComponentChildPolicy, ComponentDescriptor,
    ComponentExecutionLane, ComponentFocusContainerPolicy, ComponentFocusNavigationAxis,
    ComponentFocusSupport, ComponentHitTestClipContract, ComponentHitTestContract,
    ComponentHitTestInset, ComponentHitTestOrder, ComponentPortalChildContract,
    ComponentPropSchema, ComponentRealtimeOverlayContract, ComponentRealtimeOverlayContractDenial,
    ComponentRealtimeOverlayContractDenialReason, ComponentRealtimeOverlayPriority,
    ComponentSemanticTextContract, ComponentSemanticTextContractDenial,
    ComponentSemanticTextSpanContract, ComponentStateOwnership, ComponentStaticPaintContract,
    ComponentStaticPaintOrder, ComponentViewportAxisPlacement, ComponentViewportInset,
    ComponentViewportRegion, FrozenComponentCapabilities,
};
pub(crate) use family_names::{
    APPEARANCE_ROLE_FAMILY_NAME, APPEARANCE_THEME_FAMILY_NAME, COMMAND_FAMILY_NAME,
    COMMAND_PROJECTION_FAMILY_NAME, COMPONENT_FAMILY_NAME, ICON_FAMILY_NAME,
    INTENT_DEFINITION_FAMILY_NAME, MOSAIC_PLACEMENT_POLICY_FAMILY_NAME,
    MOSAIC_REGION_KIND_FAMILY_NAME, MOSAIC_SEAM_PAINT_FAMILY_NAME,
    MOSAIC_SIZING_CONTRACT_FAMILY_NAME, MOSAIC_STATE_SLOT_FAMILY_NAME,
    NATIVE_CAPABILITY_FAMILY_NAME, PLUGIN_SLOT_FAMILY_NAME, RUNTIME_OUTCOME_PROJECTION_FAMILY_NAME,
    SETTING_FAMILY_NAME, SURFACE_FAMILY_NAME, TASK_PRESENTATION_FAMILY_NAME,
    THEME_TOKEN_FAMILY_NAME, VIEW_BINDING_FAMILY_NAME,
};
pub use icon::{
    FrozenIconCapabilities, FrozenIconEntry, IconAccessibilityPosture, IconColorSupport,
    IconDescriptor, IconFamily, IconKey, IconSizeSupport, IconSourceDescriptor, IconSourceKind,
    IconThemePosture, RawIconAssetReference,
};
pub(crate) use icon::{IconAcceptedRegistrationProof, IconRegistry};
pub use intent::{
    FrozenIntentDefinitionCapabilities, IntentDefinitionDescriptor, UiApplicationEffectDestination,
    UiIntent, UiIntentAcceptedInteractions, UiIntentBoolean, UiIntentDefinition,
    UiIntentDefinitionDestination, UiIntentDefinitionRegistrationError,
    UiIntentExecutionDestination, UiIntentId, UiIntentPayload, UiIntentPayloadField,
    UiIntentPayloadFieldDescriptor, UiIntentPayloadFieldKind, UiIntentPayloadFieldSet,
    UiIntentPayloadProjection, UiIntentPayloadProjectionViolation, UiIntentPayloadSchemaViolation,
    UiIntentPayloadValueKind, UiIntentProductConsequenceFamilies, UiIntentProductConsequences,
    UiIntentProductOutcome, UiIntentRuntimeServiceDestination, UiIntentSchema, UiIntentSelection,
    UiIntentSelectionValue, UiIntentText, UiIntentTransitionDestination, UiIntentTransitionOutcome,
    UiIntentUnsigned64, UiRuntimeServiceDefinitionDestination, UiSemanticInteractionFamily,
    UiTransitionDefinitionDestination, UI_INTENT_PAYLOAD_FIELD_LIMIT,
    UI_INTENT_PAYLOAD_TEXT_BYTE_LIMIT,
};
pub(crate) use intent::{
    IntentDefinitionAcceptedRegistrationProof, IntentDefinitionRegistry, UiIntentDefinitionSlot,
    UiIntentProjectedValue, UiIntentSemanticDigest,
};
pub use mosaic_placement::{
    FrozenMosaicPlacementCapabilities, MosaicPlacementAction, MosaicPlacementConflictBehavior,
    MosaicPlacementEligibility, MosaicPlacementPersistence, MosaicPlacementPolicyDescriptor,
    MosaicPlacementReloadReconciliation, MosaicPlacementSource, MosaicPlacementSupport,
    MosaicPlacementTarget, MosaicStableIdentityBehavior,
};
pub(crate) use mosaic_placement::{
    MosaicPlacementAcceptedRegistrationProof, MosaicPlacementRegistry,
};
pub use mosaic_region::{
    FrozenMosaicRegionCapabilities, MosaicChildRule, MosaicClippingPosture, MosaicExteriorCorner,
    MosaicExteriorCornerPosture, MosaicFocusScopeKind, MosaicHitTestPosture,
    MosaicRegionKindDescriptor, MosaicRegionPersistence, MosaicRegionRole, MosaicScrollOwnership,
    MosaicSeamPaintContract, MosaicSeamPaintContractDenial, MosaicSeamPaintOwner, MosaicSharedEdge,
    MosaicSizingBehavior,
};
pub(crate) use mosaic_region::{
    MosaicRegionAcceptedRegistrationProof, MosaicRegionRegistry,
    MosaicSeamPaintAcceptedRegistrationProof,
};
pub use mosaic_sizing::{
    FrozenMosaicSizingCapabilities, MeasurementConstraint, MeasurementValue,
    MosaicMeasurementAuthority, MosaicOverflowBehavior, MosaicParentGrowthBehavior,
    MosaicResizePermission, MosaicSizingContractDescriptor, MosaicSizingKind,
    MosaicSizingPersistence, MosaicViewportConstraint, NamedMeasurementDefinition,
    NamedMeasurementToken, RawLayoutMeasurementForDiagnostics, RawLayoutMeasurementKind,
};
pub(crate) use mosaic_sizing::{MosaicSizingAcceptedRegistrationProof, MosaicSizingRegistry};
pub use mosaic_state::{
    FrozenMosaicStateCapabilities, FrozenMosaicStateSlotEntry, MosaicStateOwnerIdentity,
    MosaicStatePersistencePolicy, MosaicStateReconciliationKey, MosaicStateReplacementRule,
    MosaicStateSlotDescriptor, MosaicStateSlotKind, MosaicStateTruthPosture,
};
pub(crate) use mosaic_state::{MosaicStateSlotAcceptedRegistrationProof, MosaicStateSlotRegistry};
pub use native_capability::{
    AmbientHostCheck, FrozenNativeCapabilities, FrozenNativeCapabilityEntry,
    NativeCapabilityDescriptor, NativeCapabilityFamily, NativeCapabilityKey, NativePlatformPosture,
    NativeShellAuthorityClaim,
};
pub(crate) use native_capability::{
    NativeCapabilityAcceptedRegistrationProof, NativeCapabilityRegistry,
};
pub use plugin_slot::{
    FrozenPluginSlotCapabilities, FrozenPluginSlotEntry, PluginCapabilityPermission,
    PluginContributionFamily, PluginSlotContributionReference, PluginSlotDescriptor,
    PluginSlotDiagnostics, PluginSlotGlobalMutationHook, PluginSlotKey, PluginSlotOrdering,
    PluginSlotSupportPosture,
};
pub(crate) use plugin_slot::{PluginSlotAcceptedRegistrationProof, PluginSlotRegistry};
pub use runtime_outcome_projection::{
    FrozenRuntimeOutcomeProjectionCapabilities, FrozenRuntimeOutcomeProjectionEntry,
    RuntimeOutcomeAffordance, RuntimeOutcomeDenialPosture, RuntimeOutcomeFamily,
    RuntimeOutcomePresentation, RuntimeOutcomeProjectionDescriptor, RuntimeOutcomeProjectionKey,
    RuntimeOutcomeRecoveryPosture, RuntimeOutcomeSourceReference, RuntimeOutcomeTone,
};
pub(crate) use runtime_outcome_projection::{
    RuntimeOutcomeProjectionAcceptedRegistrationProof, RuntimeOutcomeProjectionRegistry,
};
pub(crate) use runtime_service::{
    UiRuntimeServiceFamily, UiRuntimeServiceSupport, UiRuntimeServiceSupportPosture,
};
pub use settings::{
    ArbitraryKeyValueSettingBag, FrozenSettingCapabilities, FrozenSettingEntry,
    SettingDefaultPosture, SettingDefaultValue, SettingDescriptor, SettingEditorHint, SettingKey,
    SettingMigrationPosture, SettingOwnershipMetadata, SettingScope, SettingValidationPosture,
    SettingValueSchema,
};
pub(crate) use settings::{SettingAcceptedRegistrationProof, SettingsRegistry};
pub use surface::{
    FrozenSurfaceCapabilities, SurfaceDescriptor, SurfaceKind, SurfacePlacementClass,
    SurfaceStateClass,
};
pub(crate) use surface::{SurfaceAcceptedRegistrationProof, SurfaceRegistry};
pub use task_presentation::{
    FrozenTaskPresentationCapabilities, FrozenTaskPresentationEntry,
    TaskPresentationCancellationPosture, TaskPresentationDescriptor,
    TaskPresentationFailurePosture, TaskPresentationFamily, TaskPresentationKey,
    TaskPresentationLifecyclePosture, TaskPresentationProjectionEligibility,
    TaskPresentationRuntimeAuthorityPosture,
};
pub(crate) use task_presentation::{
    TaskPresentationAcceptedRegistrationProof, TaskPresentationRegistry,
};
pub(crate) use theme::{
    AppearanceThemeAcceptedRegistrationProof, FrozenAppearanceThemeCapabilities,
    FrozenAppearanceThemeCapabilitiesDenial, ThemeRegistry, UiThemeDefinition,
    UiThemeDefinitionDenial, UiThemeDefinitionIdentity, UiThemeSlotCatalog,
    UiThemeSlotCatalogDenial, UiThemeSlotDeclaration, UiThemeSlotDisclosure,
    UiThemeSlotSuccessorCompatibility,
};
pub use theme_token::{
    FrozenThemeTokenCapabilities, FrozenThemeTokenEntry, RawColorOutsideTokenDefinition,
    ThemeColorValue, ThemeColorValueError, ThemeTokenAlias, ThemeTokenDescriptor, ThemeTokenFamily,
    ThemeTokenKey, ThemeTokenSource, ThemeTokenValue,
};
pub(crate) use theme_token::{ThemeTokenAcceptedRegistrationProof, ThemeTokenRegistry};
pub use view_binding::{
    FrozenViewBindingCapabilities, FrozenViewBindingEntry, QueryDenialPresentation,
    ViewBindingDescriptor, ViewBindingFamily, VisibleStateBindingDeclaration,
    WorthUiQueryViewRegistration, WorthUiViewBindingIdentity,
};
pub(crate) use view_binding::{ViewBindingAcceptedRegistrationProof, ViewBindingRegistry};
