use super::*;

#[derive(Default)]
pub(super) struct TextCoverageChanges {
    inserts: std::collections::HashSet<worth_ui_host_contract::UiMountedAppearanceMechanicIdentity>,
    replacements: Vec<worth_ui_host_contract::UiMountedAppearanceMechanicIdentity>,
    removals: Vec<worth_ui_host_contract::UiMountedAppearanceMechanicIdentity>,
}

pub(super) fn validate_replacement_set(
    fragment: &worth_ui_host_contract::UiUnpublishedAppearanceFragment,
    affinity: worth_ui_host_contract::UiMountedPresentationAffinity,
    attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    candidates: &[UiNativeFinalizedTextForeground],
) -> Result<TextCoverageChanges, Denial> {
    use worth_ui_host_contract::{
        UiMountedAppearanceMechanic as Mechanic, UiMountedAppearanceMechanicChange as Change,
        UiMountedAppearanceMechanicIdentity as Identity,
    };
    if !same_successor_affinity(fragment.presentation_affinity(), affinity)
        || fragment.work().successor().overlay_order().presentation() != attempt
    {
        return Err(Denial::AffinityMismatch);
    }
    let mut expected = std::collections::HashSet::new();
    let mut text_changes = TextCoverageChanges::default();
    for change in fragment.work().changes() {
        match change {
            Change::Insert(Mechanic::TextForeground(value)) => {
                let identity = text_identity(value);
                if !expected.insert(identity.clone()) || !text_changes.inserts.insert(identity) {
                    return Err(Denial::CommandMismatch);
                }
            }
            Change::Replace {
                predecessor,
                successor: Mechanic::TextForeground(value),
            } => {
                let identity = text_identity(value);
                if predecessor != &identity
                    || !expected.insert(identity.clone())
                    || text_changes.replacements.contains(&identity)
                {
                    return Err(Denial::CommandMismatch);
                }
                text_changes.replacements.push(identity);
            }
            Change::Remove(identity @ Identity::TextForeground { .. }) => {
                text_changes.removals.push(identity.clone());
            }
            _ => {}
        }
    }
    if expected.is_empty() && text_changes.removals.is_empty() {
        return Err(Denial::CommandMismatch);
    }
    for candidate in candidates {
        if candidate.binding() != fragment.surface_binding() {
            return Err(Denial::AffinityMismatch);
        }
        if !candidate.matches_candidates(fragment) {
            return Err(Denial::CommandMismatch);
        }
        if !expected.remove(&text_identity(candidate.mechanic())) {
            return Err(Denial::CommandMismatch);
        }
    }
    if !expected.is_empty() {
        return Err(Denial::CommandMismatch);
    }
    Ok(text_changes)
}

pub(super) fn same_successor_affinity(
    left: worth_ui_host_contract::UiMountedPresentationAffinity,
    right: worth_ui_host_contract::UiMountedPresentationAffinity,
) -> bool {
    left.successor() == right.successor()
        && left.surface() == right.surface()
        && left.binding() == right.binding()
        && left.content() == right.content()
        && left.baseline() == right.baseline()
        && left.receipt_affinity() == right.receipt_affinity()
}

pub(super) fn stage_text_coverage_changes(
    appearance: &mut UiNativeAppearanceRetained,
    fragment: &worth_ui_host_contract::UiUnpublishedAppearanceFragment,
    candidates: &[UiNativeFinalizedTextForeground],
    changes: &TextCoverageChanges,
    atlas: &UiNativeTextAtlas,
) -> Result<Vec<crate::native::presentation::appearance::UiNativeTextCoverageUndo>, Denial> {
    use worth_ui_host_contract::UiMountedAppearanceMechanic as Mechanic;
    let mut undo = Vec::new();
    let staging = (|| {
        for identity in &changes.removals {
            let native = native_text_identity(identity).ok_or(Denial::CommandMismatch)?;
            let key = appearance
                .key_for_identity(&native)
                .ok_or(Denial::CommandMismatch)?;
            undo.push(
                appearance
                    .stage_text_remove(key)
                    .map_err(|_| Denial::CommandMismatch)?,
            );
        }
        for identity in &changes.replacements {
            let candidate = candidate_for(candidates, identity)?;
            undo.push(
                appearance
                    .stage_text_replace(candidate.clone(), atlas)
                    .map_err(|_| Denial::CommandMismatch)?,
            );
        }
        let mut predecessor = None;
        for mechanic in fragment.work().successor().mechanics() {
            let Mechanic::TextForeground(value) = mechanic else {
                continue;
            };
            let identity = text_identity(value);
            if changes.inserts.contains(&identity) {
                let candidate = candidate_for(candidates, &identity)?;
                let (key, insertion) = appearance
                    .stage_text_insert(candidate.clone(), atlas, predecessor)
                    .map_err(|_| Denial::CommandMismatch)?;
                undo.push(insertion);
                predecessor = Some(key);
            } else {
                predecessor = appearance.key_for_identity(
                    &native_text_identity(&identity).ok_or(Denial::CommandMismatch)?,
                );
                if predecessor.is_none() {
                    return Err(Denial::CommandMismatch);
                }
            }
        }
        Ok(())
    })();
    if let Err(denial) = staging {
        for staged in undo.drain(..).rev() {
            appearance
                .rollback_text(staged)
                .expect("partial text coverage staging must restore its predecessor");
        }
        return Err(denial);
    }
    Ok(undo)
}

fn candidate_for<'a>(
    candidates: &'a [UiNativeFinalizedTextForeground],
    identity: &worth_ui_host_contract::UiMountedAppearanceMechanicIdentity,
) -> Result<&'a UiNativeFinalizedTextForeground, Denial> {
    candidates
        .iter()
        .find(|candidate| text_identity(candidate.mechanic()) == *identity)
        .ok_or(Denial::CommandMismatch)
}

fn text_identity(
    value: &worth_ui_host_contract::UiMountedTextForegroundAppearanceMechanic,
) -> worth_ui_host_contract::UiMountedAppearanceMechanicIdentity {
    worth_ui_host_contract::UiMountedAppearanceMechanicIdentity::TextForeground {
        target: value.node_receipt().mounted_instance(),
        command: value.command(),
        span: value.paint_span(),
    }
}

fn native_text_identity(
    identity: &worth_ui_host_contract::UiMountedAppearanceMechanicIdentity,
) -> Option<UiNativeAppearanceCommandIdentity> {
    let worth_ui_host_contract::UiMountedAppearanceMechanicIdentity::TextForeground {
        target,
        command,
        span,
    } = identity
    else {
        return None;
    };
    Some(UiNativeAppearanceCommandIdentity::TextForeground {
        target: *target,
        command: command.semantic_text_identity_parts()?,
        span_digest: span.digest(),
    })
}
