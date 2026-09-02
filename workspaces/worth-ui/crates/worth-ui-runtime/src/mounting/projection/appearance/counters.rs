#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceCounters {
    pub(super) semantic_facts_changed: u64,
    pub(super) mounted_mechanics_changed: u64,
    pub(super) equal_output_changes_suppressed: u64,
    pub(super) damage_regions_emitted: u64,
    pub(super) text_layouts_reused: u64,
    pub(super) text_layouts_requalified: u64,
    pub(super) host_commands_added: u64,
    pub(super) host_commands_changed: u64,
    pub(super) host_commands_removed: u64,
    pub(super) overlay_orders_changed: u64,
}

impl UiMountedAppearanceCounters {
    pub(super) fn observe_text_reuse(&mut self, proof: super::UiMountedTextForegroundReuseProof) {
        if proof.paint_only() {
            self.text_layouts_reused = self.text_layouts_reused.saturating_add(1);
        } else if !proof.layout_reused() {
            self.text_layouts_requalified = self.text_layouts_requalified.saturating_add(1);
        }
    }

    pub(super) fn observe(
        &mut self,
        work: &worth_ui_host_contract::UiMountedAppearanceWork,
        summary: super::delta::UiMountedAppearanceDeltaSummary,
    ) {
        self.semantic_facts_changed = self
            .semantic_facts_changed
            .saturating_add(u64::from(summary.semantic_facts_changed()));
        if summary.output_suppressed() {
            self.equal_output_changes_suppressed =
                self.equal_output_changes_suppressed.saturating_add(1);
        }
        for change in work.changes() {
            match change {
                worth_ui_host_contract::UiMountedAppearanceMechanicChange::Insert(_) => {
                    self.host_commands_added = self.host_commands_added.saturating_add(1)
                }
                worth_ui_host_contract::UiMountedAppearanceMechanicChange::Replace { .. } => {
                    self.host_commands_changed = self.host_commands_changed.saturating_add(1)
                }
                worth_ui_host_contract::UiMountedAppearanceMechanicChange::Remove(_) => {
                    self.host_commands_removed = self.host_commands_removed.saturating_add(1)
                }
            }
        }
        self.mounted_mechanics_changed = self
            .mounted_mechanics_changed
            .saturating_add(work.changes().len() as u64);
        if summary.order_changed() {
            self.overlay_orders_changed = self.overlay_orders_changed.saturating_add(1);
        }
        self.damage_regions_emitted = self
            .damage_regions_emitted
            .saturating_add(work.damage().len() as u64);
    }
}
