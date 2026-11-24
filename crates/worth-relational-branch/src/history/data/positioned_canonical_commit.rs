use worth_relational_contracts::internal_port::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativePatchEnvelopeOwnerPort as _;
use std::sync::Arc;

use serde::Serialize;

use super::CanonicalCommitEnvelope;
use crate::publication::patch::data::{PatchStreamPosition, PublishedAuthoritativePatchEnvelope};
use crate::runtime::state::subsystems::history::canonical_publication_routes::position_admission::CanonicalPositionAdmission;

/// One performed canonical commit together with its globally unique runtime
/// stream position. Both axes are sealed after construction: recovery and
/// migration must finish admission before this current artifact can exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PositionedCanonicalCommit {
    position: PatchStreamPosition,
    canonical: Arc<CanonicalCommitEnvelope>,
}

impl PositionedCanonicalCommit {
    pub fn admit(admission: CanonicalPositionAdmission) -> Self {
        let (position, canonical) = admission.into_parts();
        Self {
            position,
            canonical,
        }
    }

    pub(crate) fn for_test(
        position: PatchStreamPosition,
        canonical: Arc<CanonicalCommitEnvelope>,
    ) -> Self {
        Self {
            position,
            canonical,
        }
    }

    pub const fn position(&self) -> PatchStreamPosition {
        self.position
    }

    pub fn envelope(&self) -> &CanonicalCommitEnvelope {
        self.canonical.as_ref()
    }

    pub fn canonical_arc(&self) -> &Arc<CanonicalCommitEnvelope> {
        &self.canonical
    }

    pub fn published_patch(&self) -> PublishedAuthoritativePatchEnvelope {
        PublishedAuthoritativePatchEnvelope::from_canonical(self.position, &self.canonical.patch)
    }

    pub(crate) fn positioned_envelope_mut_for_test(
        &mut self,
        copy: impl FnOnce(&CanonicalCommitEnvelope) -> CanonicalCommitEnvelope,
    ) -> &mut super::CanonicalCommitEnvelopeBody {
        // Preserve unique actual ownership when available. Aliased originals
        // need an explicitly admitted complete data copy, never deep Clone.
        if Arc::get_mut(&mut self.canonical).is_none() {
            self.canonical = Arc::new(copy(&self.canonical));
        }
        Arc::get_mut(&mut self.canonical)
            .expect("the mutable test copy is unique")
            .body_mut_for_test()
    }
}

impl std::ops::Deref for PositionedCanonicalCommit {
    type Target = CanonicalCommitEnvelope;

    fn deref(&self) -> &Self::Target {
        self.envelope()
    }
}

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait PositionedCanonicalCommitFixturePort: Sized {
    fn positioned_commit_for_test(
        position: PatchStreamPosition,
        canonical: Arc<CanonicalCommitEnvelope>,
    ) -> Self;
    fn positioned_envelope_mut_for_test(
        &mut self,
        copy: impl FnOnce(&CanonicalCommitEnvelope) -> CanonicalCommitEnvelope,
    ) -> &mut super::CanonicalCommitEnvelopeBody;
}

impl PositionedCanonicalCommitFixturePort for PositionedCanonicalCommit {
    fn positioned_commit_for_test(
        position: PatchStreamPosition,
        canonical: Arc<CanonicalCommitEnvelope>,
    ) -> Self {
        PositionedCanonicalCommit::for_test(position, canonical)
    }

    fn positioned_envelope_mut_for_test(
        &mut self,
        copy: impl FnOnce(&CanonicalCommitEnvelope) -> CanonicalCommitEnvelope,
    ) -> &mut super::CanonicalCommitEnvelopeBody {
        self.positioned_envelope_mut_for_test(copy)
    }
}
