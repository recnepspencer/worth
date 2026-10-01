//! Selected-root release of one proof-addressed published generation.
//!
//! The issuer proof names a semantic generation; this module only derives
//! physical custody from the protected C5 root. No decoded blob frame grants
//! release authority on its own.

mod selection;

pub(super) use selection::certify_pending_drop;
pub(super) use selection::observe_selected_descriptor;
pub(super) use selection::CertifiedPendingReleasedDrop;
pub(in crate::physical_runtime) use selection::SelectedReleasedBlob;
pub(in crate::physical_runtime) use selection::SelectedReleasedDescriptorObservation;
pub(super) use selection::SELECTION_ROSTER_ENTRY_BYTES;
pub(super) use selection::{discover_release_basis, select};
