//! Pure external-effect posture value type.

use super::identity::{ExternalEffectCausalLink, ExternalEffectPostureIdentity};
use super::posture::ExternalEffectPostureKind;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalEffectPosture {
    kind: ExternalEffectPostureKind,
    identity: ExternalEffectPostureIdentity,
    predecessor: Option<ExternalEffectCausalLink>,
}

impl ExternalEffectPosture {
    pub fn initial(kind: ExternalEffectPostureKind, identity: ExternalEffectPostureIdentity) -> Self {
        Self {
            kind,
            identity,
            predecessor: None,
        }
    }

    pub fn successor(
        kind: ExternalEffectPostureKind,
        identity: ExternalEffectPostureIdentity,
        predecessor: &Self,
    ) -> Self {
        Self {
            kind,
            identity,
            predecessor: Some(ExternalEffectCausalLink::to(predecessor.identity())),
        }
    }

    pub const fn kind(&self) -> ExternalEffectPostureKind {
        self.kind
    }

    pub const fn identity(&self) -> &ExternalEffectPostureIdentity {
        &self.identity
    }

    pub const fn predecessor(&self) -> Option<&ExternalEffectCausalLink> {
        self.predecessor.as_ref()
    }
}

pub struct CausalConstructionAuthority { pub(crate) _private: () }

pub const CAUSAL_CONSTRUCTION: CausalConstructionAuthority =
    CausalConstructionAuthority { _private: () };
