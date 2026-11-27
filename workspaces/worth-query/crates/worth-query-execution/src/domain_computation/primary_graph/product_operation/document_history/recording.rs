//! One recording core for every kind of recorded step (plan r3 §3.3).
mod court_doors_for_test;
mod key;
mod kind;
mod reservation;

pub use court_doors_for_test::history_envelope_identities_for_test;
pub(super) use key::{
    envelope_meaning, group_meaning, CorrectionParts, JournalKey, JournalNamespace,
};
pub(super) use kind::RecordingKind;
pub(super) use reservation::Attempt;
