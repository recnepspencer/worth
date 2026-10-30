use super::owner;
pub(crate) use std::cell::Cell;
pub(crate) use std::convert::Infallible;

pub(crate) use worth_proof::contracts::*;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RecoveryTerminal {
    Resolved,
    Disposed,
}

impl TerminalState for RecoveryTerminal {
    fn label(&self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Disposed => "disposed",
        }
    }
}

pub(crate) struct Generations(pub(crate) Cell<u64>);

impl FreshnessSource for Generations {
    type Sample = u64;
    type Error = Infallible;

    fn sample(&self) -> Result<Self::Sample, Self::Error> {
        Ok(self.0.get())
    }
}

pub(crate) struct DeadlinePolicy;

impl FreshnessPolicy<Generations, u64> for DeadlinePolicy {
    fn classify(&self, deadline: &u64, observed_at: &u64) -> FreshnessVerdict {
        if observed_at < deadline {
            FreshnessVerdict::Current
        } else {
            FreshnessVerdict::AuthorityRevalidationRequired
        }
    }
}

pub(crate) struct Commit;
impl ActionMarker for Commit {}

pub(crate) struct Undo;
impl ActionMarker for Undo {}
impl InverseOf<Commit> for Undo {}

/// A redo lane reachable only by consuming proved inversion.
///
/// Takes the proof by value: the gate is the parameter, so there is no
/// `already_undone` flag a caller could set. Causality is not current
/// authority — an owner would still re-admit against live policy here.
pub(crate) fn redo(_proved: Inverts<Commit, owner::ContractAuthority>) -> &'static str {
    "redone"
}

/// A successor lane gated on the undo having actually run.
///
/// `DerivedFrom` borrows its predecessor, so one performed action may license
/// several successors without re-performing it.
pub(crate) fn publish(_derived: DerivedFrom<Undo, owner::ContractAuthority>) -> &'static str {
    "published"
}
