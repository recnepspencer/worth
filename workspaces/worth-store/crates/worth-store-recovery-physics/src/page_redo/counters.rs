#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PageRedoCounterSnapshot {
    stale_page_redo_required_count: u64,
    current_page_redo_skip_count: u64,
    generation_mismatch_denial_count: u64,
    redo_basis_mismatch_denial_count: u64,
    redo_current_page_lsn_mismatch_denial_count: u64,
    idempotent_redo_application_count: u64,
}

impl PageRedoCounterSnapshot {
    pub const fn empty() -> Self {
        Self {
            stale_page_redo_required_count: 0,
            current_page_redo_skip_count: 0,
            generation_mismatch_denial_count: 0,
            redo_basis_mismatch_denial_count: 0,
            redo_current_page_lsn_mismatch_denial_count: 0,
            idempotent_redo_application_count: 0,
        }
    }

    pub(super) const fn with_stale_page_redo_required(mut self) -> Self {
        self.stale_page_redo_required_count += 1;
        self
    }

    pub(super) const fn with_current_page_redo_skip(mut self) -> Self {
        self.current_page_redo_skip_count += 1;
        self
    }

    pub(super) const fn with_generation_mismatch_denial(mut self) -> Self {
        self.generation_mismatch_denial_count += 1;
        self
    }

    pub(super) const fn with_redo_basis_mismatch_denial(mut self) -> Self {
        self.redo_basis_mismatch_denial_count += 1;
        self
    }

    pub(super) const fn with_redo_current_page_lsn_mismatch_denial(mut self) -> Self {
        self.redo_current_page_lsn_mismatch_denial_count += 1;
        self
    }

    pub(super) const fn with_idempotent_redo_application(mut self) -> Self {
        self.idempotent_redo_application_count += 1;
        self
    }

    pub const fn stale_page_redo_required_count(self) -> u64 {
        self.stale_page_redo_required_count
    }

    pub const fn current_page_redo_skip_count(self) -> u64 {
        self.current_page_redo_skip_count
    }

    pub const fn generation_mismatch_denial_count(self) -> u64 {
        self.generation_mismatch_denial_count
    }

    pub const fn redo_basis_mismatch_denial_count(self) -> u64 {
        self.redo_basis_mismatch_denial_count
    }

    pub const fn redo_current_page_lsn_mismatch_denial_count(self) -> u64 {
        self.redo_current_page_lsn_mismatch_denial_count
    }

    pub const fn idempotent_redo_application_count(self) -> u64 {
        self.idempotent_redo_application_count
    }
}
