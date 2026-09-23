use worth_ui::facade::{
    app::{WorthUi, WorthUiApplicationPreparationDenial, WorthUiApplicationPreparationPhase},
    declaration::{
        CommandDescriptor, CommandId, MosaicChildRule, MosaicClippingPosture, MosaicFocusScopeKind,
        MosaicHitTestPosture, MosaicRegionKindDescriptor, MosaicRegionKindId,
        MosaicRegionPersistence, MosaicRegionRole, MosaicScrollOwnership, MosaicSizingBehavior,
        SurfacePlacementClass,
    },
    intent::{
        UiIntent, UiIntentAcceptedInteractions, UiIntentDefinition, UiIntentId, UiIntentPayload,
        UiIntentPayloadFieldSet, UiIntentPayloadProjection, UiIntentPayloadProjectionViolation,
        UiIntentProductConsequenceFamilies, UiIntentProductConsequences, UiIntentProductOutcome,
        UiIntentRuntimeServiceDestination, UiIntentSchema, UiSemanticInteractionFamily,
    },
    rebind::UiChangeProfile,
    service::{
        UiCommandKeyCode, UiCommandModifierSet, UiCommandRoutingPolicy, UiCommandShortcutSequence,
        UiCommandShortcutStroke, UiFocusPolicy, UiMotionPolicy, UiPortalPolicy, UiScrollPolicy,
        UiScrollWheelBehavior, UiScrollWheelBehaviorDenial, UiSelectionPolicy,
        UiServicePolicyNormalizationDenial, UI_SCROLL_WHEEL_SETTLE_TICK_CEILING,
    },
};
use worth_ui_runtime::certification_support::WorthUiRuntimeServiceInstallationCertificationExt;

struct CommandPayload;

impl UiIntentPayload for CommandPayload {
    const SCHEMA: UiIntentSchema = UiIntentSchema::stable("service.policy.command.payload", 1);
    const FIELDS: UiIntentPayloadFieldSet = UiIntentPayloadFieldSet::EMPTY;

    fn project(
        _fields: &mut UiIntentPayloadProjection<Self>,
    ) -> Result<Self, UiIntentPayloadProjectionViolation> {
        Ok(Self)
    }
}

struct CommandOutcome;

impl UiIntentProductOutcome for CommandOutcome {
    const SCHEMA: UiIntentSchema = UiIntentSchema::stable("service.policy.command.outcome", 1);
    const CONSEQUENCE_FAMILIES: UiIntentProductConsequenceFamilies =
        UiIntentProductConsequenceFamilies::NONE;

    fn into_consequences(self) -> UiIntentProductConsequences {
        UiIntentProductConsequences::none()
    }
}

struct CommandIntent;

impl UiIntent for CommandIntent {
    type Payload = CommandPayload;
    type ProductOutcome = CommandOutcome;

    const ID: UiIntentId = UiIntentId::stable("service.policy.command");
    const ACCEPTED_INTERACTIONS: UiIntentAcceptedInteractions =
        UiIntentAcceptedInteractions::new(&[UiSemanticInteractionFamily::Activate]);
}

#[test]
fn unused_policy_defaults_install_no_service_family() {
    let app = WorthUi::app()
        .with_change_profile(UiChangeProfile::platform_pulse())
        .with_portal_policy_defaults(UiPortalPolicy::modal_dialog())
        .with_focus_policy_defaults(UiFocusPolicy::workbench())
        .with_motion_policy_defaults(UiMotionPolicy::system_respecting())
        .with_command_routing_policy_defaults(UiCommandRoutingPolicy::desktop())
        .with_scroll_policy_defaults(UiScrollPolicy::nested_region())
        .with_selection_policy_defaults(UiSelectionPolicy::multiple())
        .freeze()
        .expect("policy defaults alone do not demand runtime owners");

    assert_eq!(app.service_policy_plan().installed_family_count(), 0);
    let session = launch_headless(app);
    assert_eq!(
        session
            .inspect_runtime_service_installation_for_certification()
            .installed_family_count(),
        0
    );
    drop(session.shutdown());
}

#[test]
fn installed_command_family_exposes_only_its_normalized_policy() {
    let custom = UiCommandRoutingPolicy::desktop().with_repeat_suppression(false);
    let shortcut = UiCommandShortcutSequence::single(UiCommandShortcutStroke::logical(
        UiCommandKeyCode::S,
        UiCommandModifierSet::none().with_primary(),
    ));
    let app = WorthUi::app()
        .with_change_profile(UiChangeProfile::platform_pulse())
        .with_portal_policy_defaults(UiPortalPolicy::modal_dialog())
        .with_command_routing_policy_defaults(custom)
        .register_command(
            CommandDescriptor::new(
                CommandId::new("service.policy.command").expect("fixture command ID"),
                "Run command",
            )
            .with_default_shortcut(shortcut)
            .with_intent_destination::<CommandIntent>(),
        )
        .register_runtime_service_intent_definition(worth_ui::facade::intent::UiIntentDefinition::<
            CommandIntent,
        >::runtime_service(
            UiIntentRuntimeServiceDestination::InvokeCommand,
        ))
        .expect("command runtime service registers")
        .freeze()
        .expect("installed command policy normalizes");
    let plan = app.service_policy_plan();

    assert_eq!(plan.command_routing(), Some(custom));
    assert_eq!(plan.installed_family_count(), 1);
    assert_eq!(plan.portal(), None);
    let session = launch_headless(app);
    let installed = session.inspect_runtime_service_installation_for_certification();
    assert_eq!(installed.installed_family_count(), 1);
    assert!(installed.command_routing());
    drop(session.shutdown());
}

/// A portal destination demands the Scroll owner that owns its focus reveal.
///
/// The Focus owner may emit exactly one reveal requirement against a portal
/// successor, and Scroll owns that decision. Installing Portal, Focus, and
/// Motion while leaving the reveal to whichever owner a scrolling Mosaic region
/// happened to install would compile a proposal whose reveal has no owner, no
/// occupancy lease, and no settlement acknowledgement. The Scroll owner
/// installed here is the reveal participant; it registers no scroll region and
/// performs no per-frame work until one exists.
#[test]
fn a_portal_destination_installs_the_scroll_owner_that_owns_its_focus_reveal() {
    let app = WorthUi::app()
        .with_change_profile(UiChangeProfile::platform_pulse())
        .register_runtime_service_intent_definition(
            UiIntentDefinition::<CommandIntent>::runtime_service(
                UiIntentRuntimeServiceDestination::OpenPortal,
            ),
        )
        .expect("portal runtime service registers")
        .freeze()
        .expect("portal service policy normalizes");
    let plan = app.service_policy_plan();

    assert_eq!(plan.installed_family_count(), 4);
    assert!(plan.portal().is_some());
    assert!(plan.focus().is_some());
    assert!(plan.motion().is_some());
    assert!(plan.scroll().is_some());
    assert_eq!(plan.selection(), None);
    assert_eq!(plan.command_routing(), None);
    let session = launch_headless(app);
    let installed = session.inspect_runtime_service_installation_for_certification();
    assert_eq!(installed.installed_family_count(), 4);
    assert!(installed.portal());
    assert!(installed.focus());
    assert!(installed.motion());
    assert!(installed.scroll());
    assert!(!installed.selection());
    assert!(!installed.command_routing());
    drop(session.shutdown());
}

#[test]
fn mosaic_scroll_ownership_demands_scroll_and_preserves_public_policy_defaults() {
    let custom = UiScrollPolicy::nested_region().with_remainder_bubbling(false);
    let app = WorthUi::app()
        .with_change_profile(UiChangeProfile::platform_pulse())
        .with_scroll_policy_defaults(custom)
        .register_mosaic_region_kind(scroll_region(MosaicScrollOwnership::viewport_owned()))
        .freeze()
        .expect("Mosaic scroll ownership demands the Scroll owner");

    assert_eq!(app.service_policy_plan().scroll(), Some(custom));
    let session = launch_headless(app);
    let installed = session.inspect_runtime_service_installation_for_certification();
    assert_eq!(installed.installed_family_count(), 1);
    assert!(installed.scroll());
    drop(session.shutdown());
}

#[test]
fn non_scrolling_mosaic_does_not_demand_scroll() {
    let app = WorthUi::app()
        .with_change_profile(UiChangeProfile::platform_pulse())
        .with_scroll_policy_defaults(UiScrollPolicy::nested_region())
        .register_mosaic_region_kind(scroll_region(MosaicScrollOwnership::no_scrolling()))
        .freeze()
        .expect("non-scrolling Mosaic remains owner-free");

    assert_eq!(app.service_policy_plan().scroll(), None);
    let session = launch_headless(app);
    assert!(!session
        .inspect_runtime_service_installation_for_certification()
        .scroll());
    drop(session.shutdown());
}

#[test]
fn shortcut_macro_produces_the_constructor_owned_typed_value() {
    let constructed = UiCommandShortcutSequence::single(UiCommandShortcutStroke::logical(
        UiCommandKeyCode::S,
        UiCommandModifierSet::none().with_primary().with_shift(),
    ));
    assert_eq!(worth_ui::shortcut!(Primary + Shift + S), constructed);

    let sequence = worth_ui::shortcut!((Primary + K), (Primary + C));
    assert_eq!(sequence.len(), 2);
}

/// A smooth wheel is a settle the Motion owner walks, so Motion must be there.
///
/// This app installs the Scroll owner and nothing else. A settle horizon
/// declared here names a walker that was never installed, so installation says
/// so before any session exists rather than dropping the horizon on the floor.
#[test]
fn a_smooth_wheel_without_the_motion_owner_is_denied_before_any_session_exists() {
    let smooth = UiScrollPolicy::nested_region().with_wheel_behavior(
        UiScrollWheelBehavior::smooth(120).expect("120 ticks is an admitted settle horizon"),
    );
    let denial = WorthUi::app()
        .with_change_profile(UiChangeProfile::platform_pulse())
        .with_scroll_policy_defaults(smooth)
        .register_mosaic_region_kind(scroll_region(MosaicScrollOwnership::viewport_owned()))
        .freeze();
    let Err(denial) = denial else {
        panic!("a settle horizon without the Motion owner has no walker");
    };

    assert_eq!(
        denial,
        WorthUiApplicationPreparationDenial::ServicePolicyNormalization(
            UiServicePolicyNormalizationDenial::SmoothWheelWithoutMotionOwner { settle_ticks: 120 }
        )
    );
    assert_eq!(
        denial.phase(),
        WorthUiApplicationPreparationPhase::ServicePolicyNormalization
    );
}

/// With Motion installed the same horizon reaches the normalized plan intact.
#[test]
fn a_smooth_wheel_survives_normalization_when_the_motion_owner_is_installed() {
    let smooth = UiScrollPolicy::nested_region().with_wheel_behavior(
        UiScrollWheelBehavior::smooth(120).expect("120 ticks is an admitted settle horizon"),
    );
    let app = WorthUi::app()
        .with_change_profile(UiChangeProfile::platform_pulse())
        .with_scroll_policy_defaults(smooth)
        .register_runtime_service_intent_definition(
            UiIntentDefinition::<CommandIntent>::runtime_service(
                UiIntentRuntimeServiceDestination::OpenPortal,
            ),
        )
        .expect("portal runtime service registers")
        .freeze()
        .expect("a settle horizon is admitted beside the Motion owner");
    let plan = app.service_policy_plan();

    assert!(plan.motion().is_some());
    assert_eq!(
        plan.scroll()
            .map(UiScrollPolicy::wheel_behavior)
            .and_then(UiScrollWheelBehavior::settle_ticks),
        Some(120)
    );
    let session = launch_headless(app);
    assert!(session
        .inspect_runtime_service_installation_for_certification()
        .motion());
    drop(session.shutdown());
}

/// A settle horizon is a positive, bounded number of ticks or it is no settle.
#[test]
fn a_settle_horizon_is_refused_outside_its_named_bounds() {
    assert_eq!(
        UiScrollWheelBehavior::smooth(0),
        Err(UiScrollWheelBehaviorDenial::SettleHorizonIsZero)
    );
    assert_eq!(
        UiScrollWheelBehavior::smooth(UI_SCROLL_WHEEL_SETTLE_TICK_CEILING + 1),
        Err(UiScrollWheelBehaviorDenial::SettleHorizonExceedsCeiling)
    );
    assert_eq!(
        UiScrollPolicy::nested_region()
            .wheel_behavior()
            .settle_ticks(),
        None,
        "a region that declares no wheel behavior moves on the observation"
    );
}

fn launch_headless(
    app: worth_ui::facade::app::WorthUiHostNeutralApp,
) -> worth_ui::facade::app::WorthUiActiveApplicationSession {
    worth_ui_runtime::facade::entry::WorthUiCertificationApplicationTransition::activate_headless(
        app,
    )
    .launch()
    .expect("normalized service policy launches through the production composition root")
}

fn scroll_region(ownership: MosaicScrollOwnership) -> MosaicRegionKindDescriptor {
    MosaicRegionKindDescriptor::new(
        MosaicRegionKindId::new("service.policy.scroll.region").expect("fixture region ID"),
        MosaicRegionRole::primary(),
    )
    .with_sizing_behavior(MosaicSizingBehavior::fills_available_space())
    .with_scroll_ownership(ownership)
    .with_focus_scope(MosaicFocusScopeKind::active_surface_scope())
    .with_child_rule(MosaicChildRule::accepts_surfaces())
    .with_allowed_surface_class(SurfacePlacementClass::primary_region())
    .with_persistence(MosaicRegionPersistence::restorable())
    .with_clipping(MosaicClippingPosture::clip_to_region())
    .with_hit_test(MosaicHitTestPosture::participates())
}
