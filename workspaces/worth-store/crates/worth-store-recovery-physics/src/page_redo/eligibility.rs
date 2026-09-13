use worth_store_physical_format::PageGenerationCell;

use super::{
    generation::same_page_generation, PageLsn, PageRedoApplicationBasis, PageRedoCounterSnapshot,
    PageRedoDenial, PageRedoDigestState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageRedoEligibilityKind {
    ApplyRedo,
    SkipAlreadyCurrent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageRedoEligibility {
    kind: PageRedoEligibilityKind,
    page_generation: PageGenerationCell,
    classified_page_lsn: PageLsn,
    redo_frontier: PageLsn,
    counters: PageRedoCounterSnapshot,
}

impl PageRedoEligibility {
    pub fn for_certification(
        page_generation: PageGenerationCell,
        classified_page_lsn: PageLsn,
        redo_frontier: PageLsn,
    ) -> Self {
        let apply_redo = !classified_page_lsn.is_at_or_beyond(redo_frontier);
        let counters = if apply_redo {
            PageRedoCounterSnapshot::empty().with_stale_page_redo_required()
        } else {
            PageRedoCounterSnapshot::empty().with_current_page_redo_skip()
        };
        Self {
            kind: if apply_redo {
                PageRedoEligibilityKind::ApplyRedo
            } else {
                PageRedoEligibilityKind::SkipAlreadyCurrent
            },
            page_generation,
            classified_page_lsn,
            redo_frontier,
            counters,
        }
    }

    pub fn apply_idempotent_redo(
        &self,
        current_page: PageRedoDigestState,
        basis: &PageRedoApplicationBasis,
    ) -> Result<PageRedoDigestState, PageRedoDenial> {
        if !same_page_generation(current_page.page_generation(), self.page_generation)
            || !same_page_generation(basis.target_generation(), self.page_generation)
        {
            return Err(PageRedoDenial::mismatched_page_generation(
                self.page_generation,
                current_page.page_generation(),
                self.counters.with_generation_mismatch_denial(),
            ));
        }
        if basis.redo_lsn() != self.redo_frontier {
            return Err(PageRedoDenial::redo_basis_lsn_mismatch(
                self.page_generation,
                self.redo_frontier,
                basis.redo_lsn(),
                self.counters.with_redo_basis_mismatch_denial(),
            ));
        }
        if current_page.page_lsn().is_at_or_beyond(self.redo_frontier) {
            return Ok(current_page);
        }
        if current_page.page_lsn() != self.classified_page_lsn {
            return Err(PageRedoDenial::redo_current_page_lsn_mismatch(
                self.page_generation,
                self.classified_page_lsn,
                current_page.page_lsn(),
                self.counters.with_redo_current_page_lsn_mismatch_denial(),
            ));
        }
        match self.kind {
            PageRedoEligibilityKind::ApplyRedo => Ok(current_page.after_redo(basis)),
            PageRedoEligibilityKind::SkipAlreadyCurrent => Ok(current_page),
        }
    }

    pub const fn record_idempotent_redo_application(&self) -> PageRedoCounterSnapshot {
        self.counters.with_idempotent_redo_application()
    }

    pub const fn kind(&self) -> PageRedoEligibilityKind {
        self.kind
    }

    pub const fn page_generation(&self) -> PageGenerationCell {
        self.page_generation
    }

    pub const fn classified_page_lsn(&self) -> PageLsn {
        self.classified_page_lsn
    }

    pub const fn redo_frontier(&self) -> PageLsn {
        self.redo_frontier
    }

    pub const fn counters(&self) -> PageRedoCounterSnapshot {
        self.counters
    }
}
