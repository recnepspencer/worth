pub(super) mod admission_outcome;
pub(super) mod bootstrap;
pub(super) mod format_admission;
pub(super) mod initialization;
mod integrity_denial;
pub(super) mod open;
pub(super) mod request;
pub(super) mod residency_policy;
pub(super) mod transition;

pub(in crate::physical_runtime) use transition::{initialize, open};
