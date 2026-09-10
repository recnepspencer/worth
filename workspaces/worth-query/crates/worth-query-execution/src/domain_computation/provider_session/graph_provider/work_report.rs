#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryProviderWorkReport {
    completed_work_units: u64,
    applied_effect_count: u64,
    produced_artifact_count: usize,
    retained_artifact_count: usize,
    disposed_artifact_count: usize,
    scratch_bytes: usize,
    retained_bytes: usize,
    output_retained_bytes: usize,
}

impl WorthQueryProviderWorkReport {
    pub const fn new(
        completed_work_units: u64,
        applied_effect_count: u64,
        scratch_bytes: usize,
        retained_bytes: usize,
    ) -> Self {
        Self {
            completed_work_units,
            applied_effect_count,
            produced_artifact_count: 0,
            retained_artifact_count: 0,
            disposed_artifact_count: 0,
            scratch_bytes,
            retained_bytes,
            output_retained_bytes: 0,
        }
    }

    pub fn with_artifact_disposition(
        mut self,
        produced: usize,
        retained: usize,
        disposed: usize,
    ) -> Option<Self> {
        if retained.checked_add(disposed) != Some(produced) {
            return None;
        }
        self.produced_artifact_count = produced;
        self.retained_artifact_count = retained;
        self.disposed_artifact_count = disposed;
        Some(self)
    }

    pub const fn completed_work_units(self) -> u64 {
        self.completed_work_units
    }

    pub const fn applied_effect_count(self) -> u64 {
        self.applied_effect_count
    }

    pub const fn produced_artifact_count(self) -> usize {
        self.produced_artifact_count
    }

    pub const fn retained_artifact_count(self) -> usize {
        self.retained_artifact_count
    }

    pub const fn disposed_artifact_count(self) -> usize {
        self.disposed_artifact_count
    }

    pub const fn scratch_bytes(self) -> usize {
        self.scratch_bytes
    }

    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }

    pub(crate) const fn with_output_retention(mut self, retained_bytes: usize) -> Self {
        self.output_retained_bytes = retained_bytes;
        self
    }

    pub const fn output_retained_bytes(self) -> usize {
        self.output_retained_bytes
    }
}
