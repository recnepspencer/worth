//! A multi-stage edit envelope records as one history step on the document's private candidate.
mod admission;
mod adoption;
mod rollback;
mod stage_constraint;

pub use admission::WorthQueryDocumentHistoryEnvelopeIntent;
pub use stage_constraint::WorthQueryDocumentHistoryEnvelopeConstraint;
