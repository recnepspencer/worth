//! Courts for the revalidation demand: bringing an unchanged record back under
//! judgement.
//!
//! The demand exists because a candidate can change the meaning records are
//! judged under without touching the records themselves, and rules only ever
//! run over what a candidate touches. These courts hold the demand to exactly
//! that: it must make a rule judge a record it would otherwise never see, it
//! must not change the record, and it must be bounded and refused like any
//! other demand against existing state.

mod accounting;
mod boundaries;
mod fixtures;
mod judgement;
mod locus_parity;
mod slot_identity;
