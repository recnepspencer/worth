//! Rustdoc presence ratchet for the stable facade names in `facades.toml`.
//!
//! Every name a facade snapshot row lists is resolved to its defining item,
//! and that definition must carry rustdoc unless the committed debt file
//! `tools/boundary-check/snapshots/facade-doc-debt.toml` still lists it. The
//! debt only shrinks: paid or unexported entries fail until deleted.
//!
//! - [`resolution`] owns source-level name resolution to definitions
//! - [`doc_presence`] owns whether one definition is documented, and where
//! - [`observation`] owns the per-row, per-name documentation state
//! - [`debt_snapshot`] owns the committed debt file
//! - [`ratchet`] owns BC8004 and BC8005

mod debt_snapshot;
mod doc_presence;
mod observation;
mod ratchet;
mod resolution;

#[cfg(test)]
mod tests;

pub(crate) use debt_snapshot::{debt_from_observation, debt_path, render_debt};
pub(crate) use observation::{observe_facade_docs, FacadeDocObservation};
pub(crate) use ratchet::facade_doc_diagnostics;
