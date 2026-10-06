use super::super::recovery_budget::{ExceededRecoveryLimit, RecoveryAllowance};
use super::page_observation::{PageLimit, PageObservationFailure};
use crate::entry::{PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension};

/// The manifest entries recovery may still observe. A charge counts what the
/// workload wrote: a root, the entries a full observation finds in leaves,
/// the entries a root step's member declares, one for a point lookup. Where
/// records landed in a tree, and how many blocks a read crosses, charge
/// nothing, so one workload charges the same entries on every run.
pub(super) struct ManifestEntryBudget {
    allowance: RecoveryAllowance,
    observed: u64,
    /// The latest refusal, for a denial that carries no counts of its own.
    refused: Option<ExceededRecoveryLimit>,
}

/// Why the budget stopped a charge: its limit, or a count past every count,
/// which no limit can state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EntriesStopped {
    Limit(ExceededRecoveryLimit),
    CountOverflow,
}

impl From<EntriesStopped> for PageObservationFailure {
    fn from(stopped: EntriesStopped) -> Self {
        match stopped {
            EntriesStopped::Limit(limit) => Self::Limit(PageLimit::Recovery(limit)),
            EntriesStopped::CountOverflow => Self::CountOverflow,
        }
    }
}

/// A root was charged its one entry. Whoever enumerates a root's leaves is
/// handed this, so no full observation can leave the root's own unit out.
pub(super) struct RootUnit(());

impl ManifestEntryBudget {
    /// Recovery's declared manifest entries, of which `already_observed`
    /// were charged before this budget.
    pub(super) const fn declared(
        limits: &PhysicalRecoveryLimitDeclaration,
        already_observed: u64,
    ) -> Self {
        Self::of(
            RecoveryAllowance::declared(limits, PhysicalRecoveryLimitDimension::ManifestEntries),
            already_observed,
        )
    }

    /// A test's budget of `admitted` entries.
    #[cfg(test)]
    pub(super) const fn new(admitted: u64, already_observed: u64) -> Self {
        Self::of(
            super::super::recovery_budget::allowance_for_test(
                PhysicalRecoveryLimitDimension::ManifestEntries,
                admitted,
            ),
            already_observed,
        )
    }

    const fn of(allowance: RecoveryAllowance, already_observed: u64) -> Self {
        Self {
            allowance,
            observed: already_observed,
            refused: None,
        }
    }

    /// A fresh view of this budget's whole allowance, charged nothing yet.
    pub(super) const fn view(&self) -> Self {
        Self::of(self.allowance, 0)
    }

    pub(super) fn consume(&mut self, entries: usize) -> Result<(), PageObservationFailure> {
        self.charge(entries).map_err(Into::into)
    }

    /// Charges the one entry a root costs, before its leaves are read.
    pub(super) fn charge_root(&mut self) -> Result<RootUnit, EntriesStopped> {
        self.charge(1).map(|()| RootUnit(()))
    }

    /// Charges `entries`, or refuses them with this budget's counts.
    pub(super) fn charge(&mut self, entries: usize) -> Result<(), EntriesStopped> {
        let needed = u64::try_from(entries)
            .ok()
            .and_then(|entries| self.observed.checked_add(entries));
        let Some(needed) = needed else {
            return Err(self.refuse(None));
        };
        match self.allowance.admit(needed) {
            Ok(observed) => {
                self.observed = observed;
                Ok(())
            }
            Err(limit) => Err(self.refuse(Some(limit))),
        }
    }

    /// A decoder handed the entries this budget had left stopped, having
    /// counted `local_observed` of its own.
    pub(super) fn refuse_decoded(&mut self, local_observed: u64) -> EntriesStopped {
        let limit = self.allowance.beside(local_observed, self.remaining());
        self.refuse(limit)
    }

    /// One view, handed every entry recovery admits, holds `entries`.
    pub(super) fn refuse_view(&mut self, entries: u64) -> EntriesStopped {
        self.refuse_beside(entries, self.admitted())
    }

    /// An owner handed `admitted` of this budget's entries counted
    /// `observed`.
    pub(in crate::orchestration::planning) fn refuse_beside(
        &mut self,
        observed: u64,
        admitted: u64,
    ) -> EntriesStopped {
        let limit = self.allowance.beside(observed, admitted);
        self.refuse(limit)
    }

    fn refuse(&mut self, limit: Option<ExceededRecoveryLimit>) -> EntriesStopped {
        self.refused = limit;
        limit.map_or(EntriesStopped::CountOverflow, EntriesStopped::Limit)
    }

    /// The latest refusal. Observation stops there, so the whole need can be
    /// higher.
    pub(super) const fn refused(&self) -> Option<ExceededRecoveryLimit> {
        self.refused
    }

    /// Every entry recovery admits, whatever has been charged against it.
    pub(super) const fn admitted(&self) -> u64 {
        self.allowance.admitted()
    }

    pub(super) const fn remaining(&self) -> u64 {
        self.admitted().saturating_sub(self.observed)
    }
}

#[cfg(test)]
mod tests {
    use super::{EntriesStopped, ManifestEntryBudget};
    use crate::entry::PhysicalRecoveryLimitDimension::ManifestEntries;
    use crate::orchestration::recovery_budget::recovery_limit_for_test;

    fn past(observed: u64, admitted: u64) -> EntriesStopped {
        EntriesStopped::Limit(recovery_limit_for_test(ManifestEntries, observed, admitted))
    }

    #[test]
    fn a_refused_charge_names_the_count_it_would_have_reached() {
        let mut budget = ManifestEntryBudget::new(10, 7);
        assert_eq!(budget.charge(3), Ok(()));
        assert_eq!(budget.refused(), None);
        assert_eq!(budget.charge(1), Err(past(11, 10)));
        assert_eq!(
            budget.refused().map(EntriesStopped::Limit),
            Some(past(11, 10))
        );

        let mut budget = ManifestEntryBudget::new(10, 7);
        assert_eq!(budget.charge(5), Err(past(12, 10)));
        // Handed the 3 left, the decoder counted 4 of its own.
        assert_eq!(budget.refuse_decoded(4), past(11, 10));
        assert_eq!(
            budget.refused().map(EntriesStopped::Limit),
            Some(past(11, 10))
        );
        // Handed all 10, one view held 13.
        assert_eq!(budget.refuse_view(13), past(13, 10));
        assert_eq!(
            budget.refused().map(EntriesStopped::Limit),
            Some(past(13, 10))
        );
    }

    #[test]
    fn a_count_past_every_count_is_no_limit() {
        let mut budget = ManifestEntryBudget::new(u64::MAX - 1, u64::MAX - 2);
        assert_eq!(budget.charge(2), Err(past(u64::MAX, u64::MAX - 1)));
        // An overflow leaves no earlier refusal standing for it.
        assert_eq!(budget.charge(3), Err(EntriesStopped::CountOverflow));
        assert_eq!(budget.refused(), None);
        assert_eq!(
            budget.refuse_decoded(u64::MAX),
            EntriesStopped::CountOverflow
        );
        assert_eq!(budget.refused(), None);
    }

    #[test]
    fn a_root_is_charged_one_entry_and_refused_when_none_is_left() {
        let mut budget = ManifestEntryBudget::new(10, 8);
        assert!(budget.charge_root().is_ok());
        assert_eq!(budget.remaining(), 1);
        assert!(budget.charge_root().is_ok());
        assert_eq!(budget.remaining(), 0);
        assert_eq!(budget.charge_root().err(), Some(past(11, 10)));
    }
}
