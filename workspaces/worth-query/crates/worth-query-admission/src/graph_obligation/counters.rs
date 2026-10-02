#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryGraphObligationSelectionCounters {
    installed_subject_checks: usize,
    installed_rows_examined: usize,
    selected_rows: usize,
    selector_index_probes: usize,
    canonical_preparations: usize,
    digest_derivations: usize,
}

impl WorthQueryGraphObligationSelectionCounters {
    pub(super) fn retained_selection(selected_rows: usize) -> Self {
        Self {
            selected_rows,
            ..Self::default()
        }
    }

    pub(super) fn checked_subject(&mut self) {
        self.installed_subject_checks += 1;
    }

    pub(super) fn examined_row(&mut self) {
        self.installed_rows_examined += 1;
    }

    pub(super) fn selected_row(&mut self) {
        self.selected_rows += 1;
    }

    pub const fn installed_subject_checks(self) -> usize {
        self.installed_subject_checks
    }

    pub const fn installed_rows_examined(self) -> usize {
        self.installed_rows_examined
    }

    pub const fn selected_rows(self) -> usize {
        self.selected_rows
    }

    pub const fn selector_index_probes(self) -> usize {
        self.selector_index_probes
    }

    pub const fn canonical_preparations(self) -> usize {
        self.canonical_preparations
    }

    pub const fn digest_derivations(self) -> usize {
        self.digest_derivations
    }
}
