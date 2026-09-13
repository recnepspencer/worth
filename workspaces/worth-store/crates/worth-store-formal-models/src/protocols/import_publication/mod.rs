//! Prospective import-publication law.
//!
//! Store does not currently expose a production publication owner for this
//! protocol. The tests here check only the model's own transition law and must
//! not be cited as production conformance evidence.

mod action;
mod model;
#[cfg(test)]
mod tests;

pub use action::ImportPublicationAction;
pub use model::{ImportPublicationModel, ImportPublicationModelDenial, ImportPublicationState};
