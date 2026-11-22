//! Stable posture-event identity and predecessor causal link (R8.23).

use worth_foundational::facade::CanonicalDigestId;

/// Exact identity of one external-effect causality event.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ExternalEffectPostureIdentity {
    digest: CanonicalDigestId,
}

impl ExternalEffectPostureIdentity {
    pub const fn from_digest(
        _authority: &super::causal_event::CausalConstructionAuthority,
        digest: CanonicalDigestId,
    ) -> Self {
        Self { digest }
    }

    pub const fn digest(&self) -> &CanonicalDigestId {
        &self.digest
    }

    pub fn bytes(&self) -> &[u8; 32] {
        self.digest.bytes()
    }
}

/// Causal link naming the exact predecessor posture identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ExternalEffectCausalLink {
    predecessor: ExternalEffectPostureIdentity,
}

impl ExternalEffectCausalLink {
    pub const fn to(predecessor: &ExternalEffectPostureIdentity) -> Self {
        Self {
            predecessor: *predecessor,
        }
    }

    pub const fn predecessor(&self) -> &ExternalEffectPostureIdentity {
        &self.predecessor
    }
}
