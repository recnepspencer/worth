//! What selected rows say of a claim that one row makes about others.

use super::super::blob_record::BlobFact;
use super::super::OfflinePhysicalDamageCause as Cause;
use super::graph::dependency_uncertainty;
use super::{damage, Outcome, Selected};

/// The verdict of one or more checks, each made on the rows it reads. A
/// contradiction proven by frames that were read stands whatever else is
/// unobserved; a claim is undecided only when nothing contradicts it and a row
/// it needs could not be observed or is itself undecided.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Proof {
    Holds,
    /// Carries the outcome of the first needed row that is not decided.
    Undecided(Outcome),
    /// Frames that were read contradict the claim, or a row it needs is
    /// damaged, or is absent from a walk that saw every row that could be it.
    Contradicted,
}

impl Proof {
    pub(super) fn of(holds: bool) -> Self {
        if holds {
            Self::Holds
        } else {
            Self::Contradicted
        }
    }

    /// The verdict of a check that reads the one row `sought` found.
    pub(super) fn on_row(
        selected: &[Selected],
        sought: Result<usize, Proof>,
        holds: impl FnOnce(&BlobFact) -> bool,
    ) -> Self {
        match needed_row(selected, sought) {
            Ok(row) => Self::of(holds(row.fact)).and(row.standing),
            Err(verdict) => verdict,
        }
    }

    /// Both verdicts together.
    pub(super) fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::Contradicted, _) | (_, Self::Contradicted) => Self::Contradicted,
            (Self::Undecided(outcome), _) | (_, Self::Undecided(outcome)) => {
                Self::Undecided(outcome)
            }
            (Self::Holds, Self::Holds) => Self::Holds,
        }
    }

    /// The outcome of the row that made the claim.
    pub(super) fn outcome(self, contradiction: Cause) -> Outcome {
        match self {
            Self::Holds => Outcome::Intact,
            Self::Undecided(outcome) => outcome,
            Self::Contradicted => damage(contradiction),
        }
    }
}

/// A row a check reads.
pub(super) struct Needed<'a> {
    /// What the row's frame says.
    pub(super) fact: &'a BlobFact,
    /// `Holds` for an intact row. A row whose frame was read but which the
    /// walk left undecided is `Undecided` with its outcome: what its frame
    /// says against a claim is still a contradiction, because the row is
    /// either intact or damage that the claim depends on.
    pub(super) standing: Proof,
}

/// The row a check reads, or the verdict that the row gives the check by
/// being unobserved, damaged or absent. `sought` is a `RowIndex` lookup, which
/// says what an absent row means.
pub(super) fn needed_row(
    selected: &[Selected],
    sought: Result<usize, Proof>,
) -> Result<Needed<'_>, Proof> {
    let row = selected.get(sought?).ok_or(Proof::Contradicted)?;
    let standing = match dependency_uncertainty(row) {
        Some(outcome) => Proof::Undecided(outcome),
        None if row.outcome == Outcome::Intact => Proof::Holds,
        None => return Err(Proof::Contradicted),
    };
    match (row.fact.as_ref(), standing) {
        (Some(fact), standing) => Ok(Needed { fact, standing }),
        (None, Proof::Holds) => Err(Proof::Contradicted),
        (None, unobserved) => Err(unobserved),
    }
}
