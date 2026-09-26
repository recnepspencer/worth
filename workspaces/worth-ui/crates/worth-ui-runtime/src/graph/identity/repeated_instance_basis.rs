use std::sync::Arc;

use worth_foundational::facade::{FoundationalExternalIdentityToken, FoundationalIdentityKind};

use crate::declaration::{stable_text_digest, UiDeclarationIdentityDigest};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiRepeatedInstanceBasisKind {
    DeclarationKeyed,
    RuntimeDataKeyed,
    Denied,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiRepeatedInstanceBasisDenial {
    MissingBasis,
    BasisFreeRuntimeIdentityDenied,
    PositionBasedBasis,
    ContradictoryBasis,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiRuntimeDataInstanceKey {
    value: String,
}

pub struct UiRuntimeDataInstanceKeyKind;

impl FoundationalIdentityKind for UiRuntimeDataInstanceKeyKind {}

pub type UiRuntimeDataInstanceKeyToken =
    FoundationalExternalIdentityToken<Arc<str>, UiRuntimeDataInstanceKeyKind>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiRepeatedInstanceBasis {
    DeclarationKeyed {
        declaration_identity_digest: UiDeclarationIdentityDigest,
    },
    RuntimeDataKeyed {
        runtime_data_key: UiRuntimeDataInstanceKey,
    },
    Denied {
        denial: UiRepeatedInstanceBasisDenial,
    },
    Unavailable,
}

impl UiRuntimeDataInstanceKey {
    pub(crate) fn admit(
        token: UiRuntimeDataInstanceKeyToken,
    ) -> Result<Self, UiRepeatedInstanceBasisDenial> {
        let value = token.into_value().to_string();
        if value.trim().is_empty() {
            return Err(UiRepeatedInstanceBasisDenial::MissingBasis);
        }
        if value.contains("position:") {
            return Err(UiRepeatedInstanceBasisDenial::PositionBasedBasis);
        }
        Ok(Self { value })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl UiRepeatedInstanceBasis {
    pub(crate) const fn declaration_keyed(
        declaration_identity_digest: UiDeclarationIdentityDigest,
    ) -> Self {
        Self::DeclarationKeyed {
            declaration_identity_digest,
        }
    }

    pub const fn denied(denial: UiRepeatedInstanceBasisDenial) -> Self {
        Self::Denied { denial }
    }

    pub const fn unavailable() -> Self {
        Self::Unavailable
    }

    pub(crate) fn runtime_data_keyed(
        runtime_data_key: UiRuntimeDataInstanceKey,
    ) -> Result<Self, UiRepeatedInstanceBasisDenial> {
        Ok(Self::RuntimeDataKeyed { runtime_data_key })
    }

    pub fn kind(&self) -> UiRepeatedInstanceBasisKind {
        match self {
            Self::DeclarationKeyed { .. } => UiRepeatedInstanceBasisKind::DeclarationKeyed,
            Self::RuntimeDataKeyed { .. } => UiRepeatedInstanceBasisKind::RuntimeDataKeyed,
            Self::Denied { .. } => UiRepeatedInstanceBasisKind::Denied,
            Self::Unavailable => UiRepeatedInstanceBasisKind::Unavailable,
        }
    }

    pub fn denial(&self) -> Option<&UiRepeatedInstanceBasisDenial> {
        match self {
            Self::Denied { denial } => Some(denial),
            Self::DeclarationKeyed { .. } | Self::RuntimeDataKeyed { .. } | Self::Unavailable => {
                None
            }
        }
    }

    pub(crate) fn identity_digest(&self) -> u64 {
        match self {
            Self::DeclarationKeyed {
                declaration_identity_digest,
            } => {
                stable_text_digest("graph-basis:declaration")
                    ^ declaration_identity_digest.raw().rotate_left(17)
            }
            Self::RuntimeDataKeyed { runtime_data_key } => {
                stable_text_digest("graph-basis:runtime")
                    ^ stable_text_digest(runtime_data_key.as_str()).rotate_left(31)
            }
            Self::Denied { denial } => {
                stable_text_digest("graph-basis:denied")
                    ^ stable_text_digest(match denial {
                        UiRepeatedInstanceBasisDenial::MissingBasis => "missing-basis",
                        UiRepeatedInstanceBasisDenial::BasisFreeRuntimeIdentityDenied => {
                            "basis-free-runtime-identity-denied"
                        }
                        UiRepeatedInstanceBasisDenial::PositionBasedBasis => "position-based-basis",
                        UiRepeatedInstanceBasisDenial::ContradictoryBasis => "contradictory-basis",
                    })
                    .rotate_left(37)
            }
            Self::Unavailable => stable_text_digest("graph-basis:unavailable"),
        }
    }
}
