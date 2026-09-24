pub(super) mod admission_outcome;
pub(super) mod bootstrap;
mod current_free_space;
mod displaced_extents;
mod displaced_segments;
pub(super) mod format_admission;
pub(super) mod initialization;
mod integrity_denial;
pub(super) mod open;
mod publication_charge;
pub(super) mod request;
pub(super) mod residency_policy;
pub(super) mod transition;

pub(in crate::physical_runtime) use transition::{initialize, open};
