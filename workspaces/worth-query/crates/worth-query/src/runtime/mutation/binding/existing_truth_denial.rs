use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
#[cfg(test)]
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::runtime::mutation::binding::existing_truth::WorthQueryExistingTruthTargetBinding;
#[cfg(test)]
use crate::runtime::WorthQueryMutationTargetCollectionIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryExistingTruthBindingDenialKind {
    UnsupportedFamily,
    ResolvedTargetMissing,
    CollectionMismatch,
}

impl WorthQueryExistingTruthBindingDenialKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedFamily => "unsupported-family",
            Self::ResolvedTargetMissing => "resolved-target-missing",
            Self::CollectionMismatch => "collection-mismatch",
        }
    }
}

impl std::fmt::Display for WorthQueryExistingTruthBindingDenialKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExistingTruthBindingDenial {
    binding: WorthQueryExistingTruthTargetBinding,
    kind: WorthQueryExistingTruthBindingDenialKind,
    message: String,
    denial_digest: String,
}

impl WorthQueryExistingTruthBindingDenial {
    #[cfg(test)]
    pub(crate) fn new(
        binding: &WorthQueryExistingTruthTargetBinding,
        kind: WorthQueryExistingTruthBindingDenialKind,
        message: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let message = message.into();
        let denial_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "existing-truth-binding-denial",
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("family"),
                    binding.family().as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("authoritative"),
                    binding.authoritative_identity().evidence_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("resolved"),
                    &binding.resolved_target_identity().evidence_identity(),
                )
                .optional_evidence_identity(
                    WorthQueryEvidenceTag::new("collection"),
                    binding
                        .target_collection_identity()
                        .map(WorthQueryMutationTargetCollectionIdentity::evidence_identity),
                )
                .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
                .field_value(WorthQueryEvidenceTag::new("message"), &message)
                .seal()?
                .as_str()
                .to_string();
        Ok(Self {
            binding: binding.clone(),
            kind,
            message,
            denial_digest,
        })
    }

    pub fn binding(&self) -> &WorthQueryExistingTruthTargetBinding {
        &self.binding
    }

    pub fn kind(&self) -> WorthQueryExistingTruthBindingDenialKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn denial_digest(&self) -> &str {
        &self.denial_digest
    }
}

impl std::fmt::Display for WorthQueryExistingTruthBindingDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "existing-truth binding `{}` denied for authoritative `{}`: {}",
            self.kind,
            self.binding.authoritative_identity().as_str(),
            self.message
        )
    }
}
