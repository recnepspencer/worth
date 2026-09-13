/// Catalog-local charges and populations. Reserved slots include active
/// callers; abandoned and updating records have separate retained charges.
#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeWorldRecoverySnapshot {
    pub(crate) installed: usize,
    pub(crate) reserved: usize,
    pub(crate) abandoned: usize,
    pub(crate) updating: usize,
    pub(crate) retained_metadata_bytes: usize,
    pub(crate) reserved_metadata_bytes: usize,
    pub(crate) costs: RuntimeWorldRecoveryCosts,
}
impl RuntimeWorldRecoverySnapshot {
    pub fn installed(&self) -> usize {
        self.installed
    }
    pub fn reserved(&self) -> usize {
        self.reserved
    }
    pub fn abandoned(&self) -> usize {
        self.abandoned
    }
    pub fn updating(&self) -> usize {
        self.updating
    }
    pub fn retained_metadata_bytes(&self) -> usize {
        self.retained_metadata_bytes
    }
    pub fn reserved_metadata_bytes(&self) -> usize {
        self.reserved_metadata_bytes
    }
    pub fn costs(&self) -> RuntimeWorldRecoveryCosts {
        self.costs
    }
}
/// Cumulative catalog transitions, never attributed to one publication.
#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeWorldRecoveryCosts {
    pub(crate) retained_records_created: u64,
    pub(crate) records_cleaned: u64,
    pub(crate) updates_started: u64,
}
impl RuntimeWorldRecoveryCosts {
    pub fn retained_records_created(&self) -> u64 {
        self.retained_records_created
    }
    pub fn records_cleaned(&self) -> u64 {
        self.records_cleaned
    }
    pub fn updates_started(&self) -> u64 {
        self.updates_started
    }
}
