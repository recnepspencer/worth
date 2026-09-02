use super::UiPortalStackOrdinal;

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
    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }
    pub(crate) fn rows(&self) -> &[UiPortalStackRow] {
        &self.rows
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
