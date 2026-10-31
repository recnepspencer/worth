use std::sync::Arc;

use serde::Serialize;

use super::CanonicalCommitEnvelope;
use crate::publication::patch::data::{PatchStreamPosition, PublishedAuthoritativePatchEnvelope};

/// One performed canonical commit together with its globally unique runtime
/// stream position. Both axes are sealed after construction: recovery and
/// migration must finish admission before this current artifact can exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct PositionedCanonicalCommit {
    position: PatchStreamPosition,
    canonical: Arc<CanonicalCommitEnvelope>,
}

impl PositionedCanonicalCommit {
    pub(crate) fn admit(admission: crate::runtime::CanonicalPositionAdmission) -> Self {
        let (position, canonical) = admission.into_parts();
        Self {
            position,
            canonical,
        }
    }

    #[cfg(test)]
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

    pub(crate) fn canonical_arc(&self) -> &Arc<CanonicalCommitEnvelope> {
        &self.canonical
    }

    pub fn published_patch(&self) -> PublishedAuthoritativePatchEnvelope {
        PublishedAuthoritativePatchEnvelope::from_canonical(self.position, &self.canonical.patch)
    }

    #[cfg(test)]
    pub(crate) fn envelope_mut_for_test(&mut self) -> &mut super::CanonicalCommitEnvelopeBody {
        Arc::make_mut(&mut self.canonical).body_mut_for_test()
    }
}

impl std::ops::Deref for PositionedCanonicalCommit {
    type Target = CanonicalCommitEnvelope;

    fn deref(&self) -> &Self::Target {
        self.envelope()
    }
}
