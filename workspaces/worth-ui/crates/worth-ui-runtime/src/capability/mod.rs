mod diagnostics;
mod family_inventory;
mod identity;
mod registered_set;
mod registration;
mod registry;
mod snapshot;
mod support;

pub use diagnostics::{
    CapabilityDiagnosticCode, CapabilityDiagnosticRichness, CapabilityDiagnosticSeverity,
    CapabilityRegistrationDiagnostic, CapabilityRegistrationReport,
};
pub use family_inventory::{
    RegistryFamily, RegistryFamilyFacadeExposure, RegistryFamilyInventoryAudit,
    RegistryFamilyLifecyclePropagation,
};
pub use identity::{
    CapabilityIdError, CommandId, CommandProjectionId, ComponentId, IconId,
    MosaicPlacementPolicyId, MosaicRegionKindId, MosaicSizingContractId, MosaicStateOwnerScopeId,
    MosaicStateSlotId, NativeCapabilityId, PluginSlotId, RuntimeOutcomeProjectionId, SettingId,
    SurfaceId, TaskPresentationId, ThemeTokenId, ViewBindingId,
};
pub use registered_set::RegisteredCapabilitySet;
pub(crate) use registration::{
    validate_registration_candidates, RegistrationCandidate, RegistrationCandidateDiagnostic,
    RegistrationDependency, RegistrationValidationReport,
};
pub use registry::AppearanceRoleRegistrationDenial;
pub use registry::ComponentAppearanceAspectContractDenial;
#[allow(
    unused_imports,
    reason = "Gate 0 exposes frozen Mosaic seam ownership vocabulary"
)]
pub use registry::{
    AmbientHostCheck, ArbitraryKeyValueSettingBag, CommandCategory, CommandDescriptor,
    CommandProjectionCommandReference, CommandProjectionDescriptor, CommandProjectionGrouping,
    CommandProjectionIconLabelPolicy, CommandProjectionKey, CommandProjectionMeaningOverride,
    CommandProjectionMosaicScope, CommandProjectionOrdering, CommandProjectionOverflowBehavior,
    CommandProjectionReadinessDisplayPolicy, CommandProjectionShortcutVisibility,
    CommandProjectionSurface, ComponentAccessibilitySupport,
    ComponentAllocationMeasurementContract, ComponentCanvasSpatialContract, ComponentChildPolicy,
    ComponentDescriptor, ComponentExecutionLane, ComponentFocusContainerPolicy,
    ComponentFocusNavigationAxis, ComponentFocusSupport, ComponentHitTestClipContract,
    ComponentHitTestContract, ComponentHitTestInset, ComponentHitTestOrder,
    ComponentPortalChildContract, ComponentPropSchema, ComponentRealtimeOverlayContract,
    ComponentRealtimeOverlayContractDenial, ComponentRealtimeOverlayContractDenialReason,
    ComponentRealtimeOverlayPriority, ComponentSemanticTextContract,
    ComponentSemanticTextContractDenial, ComponentSemanticTextSpanContract,
    ComponentStateOwnership, ComponentStaticPaintContract, ComponentStaticPaintOrder,
    ComponentViewportAxisPlacement, ComponentViewportInset, ComponentViewportRegion,
    FrozenAppearanceRoleCapabilities, FrozenCommandCapabilities,
    FrozenCommandProjectionCapabilities, FrozenCommandProjectionEntry, FrozenComponentCapabilities,
    FrozenIconCapabilities, FrozenIconEntry, FrozenIntentDefinitionCapabilities,
    FrozenMosaicPlacementCapabilities, FrozenMosaicRegionCapabilities,
    FrozenMosaicSizingCapabilities, FrozenMosaicStateCapabilities, FrozenMosaicStateSlotEntry,
    FrozenNativeCapabilities, FrozenNativeCapabilityEntry, FrozenPluginSlotCapabilities,
    FrozenPluginSlotEntry, FrozenRuntimeOutcomeProjectionCapabilities,
    FrozenRuntimeOutcomeProjectionEntry, FrozenSettingCapabilities, FrozenSettingEntry,
    FrozenSurfaceCapabilities, FrozenTaskPresentationCapabilities, FrozenTaskPresentationEntry,
    FrozenThemeTokenCapabilities, FrozenThemeTokenEntry, FrozenViewBindingCapabilities,
    FrozenViewBindingEntry, IconAccessibilityPosture, IconColorSupport, IconDescriptor, IconFamily,
    IconKey, IconSizeSupport, IconSourceDescriptor, IconSourceKind, IconThemePosture,
    IntentDefinitionDescriptor, MeasurementConstraint, MeasurementValue, MosaicChildRule,
    MosaicClippingPosture, MosaicExteriorCorner, MosaicExteriorCornerPosture, MosaicFocusScopeKind,
    MosaicHitTestPosture, MosaicMeasurementAuthority, MosaicOverflowBehavior,
    MosaicParentGrowthBehavior, MosaicPlacementAction, MosaicPlacementConflictBehavior,
    MosaicPlacementEligibility, MosaicPlacementPersistence, MosaicPlacementPolicyDescriptor,
    MosaicPlacementReloadReconciliation, MosaicPlacementSource, MosaicPlacementSupport,
    MosaicPlacementTarget, MosaicRegionKindDescriptor, MosaicRegionPersistence, MosaicRegionRole,
    MosaicResizePermission, MosaicScrollOwnership, MosaicSeamPaintContract,
    MosaicSeamPaintContractDenial, MosaicSeamPaintOwner, MosaicSharedEdge, MosaicSizingBehavior,
    MosaicSizingContractDescriptor, MosaicSizingKind, MosaicSizingPersistence,
    MosaicStableIdentityBehavior, MosaicStateOwnerIdentity, MosaicStatePersistencePolicy,
    MosaicStateReconciliationKey, MosaicStateReplacementRule, MosaicStateSlotDescriptor,
    MosaicStateSlotKind, MosaicStateTruthPosture, MosaicViewportConstraint,
    NamedMeasurementDefinition, NamedMeasurementToken, NativeCapabilityDescriptor,
    NativeCapabilityFamily, NativeCapabilityKey, NativePlatformPosture, NativeShellAuthorityClaim,
    PluginCapabilityPermission, PluginContributionFamily, PluginSlotContributionReference,
    PluginSlotDescriptor, PluginSlotDiagnostics, PluginSlotGlobalMutationHook, PluginSlotKey,
    PluginSlotOrdering, PluginSlotSupportPosture, QueryDenialPresentation,
    RawColorOutsideTokenDefinition, RawIconAssetReference, RawLayoutMeasurementForDiagnostics,
    RawLayoutMeasurementKind, RuntimeOutcomeAffordance, RuntimeOutcomeDenialPosture,
    RuntimeOutcomeFamily, RuntimeOutcomePresentation, RuntimeOutcomeProjectionDescriptor,
    RuntimeOutcomeProjectionKey, RuntimeOutcomeRecoveryPosture, RuntimeOutcomeSourceReference,
    RuntimeOutcomeTone, SettingDefaultPosture, SettingDefaultValue, SettingDescriptor,
    SettingEditorHint, SettingKey, SettingMigrationPosture, SettingOwnershipMetadata, SettingScope,
    SettingValidationPosture, SettingValueSchema, SurfaceDescriptor, SurfaceKind,
    SurfacePlacementClass, SurfaceStateClass, TaskPresentationCancellationPosture,
    TaskPresentationDescriptor, TaskPresentationFailurePosture, TaskPresentationFamily,
    TaskPresentationKey, TaskPresentationLifecyclePosture, TaskPresentationProjectionEligibility,
    TaskPresentationRuntimeAuthorityPosture, ThemeColorValue, ThemeColorValueError,
    ThemeTokenAlias, ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenKey, ThemeTokenSource,
    ThemeTokenValue, UiApplicationEffectDestination, UiCommandContextConsumption, UiCommandKeyCode,
    UiCommandLogicalKey, UiCommandModifierSet, UiCommandPhysicalKey,
    UiCommandRegistrationGeneration, UiCommandRegistrationOwner,
    UiCommandRegistrationOwnerIdentity, UiCommandRepeatPolicy, UiCommandRouteDeclaration,
    UiCommandRouteDestination, UiCommandRoutePriority, UiCommandRouteScope,
    UiCommandRouteScopeIdentity, UiCommandShortcutKey, UiCommandShortcutPlatform,
    UiCommandShortcutSequence, UiCommandShortcutStroke, UiCommandTextInputPolicy, UiIntent,
    UiIntentAcceptedInteractions, UiIntentBoolean, UiIntentDefinition,
    UiIntentDefinitionDestination, UiIntentDefinitionRegistrationError,
    UiIntentExecutionDestination, UiIntentId, UiIntentPayload, UiIntentPayloadField,
    UiIntentPayloadFieldDescriptor, UiIntentPayloadFieldKind, UiIntentPayloadFieldSet,
    UiIntentPayloadProjection, UiIntentPayloadProjectionViolation, UiIntentPayloadSchemaViolation,
    UiIntentPayloadValueKind, UiIntentProductConsequenceFamilies, UiIntentProductConsequences,
    UiIntentProductOutcome, UiIntentRuntimeServiceDestination, UiIntentSchema, UiIntentSelection,
    UiIntentSelectionValue, UiIntentText, UiIntentTransitionDestination, UiIntentTransitionOutcome,
    UiIntentUnsigned64, UiRuntimeServiceDefinitionDestination, UiSemanticInteractionFamily,
    UiTransitionDefinitionDestination, ViewBindingDescriptor, ViewBindingFamily,
    VisibleStateBindingDeclaration, WorthUiQueryViewRegistration, WorthUiViewBindingIdentity,
    UI_INTENT_PAYLOAD_FIELD_LIMIT, UI_INTENT_PAYLOAD_TEXT_BYTE_LIMIT,
};
pub(crate) use registry::{
    AppearanceRoleAcceptedRegistrationProof, AppearanceRoleRegistry,
    AppearanceThemeAcceptedRegistrationProof, CommandAcceptedRegistrationProof,
    CommandProjectionAcceptedRegistrationProof, CommandProjectionRegistry, CommandRegistry,
    ComponentAcceptedRegistrationProof, ComponentRegistry, IconAcceptedRegistrationProof,
    IconRegistry, IntentDefinitionAcceptedRegistrationProof, IntentDefinitionRegistry,
    MosaicPlacementAcceptedRegistrationProof, MosaicPlacementRegistry,
    MosaicRegionAcceptedRegistrationProof, MosaicRegionRegistry,
    MosaicSeamPaintAcceptedRegistrationProof, MosaicSizingAcceptedRegistrationProof,
    MosaicSizingRegistry, MosaicStateSlotAcceptedRegistrationProof, MosaicStateSlotRegistry,
    NativeCapabilityAcceptedRegistrationProof, NativeCapabilityRegistry,
    PluginSlotAcceptedRegistrationProof, PluginSlotRegistry,
    RuntimeOutcomeProjectionAcceptedRegistrationProof, RuntimeOutcomeProjectionRegistry,
    SettingAcceptedRegistrationProof, SettingsRegistry, SurfaceAcceptedRegistrationProof,
    SurfaceRegistry, TaskPresentationAcceptedRegistrationProof, TaskPresentationRegistry,
    ThemeRegistry, ThemeTokenAcceptedRegistrationProof, ThemeTokenRegistry, UiIntentDefinitionSlot,
    UiIntentProjectedValue, UiIntentSemanticDigest, UiRuntimeServiceFamily,
    UiRuntimeServiceSupport, UiRuntimeServiceSupportPosture, ViewBindingAcceptedRegistrationProof,
    ViewBindingRegistry, APPEARANCE_ROLE_FAMILY_NAME, APPEARANCE_THEME_FAMILY_NAME,
    COMMAND_FAMILY_NAME, COMMAND_PROJECTION_FAMILY_NAME, COMPONENT_FAMILY_NAME, ICON_FAMILY_NAME,
    INTENT_DEFINITION_FAMILY_NAME, MOSAIC_PLACEMENT_POLICY_FAMILY_NAME,
    MOSAIC_REGION_KIND_FAMILY_NAME, MOSAIC_SEAM_PAINT_FAMILY_NAME,
    MOSAIC_SIZING_CONTRACT_FAMILY_NAME, MOSAIC_STATE_SLOT_FAMILY_NAME,
    NATIVE_CAPABILITY_FAMILY_NAME, PLUGIN_SLOT_FAMILY_NAME, RUNTIME_OUTCOME_PROJECTION_FAMILY_NAME,
    SETTING_FAMILY_NAME, SURFACE_FAMILY_NAME, TASK_PRESENTATION_FAMILY_NAME,
    THEME_TOKEN_FAMILY_NAME, VIEW_BINDING_FAMILY_NAME,
};
#[allow(
    unused_imports,
    reason = "Gate 0 retains non-current theme registry contracts"
)]
pub use registry::{
    FrozenAppearanceThemeCapabilities, FrozenAppearanceThemeCapabilitiesDenial, UiThemeDefinition,
    UiThemeDefinitionDenial, UiThemeDefinitionIdentity, UiThemeSlotCatalog,
    UiThemeSlotCatalogDenial, UiThemeSlotDeclaration, UiThemeSlotDisclosure,
    UiThemeSlotSuccessorCompatibility,
};
pub use snapshot::{
    CapabilitySnapshot, CapabilitySnapshotDigest, CapabilitySnapshotIndex, FrozenCapabilityFamily,
    SnapshotFamilyIndex, SnapshotFreezeReport, SnapshotLookupCounters, SnapshotLookupReport,
    SnapshotMetrics, SnapshotReferenceValidationReport, SnapshotReferenceViolation,
    SnapshotReferenceViolationKind, SupportSnapshot,
};
pub(crate) use snapshot::{
    CapabilitySnapshotBuilder, CapabilitySnapshotFreezeInput, CapabilitySnapshotIndexParts,
    CapabilitySupportCatalog,
};
pub use support::{
    AdmittedCapability, CapabilitySupportId, CapabilitySupportKind, CapabilitySupportPosture,
    CapabilitySupportRejection, DeferredCapability, PlatformInternalCapability, SupportRequirement,
    UnsupportedCapability,
};
