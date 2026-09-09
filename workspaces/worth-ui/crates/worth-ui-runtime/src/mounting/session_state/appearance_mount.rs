use super::WorthUiMountedSessionState;
use worth_ui_host_contract::UiMountedInstanceIdentity;

impl WorthUiMountedSessionState {
    /// Read the existing mount journal against the predecessor used by frame
    /// preparation. Publication alone consumes this journal; dropping a candidate
    /// leaves the same new instances available to the next attempt.
    pub(crate) fn newly_projected_instances(
        &self,
        predecessor: Option<&crate::mounting::UiPreparedMountedFrame>,
    ) -> Vec<UiMountedInstanceIdentity> {
        let previous = predecessor
            .map(|frame| frame.semantic_projection())
            .or_else(|| {
                self.identity
                    .current_projection()
                    .map(|frame| frame.semantic_projection())
            });
        self.identity
            .projection_change_snapshot()
            .changed_instances()
            .filter(|instance| match previous {
                Some(projection) => projection.node_receipt_with_probes(*instance).0.is_none(),
                // Identity-only progression and graph replacement retain an
                // appearance capsule after dropping semantic projection. Its
                // exact membership stays a predecessor, including PhysicalOnly.
                None => !self
                    .identity
                    .appearance_predecessor()
                    .is_some_and(|appearance| appearance.contains_instance(*instance)),
            })
            .collect()
    }
}
