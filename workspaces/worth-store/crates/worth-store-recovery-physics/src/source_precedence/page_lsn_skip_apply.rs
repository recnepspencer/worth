use crate::PageLsn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageLsnSkipApplyDecision {
    SkipAlreadyApplied {
        page_lsn: PageLsn,
        redo_lsn: PageLsn,
    },
    ApplyRedo {
        page_lsn: PageLsn,
        redo_lsn: PageLsn,
    },
}

impl PageLsnSkipApplyDecision {
    pub const fn decide(page_lsn: PageLsn, redo_lsn: PageLsn) -> Self {
        if page_lsn.is_at_or_beyond(redo_lsn) {
            Self::SkipAlreadyApplied { page_lsn, redo_lsn }
        } else {
            Self::ApplyRedo { page_lsn, redo_lsn }
        }
    }
}

#[cfg(test)]
mod tests {
    use worth_store_wal::LogSequenceNumber;

    use super::*;
    use crate::PageLsn;

    #[test]
    fn current_page_lsn_skips_redo() {
        let page_lsn = PageLsn::from_lsn(LogSequenceNumber::new(10));
        let redo_lsn = PageLsn::from_lsn(LogSequenceNumber::new(9));
        assert!(
            matches!(
                PageLsnSkipApplyDecision::decide(page_lsn, redo_lsn),
                PageLsnSkipApplyDecision::SkipAlreadyApplied { .. }
            ),
            "MUTANT_PREDICATE:c8-page-lsn-skip-decision-inverted"
        );
    }
}
