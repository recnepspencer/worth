use std::path::PathBuf;

use worth_ui_dsl::{
    certification_support::with_unsupported_protocol, WorthUiAuthoredSourceInput,
    WorthUiDslCompiler,
};

use crate::capability::{
    CommandDescriptor, CommandId, ComponentChildPolicy, ComponentDescriptor, ComponentId,
    ComponentPropSchema, ComponentStateOwnership, UiCommandKeyCode, UiCommandModifierSet,
    UiCommandRouteDeclaration, UiCommandRouteDestination, UiCommandRouteScopeIdentity,
    UiCommandShortcutSequence, UiCommandShortcutStroke, UiIntent, UiIntentAcceptedInteractions,
    UiIntentDefinition, UiIntentId, UiIntentPayload, UiIntentPayloadFieldSet,
    UiIntentPayloadProjection, UiIntentPayloadProjectionViolation,
    UiIntentProductConsequenceFamilies, UiIntentProductConsequences, UiIntentProductOutcome,
    UiIntentRuntimeServiceDestination, UiIntentSchema, UiSemanticInteractionFamily,
};
use crate::facade::WorthUi;

use super::{
    prepare_semantic_handoff, WorthUiSemanticHandoffPreparationStop,
    WorthUiServiceDeclarationAdmissionCause,
};

struct CommandPayload;
struct CommandOutcome;
struct CommandIntent;

impl UiIntentPayload for CommandPayload {
    const SCHEMA: UiIntentSchema = UiIntentSchema::stable("dsl.command.payload", 1);
    const FIELDS: UiIntentPayloadFieldSet = UiIntentPayloadFieldSet::EMPTY;

    fn project(
        _fields: &mut UiIntentPayloadProjection<Self>,
    ) -> Result<Self, UiIntentPayloadProjectionViolation> {
        Ok(Self)
    }
}

impl UiIntentProductOutcome for CommandOutcome {
    const SCHEMA: UiIntentSchema = UiIntentSchema::stable("dsl.command.outcome", 1);
    const CONSEQUENCE_FAMILIES: UiIntentProductConsequenceFamilies =
        UiIntentProductConsequenceFamilies::NONE;

    fn into_consequences(self) -> UiIntentProductConsequences {
        UiIntentProductConsequences::none()
    }
}

impl UiIntent for CommandIntent {
    type Payload = CommandPayload;
    type ProductOutcome = CommandOutcome;

    const ID: UiIntentId = UiIntentId::stable("dsl.command.intent");
    const ACCEPTED_INTERACTIONS: UiIntentAcceptedInteractions =
        UiIntentAcceptedInteractions::new(&[UiSemanticInteractionFamily::Activate]);
}

#[test]
fn unsupported_protocol_stops_before_candidate_material_can_exist() {
    let capability_app = WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("capability authority should prepare");
    let package = WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace"))
            .with_module("app/main.wui", "component Dashboard {}"),
    )
    .expect("otherwise valid source should seal");
    let expected_identity = package.identity().clone();
    let unsupported = with_unsupported_protocol(package);
    let unsupported_protocol = unsupported.protocol();

    let denial = match prepare_semantic_handoff(unsupported, capability_app.capabilities()) {
        Ok(_) => panic!("unsupported package protocol must stop before runtime lowering"),
        Err(denial) => denial,
    };

    assert_eq!(
        denial.stop(),
        WorthUiSemanticHandoffPreparationStop::UnsupportedProtocol
    );
    assert_eq!(denial.handoff().identity(), &expected_identity);
    assert_eq!(denial.handoff().protocol(), unsupported_protocol);
    assert!(!denial.handoff().protocol().is_current());
}

#[test]
fn typed_command_dsl_must_match_the_registered_command_authority() {
    let app = command_capability_app(UiCommandKeyCode::P);
    let package = command_package("Primary+Shift+P");

    let material = prepare_semantic_handoff(package, app.capabilities())
        .expect("matching typed command declaration admits");
    let (_, _, evidence) = material.into_parts();

    assert_eq!(evidence.service_declarations().len(), 1);
    assert_eq!(
        evidence.service_declarations()[0]
            .provenance()
            .module_path(),
        "app/main.wui"
    );
}

#[test]
fn command_dsl_mismatch_stops_with_the_exact_source_declaration() {
    let app = command_capability_app(UiCommandKeyCode::P);
    let denial =
        match prepare_semantic_handoff(command_package("Primary+Shift+K"), app.capabilities()) {
            Ok(_) => panic!("DSL cannot override the frozen command shortcut"),
            Err(denial) => denial,
        };

    assert_eq!(
        denial.stop(),
        WorthUiSemanticHandoffPreparationStop::ServiceDeclaration {
            declaration_index: 0,
            cause: WorthUiServiceDeclarationAdmissionCause::CommandShortcutMismatch,
        }
    );
    assert_eq!(
        denial.handoff().service_declarations()[0]
            .provenance()
            .module_path(),
        "app/main.wui"
    );
}

#[test]
fn command_dsl_preserves_the_rust_authored_routing_policy() {
    let app = command_capability_app(UiCommandKeyCode::P);
    let package = command_package("Primary+Shift+P");
    let material = prepare_semantic_handoff(package, app.capabilities())
        .expect("matching typed command declaration admits");
    let (_, _, evidence) = material.into_parts();
    let custom = crate::declaration::UiCommandRoutingPolicy::desktop()
        .with_repeat_suppression(false)
        .with_text_input_suppression(false);
    let policy_plan = crate::declaration::UiNormalizedServicePolicyPlan::normalize(
        crate::declaration::UiServicePolicyDefaults::default().with_command_routing(custom),
        evidence.authored_service_policy_defaults(),
        evidence.runtime_service_support(),
    );

    assert_eq!(policy_plan.command_routing(), Some(custom));
}

#[test]
fn authored_component_scope_converges_across_dsl_admission_and_registered_route() {
    let app = scoped_command_capability_app("editor_control");
    let package = scoped_command_package("editor_control", true);

    prepare_semantic_handoff(package, app.capabilities())
        .expect("one authored component scope identity admits at the boundary");
}

#[test]
fn command_binding_cannot_name_an_undeclared_component_scope() {
    let app = scoped_command_capability_app("missing_control");
    let denial = match prepare_semantic_handoff(
        scoped_command_package("missing_control", false),
        app.capabilities(),
    ) {
        Ok(_) => panic!("a matching Rust string cannot counterfeit an absent authored component"),
        Err(denial) => denial,
    };

    assert_eq!(
        denial.stop(),
        WorthUiSemanticHandoffPreparationStop::ServiceDeclaration {
            declaration_index: 0,
            cause: WorthUiServiceDeclarationAdmissionCause::CommandScopeBindingUndeclared,
        }
    );
}

#[test]
fn service_dsl_demands_only_its_declared_owner_closure() {
    let app = WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .freeze()
        .expect("empty capability app freezes");
    let package = WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace")).with_module(
            "app/main.wui",
            r#"
            portal completion_menu {
              anchor editor_input
              layer transient
              dismiss escape anchor_gone
              focus first_enabled restore
              motion system_popover
            }
            selection results_selection {
              mode multiple
              identity result_key
              preserve stable_key
            }
            "#,
        ),
    )
    .expect("service source seals");

    let material =
        prepare_semantic_handoff(package, app.capabilities()).expect("service declarations admit");
    let (_, _, evidence) = material.into_parts();
    let support = evidence.runtime_service_support();
    use crate::capability::{UiRuntimeServiceFamily as Family, UiRuntimeServiceSupportPosture};

    // A portal declaration closes over its focus and motion requirements and
    // over the Scroll owner that owns the focus reveal a portal transition may
    // emit. Nothing else is demanded: command routing stays unsupported because
    // no declaration asked for it.
    for family in [
        Family::Portal,
        Family::Focus,
        Family::Motion,
        Family::Scroll,
        Family::Selection,
    ] {
        assert_eq!(
            support.posture(family),
            UiRuntimeServiceSupportPosture::Installed
        );
    }
    assert_eq!(
        support.posture(Family::CommandRouting),
        UiRuntimeServiceSupportPosture::Unsupported
    );
    let builder_defaults = crate::declaration::UiServicePolicyDefaults::default()
        .with_portal(crate::declaration::UiPortalPolicy::modal_dialog())
        .with_selection(crate::declaration::UiSelectionPolicy::single());
    let policy_plan = crate::declaration::UiNormalizedServicePolicyPlan::normalize(
        builder_defaults,
        evidence.authored_service_policy_defaults(),
        support,
    );
    assert_eq!(
        policy_plan.portal(),
        Some(crate::declaration::UiPortalPolicy::modal_dialog())
    );
    assert_eq!(
        policy_plan.selection(),
        Some(crate::declaration::UiSelectionPolicy::multiple())
    );
    // The reveal owner normalizes to the canonical nested-region policy: the
    // portal declaration demanded the owner, not a scroll policy of its own.
    assert_eq!(
        policy_plan.scroll(),
        Some(crate::declaration::UiScrollPolicy::nested_region())
    );
    assert_eq!(policy_plan.command_routing(), None);
}

fn command_capability_app(key: UiCommandKeyCode) -> crate::facade::entry::WorthUiHostNeutralApp {
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_command(
            CommandDescriptor::new(
                CommandId::new("show_palette").expect("valid command id"),
                "Show palette",
            )
            .with_default_shortcut(UiCommandShortcutSequence::single(
                UiCommandShortcutStroke::logical(
                    key,
                    UiCommandModifierSet::none().with_primary().with_shift(),
                ),
            ))
            .with_intent_destination::<CommandIntent>(),
        )
        .register_runtime_service_intent_definition(
            UiIntentDefinition::<CommandIntent>::runtime_service(
                UiIntentRuntimeServiceDestination::InvokeCommand,
            ),
        )
        .expect("command destination registers")
        .freeze()
        .expect("command capability app freezes")
}

fn command_package(shortcut: &str) -> worth_ui_dsl::WorthUiSealedSemanticPackage {
    WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace")).with_module(
            "app/main.wui",
            format!("command show_palette {{ shortcut {shortcut} scope application }}"),
        ),
    )
    .expect("command source seals")
}

fn scoped_command_capability_app(component: &str) -> crate::facade::entry::WorthUiHostNeutralApp {
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_component(ComponentDescriptor::new(
            ComponentId::new(component).expect("valid component id"),
            ComponentPropSchema::named("command.scope.fixture.props"),
            ComponentChildPolicy::no_children(),
            ComponentStateOwnership::runtime_owned(),
        ))
        .register_command(
            CommandDescriptor::new(
                CommandId::new("show_palette").expect("valid command id"),
                "Show palette",
            )
            .with_default_shortcut(UiCommandShortcutSequence::single(
                UiCommandShortcutStroke::logical(
                    UiCommandKeyCode::P,
                    UiCommandModifierSet::none().with_primary().with_shift(),
                ),
            ))
            .with_route(
                UiCommandRouteDeclaration::new(UiCommandRouteDestination::for_intent::<
                    CommandIntent,
                >())
                .for_focused_control(
                    UiCommandRouteScopeIdentity::for_authored_component(component),
                ),
            ),
        )
        .register_runtime_service_intent_definition(
            UiIntentDefinition::<CommandIntent>::runtime_service(
                UiIntentRuntimeServiceDestination::InvokeCommand,
            ),
        )
        .expect("command destination registers")
        .freeze()
        .expect("command capability app freezes")
}

fn scoped_command_package(
    binding: &str,
    declare_component: bool,
) -> worth_ui_dsl::WorthUiSealedSemanticPackage {
    let component = if declare_component {
        format!("component {binding} {{}}")
    } else {
        String::new()
    };
    WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace")).with_module(
            "app/main.wui",
            format!(
                "command show_palette {{ shortcut Primary+Shift+P; scope focused_control; binding {binding}; }} {component}"
            ),
        ),
    )
    .expect("scoped command source seals")
}
