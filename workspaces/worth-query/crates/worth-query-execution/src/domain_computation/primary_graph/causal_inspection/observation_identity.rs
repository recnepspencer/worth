use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use worth_runtime_bridge::facade::BridgeIdentityEvidence;

use crate::domain_computation::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

macro_rules! causal_identity_type {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name(WorthQueryEvidenceIdentity);

        impl $name {
            pub(crate) fn from_identity(identity: WorthQueryEvidenceIdentity) -> Self {
                Self(identity)
            }
            pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
                &self.0
            }
            pub fn bridge_admission_evidence(&self) -> BridgeIdentityEvidence {
                self.0.bridge_evidence_identity()
            }
            pub fn as_str(&self) -> &str {
                self.0.reporting_projection()
            }
        }

        impl From<WorthQueryEvidenceIdentity> for $name {
            fn from(value: WorthQueryEvidenceIdentity) -> Self {
                Self::from_identity(value)
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> Ordering {
                self.as_str().cmp(other.as_str())
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Hash for $name {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.as_str().hash(state);
            }
        }
    };
}

macro_rules! causal_label_identity_type {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name(WorthQueryEvidenceIdentity);

        impl $name {
            pub(crate) fn from_identity(identity: WorthQueryEvidenceIdentity) -> Self {
                Self(identity)
            }

            pub fn as_str(&self) -> &str {
                self.0.reporting_projection()
            }
        }

        impl From<WorthQueryEvidenceIdentity> for $name {
            fn from(value: WorthQueryEvidenceIdentity) -> Self {
                Self::from_identity(value)
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> Ordering {
                self.as_str().cmp(other.as_str())
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Hash for $name {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.as_str().hash(state);
            }
        }
    };
}

causal_identity_type!(CausalObservationReceiptIdentity);
causal_identity_type!(CausalObservationQueryIdentity);
causal_identity_type!(CausalObservationBasisIdentity);
causal_identity_type!(CausalObservationTargetIdentity);
causal_identity_type!(CausalResultShapeContextIdentity);
causal_identity_type!(CausalQueryObservationReceiptIdentity);
causal_identity_type!(CausalObservationAnchorDigest);
causal_identity_type!(CausalObservationAnchorCountersIdentity);
causal_label_identity_type!(CausalObservationAnchorFailureIdentity);
causal_label_identity_type!(CausalEvidenceReferenceReceiptIdentity);
causal_label_identity_type!(CausalEvidenceReferenceResolutionCountersIdentity);
causal_label_identity_type!(CausalEvidenceReferenceResolutionDenialIdentity);
causal_label_identity_type!(CausalEvidenceReferenceIndexIdentity);
causal_identity_type!(CausalEvidenceReferenceIndexRecordIdentity);
causal_label_identity_type!(CausalEvidenceReferenceIndexErrorIdentity);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalEvidenceReferenceDigest {
    identity: WorthQueryEvidenceIdentity,
    bridge_authority: Option<BridgeIdentityEvidence>,
}

impl CausalEvidenceReferenceDigest {
    pub(crate) fn from_identity(identity: WorthQueryEvidenceIdentity) -> Self {
        Self {
            identity,
            bridge_authority: None,
        }
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }

    pub fn as_str(&self) -> &str {
        self.identity.as_str()
    }

    pub(in crate::domain_computation::primary_graph) fn bridge_authority_evidence(
        &self,
    ) -> BridgeIdentityEvidence {
        self.bridge_authority
            .clone()
            .unwrap_or_else(|| self.identity.bridge_evidence_identity())
    }
}

impl From<WorthQueryEvidenceIdentity> for CausalEvidenceReferenceDigest {
    fn from(value: WorthQueryEvidenceIdentity) -> Self {
        Self::from_identity(value)
    }
}

impl Ord for CausalEvidenceReferenceDigest {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for CausalEvidenceReferenceDigest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for CausalEvidenceReferenceDigest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

pub enum CausalEvidenceReferenceInput {
    Typed(CausalEvidenceReferenceDigest),
}

impl From<CausalEvidenceReferenceDigest> for CausalEvidenceReferenceInput {
    fn from(value: CausalEvidenceReferenceDigest) -> Self {
        Self::Typed(value)
    }
}

impl From<BridgeIdentityEvidence> for CausalEvidenceReferenceInput {
    fn from(value: BridgeIdentityEvidence) -> Self {
        let bridge_authority = value.clone();
        let identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalEvidenceReference)
                .field_bridge_retained_evidence_identity(
                    WorthQueryEvidenceTag::new("bridge_evidence"),
                    &value,
                )
                .seal();
        Self::Typed(CausalEvidenceReferenceDigest {
            identity,
            bridge_authority: Some(bridge_authority),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalObservationTargetHandle {
    identity: CausalObservationTargetIdentity,
}

impl CausalObservationTargetHandle {
    pub fn from_evidence_identity(identity: &WorthQueryEvidenceIdentity) -> Self {
        let identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalObservationTarget)
                .field_evidence_identity(WorthQueryEvidenceTag::new("source_identity"), identity)
                .seal()
                .into();
        Self { identity }
    }

    pub fn rendered(&self) -> &str {
        self.identity.as_str()
    }

    pub fn identity(&self) -> &CausalObservationTargetIdentity {
        &self.identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalResultShapeContextHandle {
    identity: CausalResultShapeContextIdentity,
}

impl CausalResultShapeContextHandle {
    pub fn from_evidence_identity(identity: &WorthQueryEvidenceIdentity) -> Self {
        let identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalResultShapeContext)
                .field_evidence_identity(WorthQueryEvidenceTag::new("source_identity"), identity)
                .seal()
                .into();
        Self { identity }
    }

    pub fn rendered(&self) -> &str {
        self.identity.as_str()
    }

    pub fn identity(&self) -> &CausalResultShapeContextIdentity {
        &self.identity
    }
}
