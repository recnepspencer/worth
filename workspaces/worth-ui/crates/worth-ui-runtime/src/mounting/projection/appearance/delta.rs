use worth_ui_host_contract::{
    UiMountedAppearanceMechanicChange, UiMountedAppearanceWork, UiMountedAppearanceWorkPosture,
};

use super::damage::damage_for_change;
use super::fact::{UiMountedAppearanceFacts, UiMountedAppearanceLoweringInput};
use super::UiMountedAppearanceLoweringDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceDeltaSummary {
    pub(super) semantic_facts_changed: u32,
    pub(super) mechanics_changed: bool,
    pub(super) output_suppressed: bool,
    pub(super) order_changed: bool,
}

impl UiMountedAppearanceDeltaSummary {
    pub(super) const fn semantic_facts_changed(self) -> u32 {
        self.semantic_facts_changed
    }

    #[cfg(test)]
    pub(super) const fn mechanics_changed(self) -> bool {
        self.mechanics_changed
    }

    pub(super) const fn output_suppressed(self) -> bool {
        self.output_suppressed
    }

    pub(super) const fn order_changed(self) -> bool {
        self.order_changed
    }
}

pub(crate) struct UiMountedAppearanceDelta {
    pub(super) work: UiMountedAppearanceWork,
    pub(super) summary: UiMountedAppearanceDeltaSummary,
}

/// A lowering checked against the current facts but not yet adopted, so a
/// denied successor leaves the sidecar untouched without a defensive copy.
pub(crate) struct UiMountedAppearancePreparedDelta {
    delta: UiMountedAppearanceDelta,
    successor: UiMountedAppearanceFacts,
    /// The facts it was checked against, named by where their rows live,
    /// which moving the sidecar leaves in place.
    basis: Option<usize>,
}

impl UiMountedAppearancePreparedDelta {
    pub(in crate::mounting::projection) const fn work(&self) -> &UiMountedAppearanceWork {
        &self.delta.work
    }
}

impl super::UiMountedAppearanceSidecar {
    pub(in crate::mounting::projection) fn prepare_mount(
        &self,
        input: UiMountedAppearanceLoweringInput,
    ) -> Result<UiMountedAppearancePreparedDelta, UiMountedAppearanceLoweringDenial> {
        let successor = super::lowering::lower(input)?;
        let delta = work(self.current.as_ref(), &successor)?;
        Ok(UiMountedAppearancePreparedDelta {
            delta,
            successor,
            basis: self.facts_basis(),
        })
    }

    pub(in crate::mounting::projection) fn prepare_reconstruct(
        &self,
        input: UiMountedAppearanceLoweringInput,
    ) -> Result<UiMountedAppearancePreparedDelta, UiMountedAppearanceLoweringDenial> {
        super::reconstruction::rebuild(self.current.as_ref(), input).map(|(delta, successor)| {
            UiMountedAppearancePreparedDelta {
                delta,
                successor,
                basis: self.facts_basis(),
            }
        })
    }

    fn facts_basis(&self) -> Option<usize> {
        self.current
            .as_ref()
            .map(|facts| facts.records().as_ptr().addr())
    }

    /// Adopts a lowering prepared against these same current facts.
    pub(in crate::mounting::projection) fn commit(
        &mut self,
        prepared: UiMountedAppearancePreparedDelta,
    ) -> UiMountedAppearanceWork {
        let UiMountedAppearancePreparedDelta {
            delta,
            successor,
            basis,
        } = prepared;
        debug_assert_eq!(
            basis,
            self.facts_basis(),
            "a prepared lowering commits onto the facts it was checked against"
        );
        self.counters.observe(&delta.work, delta.summary);
        self.last_delta = Some(delta.summary);
        self.current = Some(successor);
        delta.work
    }
}

pub(super) fn work(
    predecessor: Option<&UiMountedAppearanceFacts>,
    successor: &UiMountedAppearanceFacts,
) -> Result<UiMountedAppearanceDelta, UiMountedAppearanceLoweringDenial> {
    let changes = mechanic_changes(predecessor, successor);
    let semantic_facts_changed = semantic_facts_changed(predecessor, successor);
    let order_changed = predecessor.is_none_or(|predecessor| {
        !same_overlay_structure(
            predecessor.frame().overlay_order(),
            successor.frame().overlay_order(),
        )
    });
    let mechanics_changed = !changes.is_empty() || order_changed;
    let posture = match predecessor {
        None => UiMountedAppearanceWorkPosture::Initial,
        Some(_) if !mechanics_changed => UiMountedAppearanceWorkPosture::Unchanged,
        Some(_) => UiMountedAppearanceWorkPosture::Delta,
    };
    let damage = if mechanics_changed {
        damage_for_change(predecessor, successor, &changes, order_changed)
    } else {
        Ok(Vec::new())
    }?;
    let predecessor_manifest = match predecessor {
        Some(facts) => Some(
            worth_ui_host_contract::UiMountedAppearancePredecessorManifest::from_runtime_mounting(
                facts
                    .records()
                    .iter()
                    .map(|record| record.identity().clone()),
                facts
                    .frame()
                    .overlay_order()
                    .bottom_to_top()
                    .iter()
                    .cloned(),
            )
            .ok_or(UiMountedAppearanceLoweringDenial::WorkConstruction)?,
        ),
        None => None,
    };
    let work = UiMountedAppearanceWork::from_runtime_mounting(
        posture,
        predecessor.map(|facts| facts.frame().frame()),
        predecessor_manifest,
        successor.frame().clone(),
        changes,
        damage,
        order_changed,
    )
    .ok_or(UiMountedAppearanceLoweringDenial::WorkConstruction)?;
    Ok(UiMountedAppearanceDelta {
        work,
        summary: UiMountedAppearanceDeltaSummary {
            semantic_facts_changed,
            mechanics_changed,
            output_suppressed: predecessor.is_some() && !mechanics_changed,
            order_changed,
        },
    })
}

fn same_overlay_structure(
    predecessor: &worth_ui_host_contract::UiMountedOverlayOrderMechanic,
    successor: &worth_ui_host_contract::UiMountedOverlayOrderMechanic,
) -> bool {
    predecessor.semantic_surface() == successor.semantic_surface()
        && predecessor.bottom_to_top() == successor.bottom_to_top()
}

pub(super) fn semantic_facts_changed(
    predecessor: Option<&UiMountedAppearanceFacts>,
    successor: &UiMountedAppearanceFacts,
) -> u32 {
    let Some(predecessor) = predecessor else {
        return u32::try_from(successor.records().len()).unwrap_or(u32::MAX);
    };
    let removed = predecessor
        .records()
        .iter()
        .filter(|record| successor.record(record.identity()).is_none())
        .count();
    let changed = successor
        .records()
        .iter()
        .filter(|record| {
            predecessor
                .record(record.identity())
                .is_none_or(|previous| !previous.same_semantic_meaning(record))
        })
        .count();
    u32::try_from(removed.saturating_add(changed)).unwrap_or(u32::MAX)
}

pub(super) fn mechanic_changes(
    predecessor: Option<&UiMountedAppearanceFacts>,
    successor: &UiMountedAppearanceFacts,
) -> Vec<UiMountedAppearanceMechanicChange> {
    let Some(predecessor) = predecessor else {
        return successor
            .records()
            .iter()
            .map(|record| UiMountedAppearanceMechanicChange::Insert(record.mechanic().clone()))
            .collect();
    };
    let mut changes = predecessor
        .records()
        .iter()
        .filter_map(|record| {
            let successor_record = successor.record(record.identity());
            match successor_record {
                None => Some(UiMountedAppearanceMechanicChange::Remove(
                    record.identity().clone(),
                )),
                Some(successor_record) if !successor_record.same_physical_output(record) => Some(
                    UiMountedAppearanceMechanicChange::replacement(
                        record.identity().clone(),
                        successor_record.mechanic().clone(),
                    )
                    .expect("same record identity produces a replacement"),
                ),
                Some(_) => None,
            }
        })
        .collect::<Vec<_>>();
    changes.extend(
        successor
            .records()
            .iter()
            .filter(|record| predecessor.record(record.identity()).is_none())
            .map(|record| UiMountedAppearanceMechanicChange::Insert(record.mechanic().clone())),
    );
    changes
}
