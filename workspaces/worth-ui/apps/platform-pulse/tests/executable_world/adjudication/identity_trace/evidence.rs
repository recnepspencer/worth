use super::{
    float_pair, project_axis, ExecutableVisualComparisonEvidence, ExecutableVisualIdentityFailure,
    ExecutableVisualRetirementEvidence, ExecutableVisualSnapshotEvidence,
    ExecutableVisualTraceEvidence,
};
use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseVisualPointTrace, PlatformPulseVisualSnapshotCaptured,
    PlatformPulseVisualSnapshotRetired,
};

impl ExecutableVisualSnapshotEvidence {
    pub(crate) fn sequence(&self) -> u64 {
        self.sequence
    }

    pub(crate) fn snapshot(&self) -> &PlatformPulseVisualSnapshotCaptured {
        &self.snapshot
    }

    pub(crate) fn physical_extent(&self) -> [u32; 2] {
        self.snapshot.coordinates().client_physical_dimensions()
    }

    pub(crate) fn project_logical_point(
        &self,
        logical: [u32; 2],
    ) -> Result<[u32; 2], ExecutableVisualIdentityFailure> {
        let scale = float_pair(self.snapshot.coordinates().scale_bits());
        let translation = float_pair(self.snapshot.coordinates().translation_bits());
        Ok([
            project_axis(logical[0], scale[0], translation[0])?,
            project_axis(logical[1], scale[1], translation[1])?,
        ])
    }

    pub(crate) fn expected_target_region(
        &self,
    ) -> Result<[u32; 4], ExecutableVisualIdentityFailure> {
        let manifest = super::super::visual_contract_manifest::checked_in_adjudication_contract()
            .map_err(ExecutableVisualIdentityFailure::VisualContract)?;
        let region = manifest.target_region();
        let [left, top] = self.project_logical_point([region[0], region[1]])?;
        let [right, bottom] =
            self.project_logical_point([region[0] + region[2], region[1] + region[3]])?;
        Ok([left, top, right, bottom])
    }
}

impl ExecutableVisualTraceEvidence {
    pub(crate) fn sequence(&self) -> u64 {
        self.sequence
    }

    pub(crate) fn trace(&self) -> &PlatformPulseVisualPointTrace {
        &self.trace
    }
}

impl ExecutableVisualRetirementEvidence {
    pub(crate) fn sequence(self) -> u64 {
        self.sequence
    }

    pub(crate) fn retirement(self) -> PlatformPulseVisualSnapshotRetired {
        self.retirement
    }
}

impl ExecutableVisualComparisonEvidence {
    pub(crate) fn sequence(self) -> u64 {
        self.sequence
    }
}
