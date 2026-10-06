//! Recovery's own budgets: the only place that mints recovery's limits. A
//! leaf module, because the authority's declaring module and its descendants
//! can mint. Every budget in orchestration (the manifest entries a walk may
//! charge, the bytes one observation may read, the scratch a walk may hold,
//! each count recovery admits) refuses through one allowance here, with its
//! own counts.

use worth_foundational::{BudgetRefused, ExhaustedLimit, LimitCounts, LimitDimension};
use worth_proof::Performed;
use worth_store::physical_runtime::{GrantOverrun, ObservedRecoveryArtifact, ReadGrant};

use crate::entry::{PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension};

worth_proof::authority_marker!(pub RecoveryBudgetAuthority);

impl LimitDimension for PhysicalRecoveryLimitDimension {
    type Authority = RecoveryBudgetAuthority;
}

/// A budget recovery set ran out. It says nothing about the media: the same
/// media may pass under a wider limit. `observed` is what recovery needed. A
/// need past every count is no limit: the budget names that count apart.
pub(crate) type ExceededRecoveryLimit = ExhaustedLimit<PhysicalRecoveryLimitDimension>;

/// The ceiling recovery declared for one dimension, and the one door that
/// refuses past it. Only orchestration asks. It is built from the declaration
/// alone: a count another owner refused against is read `beside` it, so no
/// other component's count can become a limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RecoveryAllowance {
    dimension: PhysicalRecoveryLimitDimension,
    admitted: u64,
}

impl RecoveryAllowance {
    pub(super) const fn declared(
        limits: &PhysicalRecoveryLimitDeclaration,
        dimension: PhysicalRecoveryLimitDimension,
    ) -> Self {
        use PhysicalRecoveryLimitDimension as Dimension;
        let admitted = match dimension {
            Dimension::SelectorCandidates => limits.selector_candidates,
            Dimension::ManifestBytes => limits.manifest_bytes,
            Dimension::ManifestEntries => limits.manifest_entries,
            Dimension::WalSegments => limits.wal_segments,
            Dimension::WalFrames => limits.wal_frames,
            Dimension::WalBytes => limits.wal_bytes,
            Dimension::DistinctPagesAndExtents => limits.distinct_pages_and_extents,
            Dimension::ObservationBytes => limits.observation_bytes,
            Dimension::OperationBindings => limits.operation_bindings,
            Dimension::RedoTargets => limits.redo_targets,
            Dimension::RedoBytes => limits.redo_bytes,
            Dimension::StagingBytes => limits.staging_bytes,
            Dimension::RecoveryMemoryBytes => limits.recovery_memory_bytes,
            Dimension::DirtyFrames => limits.dirty_frames,
            Dimension::PublicationEffects => limits.publication_effects,
        };
        Self {
            dimension,
            admitted,
        }
    }

    pub(super) const fn admitted(self) -> u64 {
        self.admitted
    }

    /// A need within the allowance is returned; past it is this dimension's
    /// limit.
    pub(super) fn admit(self, needed: u64) -> Result<u64, ExceededRecoveryLimit> {
        if needed <= self.admitted {
            return Ok(needed);
        }
        let counts = LimitCounts::new(needed, self.admitted);
        let refusal =
            Performed::<BudgetRefused, _, _>::record(&RecoveryBudgetAuthority::witness(), counts);
        Err(ExhaustedLimit::refused(self.dimension, refusal))
    }

    /// `needed` past the allowance, as the limit a block reports; `None`
    /// within it.
    pub(super) fn past(self, needed: u64) -> Option<crate::entry::PhysicalRecoveryLimitFailure> {
        self.admit(needed).err().map(Into::into)
    }

    /// What a narrower allowance refused, `observed` past `admitted`, read
    /// against this whole allowance, which had already spent the rest of
    /// itself beside the narrower one: both counts move by what is held, so
    /// the distance between them stays the one the narrower allowance found.
    /// `None` where the narrower allowance was no part of this one, or a moved
    /// count passes every count: no limit can state either.
    pub(super) fn beside(self, observed: u64, admitted: u64) -> Option<ExceededRecoveryLimit> {
        let held = self.admitted.checked_sub(admitted)?;
        self.admit(observed.checked_add(held)?).err()
    }
}

/// What is left of one byte budget recovery declared, shared by the reads that
/// spend it. Each read is granted what is left and charged what it returned,
/// so the reads together never pass the whole, and one that would is refused
/// with the budget's own counts.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct RecoveryReadBudget {
    whole: RecoveryAllowance,
    spent: u64,
}

impl RecoveryReadBudget {
    pub(super) const fn declared(
        limits: &PhysicalRecoveryLimitDeclaration,
        dimension: PhysicalRecoveryLimitDimension,
    ) -> Self {
        Self {
            whole: RecoveryAllowance::declared(limits, dimension),
            spent: 0,
        }
    }

    /// The bytes granted reads returned.
    pub(super) const fn spent(&self) -> u64 {
        self.spent
    }

    /// What is left, granted to one read.
    pub(super) fn grant(&self) -> ReadGrant<PhysicalRecoveryLimitDimension> {
        let left = self
            .whole
            .admitted
            .checked_sub(self.spent)
            .expect("a budget charges no read past what it granted");
        ReadGrant::granted(
            self.whole.dimension,
            Performed::record(&RecoveryBudgetAuthority::witness(), left),
        )
    }

    /// Charges what a granted read returned. The read returned no more than
    /// its grant, which was what was left.
    pub(super) fn charge(&mut self, observed: &ObservedRecoveryArtifact) {
        let bytes = observed.bytes().map_or(0, |bytes| bytes.len() as u64);
        self.spent = self
            .spent
            .checked_add(bytes)
            .filter(|spent| *spent <= self.whole.admitted)
            .expect("a granted read returns no more than its grant");
    }

    /// A read past its grant, as this budget's limit: what the budget had
    /// spent beside the grant, plus the read's real length. `None` where that
    /// passes every count.
    pub(super) fn refuse(
        &self,
        overrun: GrantOverrun<PhysicalRecoveryLimitDimension>,
    ) -> Option<ExceededRecoveryLimit> {
        self.whole.beside(overrun.length(), overrun.granted())
    }
}

/// A test's allowance of `admitted`, as a declaration of that ceiling would
/// build it.
#[cfg(test)]
pub(super) const fn allowance_for_test(
    dimension: PhysicalRecoveryLimitDimension,
    admitted: u64,
) -> RecoveryAllowance {
    RecoveryAllowance {
        dimension,
        admitted,
    }
}

/// What an allowance of `admitted` refuses for `observed`, which must be past
/// it: a test's expected limit, minted through recovery's own door.
#[cfg(test)]
pub(crate) fn recovery_limit_for_test(
    dimension: PhysicalRecoveryLimitDimension,
    observed: u64,
    admitted: u64,
) -> ExceededRecoveryLimit {
    assert!(observed > admitted, "a limit is a need past its ceiling");
    allowance_for_test(dimension, admitted)
        .admit(observed)
        .expect_err("a need past the ceiling")
}

#[cfg(test)]
#[path = "recovery_budget/read_budget_tests.rs"]
mod read_budget_tests;

#[cfg(test)]
mod tests {
    use super::{allowance_for_test as allowance, PhysicalRecoveryLimitDimension::*};

    fn named(
        limit: super::ExceededRecoveryLimit,
    ) -> (super::PhysicalRecoveryLimitDimension, u64, u64) {
        (limit.dimension(), limit.observed(), limit.admitted())
    }

    #[test]
    fn a_need_past_the_allowance_names_its_dimension_and_both_counts() {
        for dimension in [ManifestEntries, ObservationBytes, StagingBytes, WalSegments] {
            let whole = allowance(dimension, 3);
            assert_eq!(whole.admit(3), Ok(3));
            assert_eq!(named(whole.admit(4).unwrap_err()), (dimension, 4, 3));
        }
        assert_eq!(allowance(WalBytes, u64::MAX).admit(u64::MAX), Ok(u64::MAX));
    }

    #[test]
    fn each_dimension_admits_its_own_declared_ceiling() {
        // Each of the 19 declared values is distinct, so a dimension read
        // from another's field admits the wrong ceiling.
        let mut values = [0; 19];
        for (value, index) in values.iter_mut().zip(1_000..) {
            *value = index;
        }
        let limits = crate::entry::PhysicalRecoveryLimitDeclaration::from_values_for_test(values);
        for (dimension, value) in [
            (SelectorCandidates, 1_000),
            (ManifestBytes, 1_002),
            (ManifestEntries, 1_003),
            (WalSegments, 1_004),
            (WalFrames, 1_005),
            (WalBytes, 1_006),
            (RedoTargets, 1_007),
            (RedoBytes, 1_008),
            (DistinctPagesAndExtents, 1_009),
            (OperationBindings, 1_010),
            (StagingBytes, 1_011),
            (RecoveryMemoryBytes, 1_012),
            (DirtyFrames, 1_013),
            (PublicationEffects, 1_015),
            (ObservationBytes, 1_018),
        ] {
            let whole = super::RecoveryAllowance::declared(&limits, dimension);
            assert_eq!(whole, allowance(dimension, value), "{dimension:?}");
        }
    }

    #[test]
    #[should_panic(expected = "a limit is a need past its ceiling")]
    fn a_test_limit_within_its_ceiling_is_refused() {
        super::recovery_limit_for_test(StagingBytes, 3, 3);
    }

    #[test]
    fn a_narrower_refusal_moves_both_counts_by_what_the_whole_held() {
        // Handed 60 of 100, the narrower allowance needed 70: 110 of 100.
        let whole = allowance(StagingBytes, 100);
        assert_eq!(
            whole.beside(70, 60).map(named),
            Some((StagingBytes, 110, 100))
        );
        // A narrower allowance wider than the whole is no part of it.
        assert_eq!(whole.beside(170, 160), None);
        // A moved count past every count is no limit.
        assert_eq!(whole.beside(u64::MAX, 60), None);
    }
}
