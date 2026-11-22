#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryArtifactNativeAccessCounters {
    pub authority_checks: usize,
    pub lifecycle_checks: usize,
    pub layout_checks: usize,
    pub requested_field_checks: usize,
    pub provider_session_checks: usize,
    pub provider_contacts: usize,
    pub row_batch_contacts: usize,
    pub field_slice_contacts: usize,
    pub chunk_contacts: usize,
    pub projection_contacts: usize,
    pub scalar_calls: usize,
    pub rows_exposed: usize,
    pub values_exposed: usize,
    pub source_bytes: usize,
    pub result_bytes: usize,
    pub peak_result_capacity_bytes: usize,
    pub generic_row_clones: usize,
}

impl WorthQueryArtifactNativeAccessCounters {
    pub fn accumulate(&mut self, increment: Self) {
        self.authority_checks += increment.authority_checks;
        self.lifecycle_checks += increment.lifecycle_checks;
        self.layout_checks += increment.layout_checks;
        self.requested_field_checks += increment.requested_field_checks;
        self.provider_session_checks += increment.provider_session_checks;
        self.provider_contacts += increment.provider_contacts;
        self.row_batch_contacts += increment.row_batch_contacts;
        self.field_slice_contacts += increment.field_slice_contacts;
        self.chunk_contacts += increment.chunk_contacts;
        self.projection_contacts += increment.projection_contacts;
        self.scalar_calls += increment.scalar_calls;
        self.rows_exposed += increment.rows_exposed;
        self.values_exposed += increment.values_exposed;
        self.source_bytes += increment.source_bytes;
        self.result_bytes += increment.result_bytes;
        self.peak_result_capacity_bytes = self
            .peak_result_capacity_bytes
            .max(increment.peak_result_capacity_bytes);
        self.generic_row_clones += increment.generic_row_clones;
    }
}
