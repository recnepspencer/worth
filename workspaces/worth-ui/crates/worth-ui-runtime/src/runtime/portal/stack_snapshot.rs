#![allow(
    dead_code,
    reason = "Gate 1 retains the Portal stack snapshot for later overlay composition"
)]

use super::UiPortalStackOrdinal;
use crate::runtime::persistent_index::UiPersistentOrdMap;

#[derive(Clone, Default)]
pub(crate) struct UiPortalSurfaceStackSnapshot {
    rows: UiPersistentOrdMap<super::UiPortalIdentity, UiPortalStackRow>,
    order: UiPersistentOrdMap<UiPortalStackOrdinal, super::UiPortalIdentity>,
}

impl UiPortalSurfaceStackSnapshot {
    pub(crate) fn for_transition(
        mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        transition: &super::UiPreparedPortalServiceTransition,
        retain_exit: bool,
    ) -> Option<Self> {
        if transition.closes_portal() {
            for portal in std::iter::once(transition.portal())
                .chain(transition.closed_descendants().iter().copied())
            {
                let Some(mut row) = self.rows.get(&portal).copied() else {
                    continue;
                };
                if retain_exit {
                    row.lifecycle = super::UiPortalLifecyclePosture::Closing;
                    self.rows.insert(portal, row);
                } else {
                    self.rows.remove(&portal);
                    self.order.remove(&row.ordinal);
                }
            }
        }
        if transition.opens_portal() && transition.request().semantic_surface() == surface {
            let placement = transition.placement()?;
            let ordinal = transition.stack_ordinal()?;
            let portal = transition.portal();
            let row = UiPortalStackRow {
                portal,
                parent: placement.layer().parent(),
                surface,
                ordinal,
                lifecycle: super::UiPortalLifecyclePosture::Visible,
            };
            self.rows.insert(portal, row);
            self.order.insert(ordinal, portal);
        }
        Some(self)
    }

    pub(crate) fn changed_portals(&self, previous: &Self) -> (Vec<super::UiPortalIdentity>, usize) {
        let (changed, work) = self.rows.changed_keys_with_work(&previous.rows);
        (changed, work.cursor_steps())
    }

    pub(crate) fn row(&self, portal: super::UiPortalIdentity) -> Option<&UiPortalStackRow> {
        self.rows.get(&portal)
    }

    pub(crate) fn topmost_portal(&self) -> Option<super::UiPortalIdentity> {
        self.order.last_key_value().map(|(_, portal)| *portal)
    }

    pub(crate) fn snapshot(
        &self,
        revision: u64,
        bindings: &super::UiPortalOverlayBindingOwner,
    ) -> (UiPortalStackSnapshot, usize) {
        let mut visited = 0;
        let mut rows = if self.rows.len() == bindings.binding_count() {
            self.order
                .iter()
                .map(|(_, portal)| {
                    visited += 1;
                    *self
                        .rows
                        .get(portal)
                        .expect("stack order retains its exact row")
                })
                .collect::<Vec<_>>()
        } else {
            let mut rows = bindings
                .bindings()
                .filter_map(|(portal, _)| {
                    visited += 1;
                    self.rows.get(&portal).copied()
                })
                .collect::<Vec<_>>();
            rows.sort_by_key(|row| row.ordinal);
            rows
        };
        rows.retain(|row| bindings.binding_for_portal(row.portal).is_some());
        (
            UiPortalStackSnapshot {
                owner_revision: revision,
                rows: rows.into_boxed_slice(),
            },
            visited,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiPortalStackRow {
    portal: super::UiPortalIdentity,
    parent: Option<super::UiPortalIdentity>,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ordinal: UiPortalStackOrdinal,
    lifecycle: super::UiPortalLifecyclePosture,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiPortalStackSnapshot {
    owner_revision: u64,
    rows: Box<[UiPortalStackRow]>,
}

impl super::UiPortalRuntimeState {
    pub(crate) fn surface_stack_snapshot(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> UiPortalSurfaceStackSnapshot {
        self.surface_stacks
            .get(&surface)
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn remove_surface_stack_row(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        portal: super::UiPortalIdentity,
    ) {
        if let Some(stack) = self.surface_stacks.get_mut(&surface) {
            if let Some(row) = stack.rows.get(&portal).copied() {
                stack.order.remove(&row.ordinal);
                stack.rows.remove(&portal);
            }
            if stack.rows.is_empty() {
                self.surface_stacks.remove(&surface);
            }
        }
    }

    pub(super) fn refresh_surface_stack(
        &mut self,
        portal: super::UiPortalIdentity,
        record: &super::state::UiPortalRecord,
    ) {
        if let Some(previous) = self
            .records
            .get(&portal)
            .map(|record| record.semantic_surface)
        {
            if previous != record.semantic_surface {
                self.remove_surface_stack_row(previous, portal);
            }
        }
        if record.posture == super::UiPortalLifecyclePosture::Closed {
            self.remove_surface_stack_row(record.semantic_surface, portal);
            return;
        }
        let row = UiPortalStackRow {
            portal,
            parent: record
                .placement
                .and_then(|placement| placement.prepared().layer().parent()),
            surface: record.semantic_surface,
            ordinal: record.stack_ordinal,
            lifecycle: record.posture,
        };
        let stack = self
            .surface_stacks
            .entry(record.semantic_surface)
            .or_default();
        if stack.rows.get(&portal) == Some(&row) {
            return;
        }
        if let Some(previous) = stack.rows.get(&portal) {
            stack.order.remove(&previous.ordinal);
        }
        stack.order.insert(row.ordinal, portal);
        stack.rows.insert(portal, row);
    }

    pub(crate) fn stack_snapshot(&self) -> UiPortalStackSnapshot {
        assert_eq!(
            self.stack_order.len_for_snapshot(),
            self.records.len(),
            "Portal stack order must cover exactly every live record"
        );
        let rows = self
            .stack_order
            .iter()
            .map(|(_, portal)| {
                let record = self
                    .records
                    .get(portal)
                    .expect("Portal order index retains every live record");
                UiPortalStackRow {
                    portal: *portal,
                    parent: record
                        .placement
                        .and_then(|placement| placement.prepared().layer().parent()),
                    surface: record.semantic_surface,
                    ordinal: record.stack_ordinal,
                    lifecycle: record.posture,
                }
            })
            .collect::<Vec<_>>();
        UiPortalStackSnapshot {
            owner_revision: self.revision(),
            rows: rows.into_boxed_slice(),
        }
    }
}

impl UiPortalStackSnapshot {
    pub(crate) fn empty() -> Self {
        Self {
            owner_revision: 1,
            rows: Box::new([]),
        }
    }

    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }
    pub(crate) fn rows(&self) -> &[UiPortalStackRow] {
        &self.rows
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        owner_revision: u64,
        rows: impl IntoIterator<
            Item = (
                super::UiPortalIdentity,
                Option<super::UiPortalIdentity>,
                worth_ui_host_contract::UiSemanticSurfaceIdentity,
                u64,
                super::UiPortalLifecyclePosture,
            ),
        >,
    ) -> Self {
        let mut rows = rows
            .into_iter()
            .map(
                |(portal, parent, surface, ordinal, lifecycle)| UiPortalStackRow {
                    portal,
                    parent,
                    surface,
                    ordinal: UiPortalStackOrdinal::minted(ordinal),
                    lifecycle,
                },
            )
            .collect::<Vec<_>>();
        rows.sort_by_key(|row| row.ordinal);
        Self {
            owner_revision,
            rows: rows.into_boxed_slice(),
        }
    }
}

impl UiPortalStackRow {
    pub(crate) const fn portal(self) -> super::UiPortalIdentity {
        self.portal
    }
    pub(crate) const fn parent(self) -> Option<super::UiPortalIdentity> {
        self.parent
    }
    pub(crate) const fn surface(self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.surface
    }
    pub(crate) const fn ordinal(self) -> UiPortalStackOrdinal {
        self.ordinal
    }
    pub(crate) const fn lifecycle(self) -> super::UiPortalLifecyclePosture {
        self.lifecycle
    }
}
