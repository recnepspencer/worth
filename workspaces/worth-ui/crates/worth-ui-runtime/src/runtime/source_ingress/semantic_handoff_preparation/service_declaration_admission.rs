use std::collections::{BTreeMap, BTreeSet};

use crate::capability::{
    CapabilitySnapshot, CommandId, UiCommandModifierSet, UiCommandRouteScope,
    UiCommandShortcutSequence,
};

use super::{
    WorthUiSemanticHandoffEvidence, WorthUiSemanticHandoffPreparationStop,
    WorthUiServiceDeclarationAdmissionCause,
};

pub(super) fn admit_service_declarations(
    handoff: &WorthUiSemanticHandoffEvidence,
    snapshot: &CapabilitySnapshot,
) -> Result<(), WorthUiSemanticHandoffPreparationStop> {
    let mut identities = BTreeSet::new();
    let mut family_policies = BTreeMap::new();
    for (declaration_index, declaration) in handoff.service_declarations().iter().enumerate() {
        let family = declaration.meaning().family();
        if !identities.insert((family, declaration.identity())) {
            return Err(service_stop(
                declaration_index,
                WorthUiServiceDeclarationAdmissionCause::DuplicateIdentity,
            ));
        }
        // The wheel horizon is judged before the policy digest is taken: lowering
        // an authored policy relies on every horizon having been admitted.
        if let worth_ui_dsl::WorthUiServiceDeclarationMeaning::Scroll(scroll) =
            declaration.meaning()
        {
            admit_scroll_wheel(declaration_index, scroll.wheel())?;
        }
        if let Some(policy) = declaration.policy_digest() {
            if family_policies
                .insert(family, policy)
                .is_some_and(|prior| prior != policy)
            {
                return Err(service_stop(
                    declaration_index,
                    WorthUiServiceDeclarationAdmissionCause::ConflictingFamilyPolicy,
                ));
            }
        }
        if let worth_ui_dsl::WorthUiServiceDeclarationMeaning::Command(command) =
            declaration.meaning()
        {
            admit_command(declaration_index, command, handoff, snapshot)?;
        }
    }
    Ok(())
}

fn admit_command(
    declaration_index: usize,
    command: &worth_ui_dsl::WorthUiCommandDeclaration,
    handoff: &WorthUiSemanticHandoffEvidence,
    snapshot: &CapabilitySnapshot,
) -> Result<(), WorthUiSemanticHandoffPreparationStop> {
    let command_id = CommandId::new(command.identity()).map_err(|_| {
        service_stop(
            declaration_index,
            WorthUiServiceDeclarationAdmissionCause::InvalidCommandIdentity,
        )
    })?;
    let descriptor = snapshot.commands().get(&command_id).ok_or_else(|| {
        service_stop(
            declaration_index,
            WorthUiServiceDeclarationAdmissionCause::CommandNotRegistered,
        )
    })?;
    let shortcut = descriptor.default_shortcut().ok_or_else(|| {
        service_stop(
            declaration_index,
            WorthUiServiceDeclarationAdmissionCause::CommandShortcutMissing,
        )
    })?;
    if !shortcut_matches(shortcut, command.shortcut()) {
        return Err(service_stop(
            declaration_index,
            WorthUiServiceDeclarationAdmissionCause::CommandShortcutMismatch,
        ));
    }
    let route = descriptor.route().ok_or_else(|| {
        service_stop(
            declaration_index,
            WorthUiServiceDeclarationAdmissionCause::CommandRouteMissing,
        )
    })?;
    if route.scope() != route_scope(command.scope()) {
        return Err(service_stop(
            declaration_index,
            WorthUiServiceDeclarationAdmissionCause::CommandScopeMismatch,
        ));
    }
    let authored_binding = command
        .scope_identity()
        .map(crate::capability::UiCommandRouteScopeIdentity::for_authored_component);
    if authored_binding.is_some_and(|binding| !handoff.declares_command_scope(binding)) {
        return Err(service_stop(
            declaration_index,
            WorthUiServiceDeclarationAdmissionCause::CommandScopeBindingUndeclared,
        ));
    }
    if route.scope_identity() != authored_binding {
        return Err(service_stop(
            declaration_index,
            WorthUiServiceDeclarationAdmissionCause::CommandScopeBindingMismatch,
        ));
    }
    Ok(())
}

fn shortcut_matches(
    shortcut: UiCommandShortcutSequence,
    authored: &[worth_ui_dsl::WorthUiCommandShortcutStrokeSpec],
) -> bool {
    shortcut.len() == authored.len()
        && shortcut
            .strokes()
            .iter()
            .zip(authored)
            .all(|(registered, authored)| {
                !registered.key().is_physical()
                    && registered.key().code().canonical_name() == authored.key().canonical_name()
                    && modifiers_match(registered.modifiers(), authored.modifiers())
            })
}

fn modifiers_match(
    registered: UiCommandModifierSet,
    authored: &[worth_ui_dsl::WorthUiCommandModifier],
) -> bool {
    use worth_ui_dsl::WorthUiCommandModifier as Modifier;
    registered.primary() == authored.contains(&Modifier::Primary)
        && registered.shift() == authored.contains(&Modifier::Shift)
        && registered.control() == authored.contains(&Modifier::Control)
        && registered.alt() == authored.contains(&Modifier::Alt)
        && registered.meta() == authored.contains(&Modifier::Meta)
}

const fn route_scope(scope: worth_ui_dsl::WorthUiCommandScope) -> UiCommandRouteScope {
    match scope {
        worth_ui_dsl::WorthUiCommandScope::Application => UiCommandRouteScope::Application,
        worth_ui_dsl::WorthUiCommandScope::Surface => UiCommandRouteScope::Surface,
        worth_ui_dsl::WorthUiCommandScope::ActiveRegion => UiCommandRouteScope::ActiveRegion,
        worth_ui_dsl::WorthUiCommandScope::FocusedControl => UiCommandRouteScope::FocusedControl,
        worth_ui_dsl::WorthUiCommandScope::ActivePortal => UiCommandRouteScope::ActivePortal,
    }
}

/// A smooth wheel is only admissible with a horizon the runtime honours; the
/// DSL owns the spelling, the runtime owns the range.
fn admit_scroll_wheel(
    declaration_index: usize,
    wheel: worth_ui_dsl::WorthUiScrollWheelPolicy,
) -> Result<(), WorthUiSemanticHandoffPreparationStop> {
    match wheel {
        worth_ui_dsl::WorthUiScrollWheelPolicy::Immediate => Ok(()),
        worth_ui_dsl::WorthUiScrollWheelPolicy::Smooth { settle_ticks } => {
            crate::declaration::UiScrollWheelBehavior::smooth(settle_ticks)
                .map(|_| ())
                .map_err(|denial| {
                    service_stop(
                        declaration_index,
                        WorthUiServiceDeclarationAdmissionCause::ScrollWheelHorizon(denial),
                    )
                })
        }
    }
}

const fn service_stop(
    declaration_index: usize,
    cause: WorthUiServiceDeclarationAdmissionCause,
) -> WorthUiSemanticHandoffPreparationStop {
    WorthUiSemanticHandoffPreparationStop::ServiceDeclaration {
        declaration_index,
        cause,
    }
}
