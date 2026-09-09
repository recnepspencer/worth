use super::{owner_state::UiActiveBackdropAppearanceWork, UiActiveOverlaySurfacePreparation};
use crate::runtime::motion::{UiMotionOverlayRows, UiMotionTargetIdentity};
use crate::runtime::overlay_composition::UiOverlayStackSnapshot;
use crate::runtime::portal::{
    UiPortalIdentity, UiPortalOverlayBindingOwner, UiPortalSurfaceStackSnapshot,
};
use std::collections::{BTreeMap, BTreeSet};
use worth_ui_dsl::UiPortalDeclarationId;

#[cfg(test)]
#[path = "local_source_tests.rs"]
mod tests;

pub(super) struct UiActiveOverlaySourceSnapshot {
    portals: UiPortalSurfaceStackSnapshot,
    bindings: UiPortalOverlayBindingOwner,
    motion: UiMotionOverlayRows,
    motion_targets: BTreeMap<UiMotionTargetIdentity, (UiPortalDeclarationId, UiPortalIdentity)>,
}

pub(super) fn capture(
    portal_owner: Option<&crate::runtime::portal::UiPortalRuntimeState>,
    motion: Option<&crate::runtime::motion::UiMotionRuntimeState>,
    surface: &UiActiveOverlaySurfacePreparation,
    previous: Option<&UiActiveOverlaySourceSnapshot>,
    work: &mut UiActiveBackdropAppearanceWork,
) -> (UiActiveOverlaySourceSnapshot, Vec<UiPortalIdentity>) {
    let bindings = surface.bindings.clone();
    let changed_bindings = if let Some(previous) = previous {
        let (changed, steps) = bindings.changed_portals(&previous.bindings);
        work.source_comparison_steps += steps;
        changed
    } else {
        bindings.bindings().map(|(portal, _)| portal).collect()
    };
    let dependencies = surface
        .backdrops
        .values()
        .filter_map(|declaration| {
            work.motion_declarations_visited += 1;
            match declaration.motion() {
                worth_ui_dsl::UiBackdropMotionBasis::PortalPresentation(portal) => Some(portal),
                worth_ui_dsl::UiBackdropMotionBasis::None => None,
            }
        })
        .collect::<BTreeSet<_>>();
    let mut motion_targets = BTreeMap::new();
    for declaration in dependencies {
        for portal in bindings.portals_for_declaration(declaration) {
            work.motion_bindings_visited += 1;
            motion_targets.insert(
                UiMotionTargetIdentity::from_family_owner(
                    surface.runtime_surface,
                    portal.owner().mounted_instance_identity(),
                    portal.diagnostic_value(),
                ),
                (declaration, portal),
            );
        }
    }
    let mut motion_rows = motion
        .map(|owner| owner.overlay_rows_for_surface(surface.runtime_surface))
        .unwrap_or_default();
    if let Some(row) = surface.staged_motion {
        if row.target().semantic_surface() == surface.runtime_surface {
            motion_rows.insert(row.target(), row);
        }
    }
    let portals = surface.portal_stack.clone().unwrap_or_else(|| {
        portal_owner
            .map(|owner| owner.surface_stack_snapshot(surface.runtime_surface))
            .unwrap_or_default()
    });
    if let Some(previous) = previous {
        for (target, (_, portal)) in &previous.motion_targets {
            if motion_targets.get(target).is_some()
                && portals.row(*portal).is_some()
                && motion_rows.get(target).is_none()
            {
                if let Some(row) = previous.motion.get(target).copied() {
                    motion_rows.insert(*target, row);
                }
            }
        }
    }
    (
        UiActiveOverlaySourceSnapshot {
            portals,
            bindings,
            motion: motion_rows,
            motion_targets,
        },
        changed_bindings,
    )
}

pub(super) fn localized_portal_source(
    next: &UiActiveOverlaySourceSnapshot,
    previous: Option<&UiActiveOverlaySourceSnapshot>,
    current: Option<&UiOverlayStackSnapshot>,
    changed_bindings: &[UiPortalIdentity],
    work: &mut UiActiveBackdropAppearanceWork,
) -> Result<
    (
        crate::runtime::portal::UiPortalStackSnapshot,
        Vec<UiPortalDeclarationId>,
    ),
    (),
> {
    let empty = UiPortalSurfaceStackSnapshot::default();
    let (changed, steps) = next
        .portals
        .changed_portals(previous.map_or(&empty, |source| &source.portals));
    work.source_comparison_steps += steps;
    let changed = changed
        .into_iter()
        .chain(changed_bindings.iter().copied())
        .collect::<BTreeSet<_>>();
    let mut declarations = BTreeSet::new();
    for portal in changed {
        work.source_changed_keys_visited += 1;
        let before = previous.and_then(|source| source.bindings.binding_for_portal(portal));
        let after = next.bindings.binding_for_portal(portal);
        let old_row = previous.and_then(|source| source.portals.row(portal));
        let new_row = next.portals.row(portal);
        if before != after || old_row != new_row {
            declarations.extend(before);
            declarations.extend(after);
        }
    }
    let revision = match current {
        None => 1,
        Some(current) if declarations.is_empty() => current.portal_revision(),
        Some(current) => current.portal_revision().checked_add(1).ok_or(())?,
    };
    let (snapshot, visited) = next.portals.snapshot(revision, &next.bindings);
    work.portal_source_rows_visited += visited;
    work.portal_rows_serialized += snapshot.rows().len();
    Ok((snapshot, declarations.into_iter().collect()))
}

pub(super) fn localized_motion_source(
    next: &UiActiveOverlaySourceSnapshot,
    previous: Option<&UiActiveOverlaySourceSnapshot>,
    current: Option<&UiOverlayStackSnapshot>,
    changed_bindings: &[UiPortalIdentity],
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    work: &mut UiActiveBackdropAppearanceWork,
) -> Result<
    (
        Option<crate::runtime::motion::UiMotionOverlayOwnerExport>,
        Vec<UiPortalDeclarationId>,
    ),
    (),
> {
    let empty = UiMotionOverlayRows::default();
    let (changed, comparison) = next
        .motion
        .changed_keys_with_work(previous.map_or(&empty, |source| &source.motion));
    work.source_comparison_steps += comparison.cursor_steps();
    let changed = changed
        .into_iter()
        .chain(changed_bindings.iter().map(|portal| {
            UiMotionTargetIdentity::from_family_owner(
                surface,
                portal.owner().mounted_instance_identity(),
                portal.diagnostic_value(),
            )
        }))
        .collect::<BTreeSet<_>>();
    let mut declarations = BTreeSet::new();
    for target in changed {
        work.source_changed_keys_visited += 1;
        let before = previous
            .and_then(|source| source.motion_targets.get(&target))
            .copied();
        let after = next.motion_targets.get(&target).copied();
        let old_row = previous.and_then(|source| source.motion.get(&target));
        let new_row = next.motion.get(&target);
        if before != after || old_row != new_row {
            if old_row.is_some() {
                declarations.extend(before.map(|(declaration, _)| declaration));
            }
            if new_row.is_some() {
                declarations.extend(after.map(|(declaration, _)| declaration));
            }
        }
    }
    let rows = next
        .motion_targets
        .keys()
        .filter_map(|target| {
            work.motion_targets_looked_up += 1;
            next.motion.get(target).copied()
        })
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return Ok((None, declarations.into_iter().collect()));
    }
    let revision = match current {
        None => 1,
        Some(current) if declarations.is_empty() => current.motion_revision().unwrap_or(1),
        Some(current) => current
            .motion_revision()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(())?,
    };
    Ok((
        Some(crate::runtime::motion::UiMotionOverlayOwnerExport::from_rows(revision, rows)),
        declarations.into_iter().collect(),
    ))
}
