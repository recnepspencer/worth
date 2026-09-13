use std::collections::HashSet;

use worth_ui_host_contract::{
    UiHostSurfacePresentationDenial, UiMountedInstanceIdentity, UiMountedPaintCommandChange,
    UiMountedPresentationAuxiliaryState, UiMountedPresentationNodeChange,
    UiMountedPresentationNodePaint,
};

use super::super::UiHeadlessRetainedPresentation;

pub(super) struct UiHeadlessNodeMutation {
    rows: Vec<(UiMountedInstanceIdentity, Option<u64>, Option<u64>)>,
    final_count: usize,
}

impl UiHeadlessNodeMutation {
    pub(super) fn prepare(
        current: &UiHeadlessRetainedPresentation,
        command_changes: &[UiMountedPaintCommandChange],
        changes: &[UiMountedPresentationNodeChange],
        auxiliary: &UiMountedPresentationAuxiliaryState,
        capacity: usize,
    ) -> Result<Self, UiHostSurfacePresentationDenial> {
        let mut target_positions = HashSet::with_capacity(changes.len());
        let affected = changes
            .iter()
            .map(|change| change.mounted_instance())
            .collect::<HashSet<_>>();
        if affected.len() != changes.len() {
            return Err(malformed());
        }
        let mut rows = Vec::with_capacity(changes.len());
        let mut inserted = 0usize;
        let mut removed = 0usize;
        for change in changes {
            let instance = change.mounted_instance();
            let existing = current.node_positions.get(&instance).copied();
            let successor = match change {
                UiMountedPresentationNodeChange::Remove(_) => {
                    existing.ok_or_else(malformed)?;
                    removed += 1;
                    None
                }
                UiMountedPresentationNodeChange::Upsert(state) => {
                    validate_paint(current, command_changes, auxiliary, state.paint())?;
                    inserted += usize::from(existing.is_none());
                    let position = state.authored_position();
                    if !target_positions.insert(position)
                        || current
                            .node_by_position
                            .get(&position)
                            .is_some_and(|owner| !affected.contains(owner))
                    {
                        return Err(malformed());
                    }
                    Some(position)
                }
            };
            rows.push((instance, existing, successor));
        }
        let count = current
            .node_positions
            .len()
            .checked_add(inserted)
            .and_then(|count| count.checked_sub(removed))
            .ok_or_else(malformed)?;
        if count > capacity {
            return Err(UiHostSurfacePresentationDenial::CapacityExceeded);
        }
        Ok(Self {
            rows,
            final_count: count,
        })
    }

    pub(super) const fn final_count(&self) -> usize {
        self.final_count
    }

    pub(super) fn apply(&self, current: &mut UiHeadlessRetainedPresentation) {
        for (instance, before, _) in &self.rows {
            current.node_positions.remove(instance);
            if let Some(position) = before {
                current.node_by_position.remove(position);
            }
        }
        for (instance, _, position) in &self.rows {
            if let Some(position) = position {
                current.node_positions.insert(*instance, *position);
                current.node_by_position.insert(*position, *instance);
            }
        }
    }

    pub(super) fn restore(self, current: &mut UiHeadlessRetainedPresentation) {
        for (instance, _, position) in &self.rows {
            current.node_positions.remove(instance);
            if let Some(position) = position {
                current.node_by_position.remove(position);
            }
        }
        for (instance, position, _) in self.rows {
            if let Some(position) = position {
                current.node_positions.insert(instance, position);
                current.node_by_position.insert(position, instance);
            }
        }
    }
}

fn validate_paint(
    current: &UiHeadlessRetainedPresentation,
    changes: &[UiMountedPaintCommandChange],
    auxiliary: &UiMountedPresentationAuxiliaryState,
    paint: UiMountedPresentationNodePaint,
) -> Result<(), UiHostSurfacePresentationDenial> {
    match paint {
        UiMountedPresentationNodePaint::Command(identity) => {
            let _ = (current, changes, identity);
            return Err(malformed());
        }
        UiMountedPresentationNodePaint::CountOnlyBatch(index)
            if usize::from(index) >= auxiliary.paint_batch_count() =>
        {
            return Err(malformed());
        }
        UiMountedPresentationNodePaint::CountOnlyBatch(_)
        | UiMountedPresentationNodePaint::Omitted(_) => {}
    }
    Ok(())
}

fn malformed() -> UiHostSurfacePresentationDenial {
    UiHostSurfacePresentationDenial::MalformedProjection
}
