//! The manifest entries one walk may charge, and the token a charge mints for
//! the read it pays for. A leaf module: the authority declared here alone
//! mints the entry limit and the charge token, and only the walk entry, this
//! module's parent, builds a budget, so a walk has exactly one.
//!
//! The module names nothing else of its crate, so a compile-fail test can
//! include it whole.

use std::num::NonZeroUsize;

use worth_foundational::{ExhaustedLimit, LimitCounts, LimitDimension};
use worth_proof::{ActionMarker, Performed};

worth_foundational::limit_authority!(pub ManifestEntryAuthority);

/// The one dimension the entry budget refuses in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestEntryBound {
    Entries,
}

impl LimitDimension for ManifestEntryBound {
    type Authority = ManifestEntryAuthority;
}

/// Entries recovery ran out of. `observed` is what the refused charge would
/// have reached.
pub type ExceededManifestEntries = ExhaustedLimit<ManifestEntryBound>;

/// Entries were charged before the read they pay for.
#[derive(Debug)]
pub struct EntriesCharged;

impl ActionMarker for EntriesCharged {}

/// The root a charge pays for: the root's own page and every page read under
/// it. Every read knows its root, and nothing else, so a charge names nothing
/// else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChargeTarget {
    generation: u64,
}

impl ChargeTarget {
    pub const fn root(generation: u64) -> Self {
        Self { generation }
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }
}

/// The one entry a root costs, found or not.
pub const ROOT_ENTRY: NonZeroUsize = NonZeroUsize::MIN;

/// A charge for the reads of one root. One charge pays for every page of its
/// root: each read of the root but the last borrows it, and the last takes
/// it by value and spends it. No read hands it back.
pub type ChargeToken = Performed<EntriesCharged, ManifestEntryAuthority, ChargeTarget>;

/// `charge`, lent to a read of root `generation`'s pages. A charge for
/// another root is a programming error.
#[track_caller]
pub fn pays_for(charge: &ChargeToken, generation: u64) {
    debug_assert_eq!(
        charge.outcome().generation(),
        generation,
        "a charge pays for the read of its own target"
    );
}

/// `charge`, spent by the last read of root `generation` it paid for.
#[track_caller]
pub fn spend(charge: ChargeToken, generation: u64) {
    pays_for(&charge, generation);
    let _ = charge.into_outcome();
}

/// Why the budget stopped a charge: its limit, or a count past every count,
/// which no limit can state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntriesStopped {
    Limit(ExceededManifestEntries),
    CountOverflow,
}

/// Leaf entries a tree read finds, admitted as each block is decoded. The
/// walk's budget charges them; a view counts them against its cap alone.
pub trait EntryAdmission {
    /// What a decoder may still find.
    fn remaining(&self) -> u64;
    /// Admits `entries` found, or refuses them with this owner's counts.
    fn admit(&mut self, entries: usize) -> Result<(), EntriesStopped>;
    /// A decoder handed `remaining` stopped, having counted
    /// `local_observed` of its own.
    fn refuse_decoded(&mut self, local_observed: u64) -> EntriesStopped;
}

/// `observed` entries past `admitted`, as this authority's limit; a count
/// within it, as no limit.
fn refuse(observed: u64, admitted: u64) -> Option<ExceededManifestEntries> {
    (observed > admitted).then(|| {
        ManifestEntryAuthority::refuse(
            ManifestEntryBound::Entries,
            LimitCounts::new(observed, admitted),
        )
    })
}

/// `observed` past what `admitted` left of `whole`, which had charged the
/// rest beside it: both counts move by what was held.
fn beside(whole: u64, observed: u64, admitted: u64) -> Option<ExceededManifestEntries> {
    let held = whole.checked_sub(admitted)?;
    refuse(observed.checked_add(held)?, whole)
}

/// The manifest entries one walk may still charge. A charge counts what the
/// workload wrote: a root, the entries a full observation finds in leaves,
/// the entries a root step's member declares, one for a point lookup. Where
/// records landed in a tree, and how many blocks a read crosses, charge
/// nothing, so one workload charges the same entries on every run.
#[derive(Debug)]
pub struct ManifestEntryBudget {
    admitted: u64,
    observed: u64,
    /// The latest refusal, for a denial that carries no counts of its own.
    refused: Option<ExceededManifestEntries>,
}

impl ManifestEntryBudget {
    /// The walk's `admitted` entries, of which `already_observed` were
    /// charged before it. Only the walk entry builds one.
    pub(super) const fn declared(admitted: u64, already_observed: u64) -> Self {
        Self {
            admitted,
            observed: already_observed,
            refused: None,
        }
    }

    /// A test's budget of `admitted` entries.
    #[cfg(test)]
    pub(crate) const fn for_test(admitted: u64, already_observed: u64) -> Self {
        Self::declared(admitted, already_observed)
    }

    /// Charges `units` before the reads of `target` they pay for, and mints
    /// the token those reads take; or refuses them with this budget's counts.
    /// A charge is at least one entry, so no token pays for a read with
    /// nothing. Entries that pay for no read are admitted, and mint nothing.
    pub fn charge(
        &mut self,
        units: NonZeroUsize,
        target: ChargeTarget,
    ) -> Result<ChargeToken, EntriesStopped> {
        self.admit(units.get())?;
        Ok(Performed::record(
            &ManifestEntryAuthority::witness(),
            target,
        ))
    }

    /// An owner handed `admitted` of this budget's entries counted
    /// `observed`.
    pub fn refuse_beside(&mut self, observed: u64, admitted: u64) -> EntriesStopped {
        self.record(beside(self.admitted, observed, admitted))
    }

    /// Records what a view of this budget refused, for a denial that carries
    /// no counts of its own.
    pub fn view_refused(&mut self, stopped: EntriesStopped) -> EntriesStopped {
        self.refused = match stopped {
            EntriesStopped::Limit(limit) => Some(limit),
            EntriesStopped::CountOverflow => None,
        };
        stopped
    }

    fn record(&mut self, limit: Option<ExceededManifestEntries>) -> EntriesStopped {
        self.refused = limit;
        limit.map_or(EntriesStopped::CountOverflow, EntriesStopped::Limit)
    }

    /// The latest refusal. Observation stops there, so the whole need can be
    /// higher.
    pub const fn refused(&self) -> Option<ExceededManifestEntries> {
        self.refused
    }
}

impl EntryAdmission for ManifestEntryBudget {
    fn remaining(&self) -> u64 {
        self.admitted.saturating_sub(self.observed)
    }

    fn admit(&mut self, entries: usize) -> Result<(), EntriesStopped> {
        let needed = u64::try_from(entries)
            .ok()
            .and_then(|entries| self.observed.checked_add(entries));
        let Some(needed) = needed else {
            return Err(self.record(None));
        };
        if let Some(limit) = refuse(needed, self.admitted) {
            return Err(self.record(Some(limit)));
        }
        self.observed = needed;
        Ok(())
    }

    fn refuse_decoded(&mut self, local_observed: u64) -> EntriesStopped {
        let limit = beside(self.admitted, local_observed, self.remaining());
        self.record(limit)
    }
}

/// One view of a walk's entries: a reread that holds no more entries than
/// recovery admits, and charges nothing. It refuses in the budget's own
/// dimension, but mints no charge token.
#[derive(Debug)]
pub struct ViewEntryCap {
    admitted: u64,
    observed: u64,
}

impl ViewEntryCap {
    /// A view of every entry `budget` admits, whatever it has charged.
    pub const fn of(budget: &ManifestEntryBudget) -> Self {
        Self {
            admitted: budget.admitted,
            observed: 0,
        }
    }

    pub const fn admitted(&self) -> u64 {
        self.admitted
    }

    /// The view holds `entries`: past its cap, the limit; within it, a count
    /// no limit states.
    pub fn refuse(&self, entries: u64) -> EntriesStopped {
        refuse(entries, self.admitted).map_or(EntriesStopped::CountOverflow, EntriesStopped::Limit)
    }
}

impl EntryAdmission for ViewEntryCap {
    fn remaining(&self) -> u64 {
        self.admitted.saturating_sub(self.observed)
    }

    fn admit(&mut self, entries: usize) -> Result<(), EntriesStopped> {
        let needed = u64::try_from(entries)
            .ok()
            .and_then(|entries| self.observed.checked_add(entries))
            .ok_or(EntriesStopped::CountOverflow)?;
        if let Some(limit) = refuse(needed, self.admitted) {
            return Err(EntriesStopped::Limit(limit));
        }
        self.observed = needed;
        Ok(())
    }

    fn refuse_decoded(&mut self, local_observed: u64) -> EntriesStopped {
        beside(self.admitted, local_observed, self.remaining())
            .map_or(EntriesStopped::CountOverflow, EntriesStopped::Limit)
    }
}

/// What a budget of `admitted` refuses for `observed`, which must be past
/// it: a test's expected limit, minted through this authority's own door.
#[cfg(test)]
pub(crate) fn manifest_entry_limit_for_test(
    observed: u64,
    admitted: u64,
) -> ExceededManifestEntries {
    refuse(observed, admitted).expect("a limit is a need past its ceiling")
}

#[cfg(test)]
#[path = "manifest_entry_budget_tests.rs"]
mod tests;
