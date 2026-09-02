#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiPortalShutdownReport {
    closed_records: usize,
    /// Always zero for this owner: the portal owner holds no physically issued
    /// work. Indeterminate exit terminals belong to the presentation/host-truth
    /// coordinator and are reported by its own shutdown path.
    abandoned_indeterminate_records: usize,
    final_active_records: usize,
}

impl super::UiPortalRuntimeState {
    pub(crate) fn shutdown(&mut self) -> UiPortalShutdownReport {
        // The live table holds only active portals, so every record terminalizes.
        let closed_records = self.records.len();
        for record in self.records.values_mut() {
            terminalize(record);
        }
        self.last_closed = None;
        if closed_records > 0 {
            self.revision = self.revision.saturating_add(1);
        }
        self.records.clear();
        self.stack_order.clear();
        self.clear_closed_requests();
        UiPortalShutdownReport {
            closed_records,
            abandoned_indeterminate_records: 0,
            final_active_records: self.active_count(),
        }
    }
}

/// Terminalizing before release keeps the released record self-describing for a
/// reader holding it, even though the table drops it immediately afterwards.
fn terminalize(record: &mut super::UiPortalRecord) {
    record.posture = crate::runtime::portal::UiPortalLifecyclePosture::Closed;
    record.dismissal = Some(crate::runtime::portal::UiPortalDismissalCause::ApplicationShutdown);
    record.placement = None;
}

impl UiPortalShutdownReport {
    pub(crate) const fn closed_records(self) -> usize {
        self.closed_records
    }

    pub(crate) const fn abandoned_indeterminate_records(self) -> usize {
        self.abandoned_indeterminate_records
    }

    pub(crate) const fn final_active_records(self) -> usize {
        self.final_active_records
    }
}
