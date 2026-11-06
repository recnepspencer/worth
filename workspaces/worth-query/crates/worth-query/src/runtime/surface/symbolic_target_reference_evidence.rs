use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::runtime::{
    WorthQueryMutationSymbolIdentity, WorthQueryMutationTargetCollectionIdentity,
};
use worth_runtime_bridge::facade::{
    BridgeSymbolicTargetReferenceBundle, BridgeSymbolicTargetReferenceFamily,
    BridgeSymbolicTargetReferenceOutcome,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQuerySymbolicTargetReferenceOutcome {
    SameBatchSymbolicTarget,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySymbolicTargetReferenceEvidence {
    family: crate::runtime::WorthQuerySymbolicTargetReferenceFamily,
    outcome: WorthQuerySymbolicTargetReferenceOutcome,
    symbol: WorthQueryMutationSymbolIdentity,
    resolved_entity_identity: WorthQueryEntityIdentity,
    target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
}

impl WorthQuerySymbolicTargetReferenceEvidence {
    pub(in crate::runtime) fn from_bridge_with_query_context(
        reference: &BridgeSymbolicTargetReferenceBundle,
        _resolved_entity_identity: Option<&WorthQueryEntityIdentity>,
        target_collection: Option<&str>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            family: match reference.family() {
                BridgeSymbolicTargetReferenceFamily::SameBatchDeclaredTarget => {
                    crate::runtime::WorthQuerySymbolicTargetReferenceFamily::SameBatchDeclaredTarget
                }
            },
            outcome: match reference.outcome() {
                BridgeSymbolicTargetReferenceOutcome::SameBatchSymbolicTarget => {
                    WorthQuerySymbolicTargetReferenceOutcome::SameBatchSymbolicTarget
                }
            },
            symbol: WorthQueryMutationSymbolIdentity::from_bridge_symbolic_target(
                "symbolic-target-reference",
                reference.symbol_handle(),
            )?,
            resolved_entity_identity: WorthQueryEntityIdentity::from_relational_record(
                reference
                    .resolved_entity_identity_handle()
                    .relational_record_parts(),
            ),
            target_collection: reference.target_collection().or(target_collection).map(
                |collection| {
                    WorthQueryMutationTargetCollectionIdentity::new("symbolic-target", collection)
                },
            )?,
        })
    }

    pub(in crate::runtime) fn from_reference(
        reference: &crate::runtime::WorthQuerySymbolicTargetReference,
        resolved_entity_identity: &WorthQueryEntityIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            family: reference.family(),
            outcome: WorthQuerySymbolicTargetReferenceOutcome::SameBatchSymbolicTarget,
            symbol: WorthQueryMutationSymbolIdentity::new(
                "symbolic-target-reference",
                reference.symbol(),
            )?,
            resolved_entity_identity: resolved_entity_identity.clone(),
            target_collection: reference.target_collection_identity().cloned(),
        })
    }

    pub fn family(&self) -> crate::runtime::WorthQuerySymbolicTargetReferenceFamily {
        self.family
    }

    pub fn outcome(&self) -> WorthQuerySymbolicTargetReferenceOutcome {
        self.outcome
    }

    pub fn symbol(&self) -> &WorthQueryMutationSymbolIdentity {
        &self.symbol
    }

    pub fn resolved_entity_identity(&self) -> &WorthQueryEntityIdentity {
        &self.resolved_entity_identity
    }

    pub fn target_collection(&self) -> Option<&WorthQueryMutationTargetCollectionIdentity> {
        self.target_collection.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn test_only(
        symbol: impl Into<String>,
        resolved_entity_identity: WorthQueryEntityIdentity,
        target_collection: Option<&str>,
    ) -> Self {
        Self {
            family:
                crate::runtime::WorthQuerySymbolicTargetReferenceFamily::SameBatchDeclaredTarget,
            outcome: WorthQuerySymbolicTargetReferenceOutcome::SameBatchSymbolicTarget,
            symbol: WorthQueryMutationSymbolIdentity::new("symbolic-target-reference", symbol),
            resolved_entity_identity,
            target_collection: target_collection.map(|collection| {
                WorthQueryMutationTargetCollectionIdentity::new("symbolic-target", collection)
            }),
        }
    }
}
