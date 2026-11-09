use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQueryWorkspaceError;
use crate::runtime::{
    WorthQueryMutationSymbolIdentity, WorthQueryMutationTargetCollectionIdentity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQuerySymbolicTargetReferenceFamily {
    SameBatchDeclaredTarget,
}

impl WorthQuerySymbolicTargetReferenceFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SameBatchDeclaredTarget => "same-batch-declared-target",
        }
    }
}

impl std::fmt::Display for WorthQuerySymbolicTargetReferenceFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySymbolicTargetReference {
    family: WorthQuerySymbolicTargetReferenceFamily,
    symbol: WorthQueryMutationSymbolIdentity,
    target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
}

impl WorthQuerySymbolicTargetReference {
    pub fn new(symbol: impl Into<String>) -> Result<Self, WorthQueryWorkspaceError> {
        let symbol = symbol.into();
        if symbol.trim().is_empty() {
            return Err(WorthQueryWorkspaceError::new(
                "symbolic target reference may not be empty",
            ));
        }
        Ok(Self {
            family: WorthQuerySymbolicTargetReferenceFamily::SameBatchDeclaredTarget,
            symbol: WorthQueryMutationSymbolIdentity::new("symbolic-target-reference", symbol)?,
            target_collection: None,
        })
    }

    pub fn in_target_collection(
        mut self,
        collection: impl Into<String>,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        let collection = collection.into();
        if collection.trim().is_empty() {
            return Err(WorthQueryWorkspaceError::new(
                "symbolic target collection may not be empty",
            ));
        }
        self.target_collection = Some(WorthQueryMutationTargetCollectionIdentity::new(
            "symbolic-target-reference",
            collection,
        )?);
        Ok(self)
    }

    pub fn family(&self) -> WorthQuerySymbolicTargetReferenceFamily {
        self.family
    }

    pub fn symbol(&self) -> &str {
        self.symbol.as_str()
    }

    pub fn symbol_identity(&self) -> &WorthQueryMutationSymbolIdentity {
        &self.symbol
    }

    pub fn target_collection(&self) -> Option<&str> {
        self.target_collection
            .as_ref()
            .map(WorthQueryMutationTargetCollectionIdentity::as_str)
    }

    pub fn target_collection_identity(
        &self,
    ) -> Option<&WorthQueryMutationTargetCollectionIdentity> {
        self.target_collection.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQuerySymbolicTargetReferenceDenialKind {
    RequiresBatchContext,
    UnresolvedSameBatchTarget,
    CollectionMismatch,
    NonEntityReferenceTarget,
}

impl WorthQuerySymbolicTargetReferenceDenialKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RequiresBatchContext => "requires-batch-context",
            Self::UnresolvedSameBatchTarget => "unresolved-same-batch-target",
            Self::CollectionMismatch => "collection-mismatch",
            Self::NonEntityReferenceTarget => "non-entity-reference-target",
        }
    }
}

impl std::fmt::Display for WorthQuerySymbolicTargetReferenceDenialKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySymbolicTargetReferenceDenial {
    reference: WorthQuerySymbolicTargetReference,
    kind: WorthQuerySymbolicTargetReferenceDenialKind,
    message: String,
    denial_digest: Result<String, IdentityDerivationDenial>,
}

impl WorthQuerySymbolicTargetReferenceDenial {
    pub(crate) fn new(
        reference: &WorthQuerySymbolicTargetReference,
        kind: WorthQuerySymbolicTargetReferenceDenialKind,
        message: impl Into<String>,
    ) -> Self {
        let message = message.into();
        let denial_digest = (|| {
            Ok(worth_query_evidence_identity(
                WorthQueryEvidenceScope::MutationEvidenceAggregateDigest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("role"),
                "symbolic-target-reference-denial",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                reference.family().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("symbol"),
                reference.symbol_identity().evidence_identity(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("collection"),
                reference
                    .target_collection_identity()
                    .map(WorthQueryMutationTargetCollectionIdentity::evidence_identity),
            )
            .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
            .field_value(WorthQueryEvidenceTag::new("message"), &message)
            .seal()?
            .as_str()
            .to_string())
        })();
        Self {
            reference: reference.clone(),
            kind,
            message,
            denial_digest,
        }
    }

    pub fn reference(&self) -> &WorthQuerySymbolicTargetReference {
        &self.reference
    }

    pub fn kind(&self) -> WorthQuerySymbolicTargetReferenceDenialKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn denial_digest(&self) -> Result<&str, IdentityDerivationDenial> {
        Ok(self.denial_digest.as_ref().map_err(|cause| *cause)?)
    }
}

impl std::fmt::Display for WorthQuerySymbolicTargetReferenceDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "symbolic target reference `{}` denied for symbol `{}`: {}",
            self.kind,
            self.reference.symbol(),
            self.message
        )
    }
}
