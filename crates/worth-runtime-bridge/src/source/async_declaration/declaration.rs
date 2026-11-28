use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::identity::{
    AsyncSourceDeclarationIdentityTag, BridgeIdentity, SourceDeclarationIdentityTag,
};
use worth_signal::facade::{AsyncNodeCapabilityDeclaration, ResourceNodeDeclaration};

use super::digest_basis::{request_response_basis, subscription_backed_basis};
use super::family::BridgeAsyncSourceDeclarationFamilyKind;
use super::rejection::BridgeAsyncSourceDeclarationRejection;
use super::BridgeAsyncSourceDeclarationCounters;

pub type BridgeAsyncSourceDeclarationIdentity = BridgeIdentity<AsyncSourceDeclarationIdentityTag>;
pub type BridgeAsyncSourceLegacyDeclarationIdentity = BridgeIdentity<SourceDeclarationIdentityTag>;

impl BridgeAsyncSourceDeclarationIdentity {
    pub fn from_stable_name(value: impl Into<Arc<str>>) -> Self {
        Self::admit_bridge_owned(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAsyncSourceDeclarationDraft {
    declaration_identity: BridgeAsyncSourceDeclarationIdentity,
    legacy_declaration_identity: BridgeAsyncSourceLegacyDeclarationIdentity,
    body: BridgeAsyncSourceDeclarationBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedBridgeAsyncSourceDeclaration {
    declaration_identity: BridgeAsyncSourceDeclarationIdentity,
    legacy_declaration_identity: BridgeAsyncSourceLegacyDeclarationIdentity,
    family_kind: BridgeAsyncSourceDeclarationFamilyKind,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
    counters: BridgeAsyncSourceDeclarationCounters,
    body: BridgeAsyncSourceDeclarationBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum BridgeAsyncSourceDeclarationBody {
    RequestResponse {
        declaration: ResourceNodeDeclaration,
    },
    SubscriptionBacked {
        declaration: AsyncNodeCapabilityDeclaration,
    },
}

impl BridgeAsyncSourceDeclarationDraft {
    pub fn request_response(
        declaration_identity: BridgeAsyncSourceDeclarationIdentity,
        legacy_declaration_identity: BridgeAsyncSourceLegacyDeclarationIdentity,
        declaration: ResourceNodeDeclaration,
    ) -> Self {
        Self {
            declaration_identity,
            legacy_declaration_identity,
            body: BridgeAsyncSourceDeclarationBody::RequestResponse { declaration },
        }
    }

    pub fn subscription_backed(
        declaration_identity: BridgeAsyncSourceDeclarationIdentity,
        legacy_declaration_identity: BridgeAsyncSourceLegacyDeclarationIdentity,
        declaration: AsyncNodeCapabilityDeclaration,
    ) -> Self {
        Self {
            declaration_identity,
            legacy_declaration_identity,
            body: BridgeAsyncSourceDeclarationBody::SubscriptionBacked { declaration },
        }
    }

    pub fn declaration_identity(&self) -> &BridgeAsyncSourceDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn legacy_declaration_identity(&self) -> &BridgeAsyncSourceLegacyDeclarationIdentity {
        &self.legacy_declaration_identity
    }
}

impl ValidatedBridgeAsyncSourceDeclaration {
    pub fn validate(
        draft: BridgeAsyncSourceDeclarationDraft,
    ) -> Result<Self, BridgeAsyncSourceDeclarationRejection> {
        let family_kind = draft.family_kind();
        let canonical_basis = match &draft.body {
            BridgeAsyncSourceDeclarationBody::RequestResponse { declaration } => {
                request_response_basis(draft.declaration_identity.as_str(), declaration)
            }
            BridgeAsyncSourceDeclarationBody::SubscriptionBacked { declaration } => {
                subscription_backed_basis(draft.declaration_identity.as_str(), declaration)
            }
        };
        let counters = match &draft.body {
            BridgeAsyncSourceDeclarationBody::RequestResponse { .. } => {
                BridgeAsyncSourceDeclarationCounters::request_response_validated()
            }
            BridgeAsyncSourceDeclarationBody::SubscriptionBacked { .. } => {
                BridgeAsyncSourceDeclarationCounters::subscription_backed_validated()
            }
        };
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Ok(Self {
            declaration_identity: draft.declaration_identity,
            legacy_declaration_identity: draft.legacy_declaration_identity,
            family_kind,
            canonical_basis: Arc::from(canonical_basis),
            digest: Arc::from(format!("bridge-async-source:sha256:{digest:x}")),
            counters,
            body: draft.body,
        })
    }

    pub fn declaration_identity(&self) -> &BridgeAsyncSourceDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn legacy_declaration_identity(&self) -> &BridgeAsyncSourceLegacyDeclarationIdentity {
        &self.legacy_declaration_identity
    }

    pub fn family_kind(&self) -> BridgeAsyncSourceDeclarationFamilyKind {
        self.family_kind
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }

    pub fn counters(&self) -> &BridgeAsyncSourceDeclarationCounters {
        &self.counters
    }

    pub fn request_response_declaration(&self) -> Option<&ResourceNodeDeclaration> {
        match &self.body {
            BridgeAsyncSourceDeclarationBody::RequestResponse { declaration } => Some(declaration),
            BridgeAsyncSourceDeclarationBody::SubscriptionBacked { .. } => None,
        }
    }

    pub fn subscription_backed_declaration(&self) -> Option<&AsyncNodeCapabilityDeclaration> {
        match &self.body {
            BridgeAsyncSourceDeclarationBody::RequestResponse { .. } => None,
            BridgeAsyncSourceDeclarationBody::SubscriptionBacked { declaration } => {
                Some(declaration)
            }
        }
    }

    pub(super) fn body(&self) -> &BridgeAsyncSourceDeclarationBody {
        &self.body
    }
}

impl BridgeAsyncSourceDeclarationDraft {
    fn family_kind(&self) -> BridgeAsyncSourceDeclarationFamilyKind {
        match &self.body {
            BridgeAsyncSourceDeclarationBody::RequestResponse { .. } => {
                BridgeAsyncSourceDeclarationFamilyKind::RequestResponse
            }
            BridgeAsyncSourceDeclarationBody::SubscriptionBacked { .. } => {
                BridgeAsyncSourceDeclarationFamilyKind::SubscriptionBacked
            }
        }
    }
}
