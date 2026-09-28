use std::sync::Arc;

use crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeSymbolicTargetReferenceFamily {
    SameBatchDeclaredTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeSymbolicTargetReferenceOutcome {
    SameBatchSymbolicTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSymbolicTargetSymbolIdentity {
    value: Arc<str>,
}

impl BridgeSymbolicTargetSymbolIdentity {
    pub fn from_external_symbol_evidence(symbol_evidence: impl AsRef<str>) -> Self {
        Self {
            value: Arc::from(format!(
                "bridge-symbolic-target-symbol:{}",
                symbol_evidence.as_ref()
            )),
        }
    }

    pub fn as_str(&self) -> &str {
        self.value.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSymbolicTargetResolvedEntityIdentity {
    value: Arc<str>,
    parts: RelationalBridgeRecordIdentityParts,
}

impl BridgeSymbolicTargetResolvedEntityIdentity {
    pub fn from_relational_record(parts: RelationalBridgeRecordIdentityParts) -> Self {
        Self {
            value: Arc::from(parts.bridge_entity_identity()),
            parts,
        }
    }

    pub fn as_str(&self) -> &str {
        self.value.as_ref()
    }

    pub fn relational_record_parts(&self) -> RelationalBridgeRecordIdentityParts {
        self.parts
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSymbolicTargetCollection {
    value: Arc<str>,
}

impl BridgeSymbolicTargetCollection {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn as_str(&self) -> &str {
        self.value.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSymbolicTargetReferenceBundle {
    family: BridgeSymbolicTargetReferenceFamily,
    outcome: BridgeSymbolicTargetReferenceOutcome,
    symbol: BridgeSymbolicTargetSymbolIdentity,
    resolved_entity_identity: BridgeSymbolicTargetResolvedEntityIdentity,
    target_collection: Option<BridgeSymbolicTargetCollection>,
}

impl BridgeSymbolicTargetReferenceBundle {
    pub fn same_batch_target(
        symbol: BridgeSymbolicTargetSymbolIdentity,
        resolved_entity_identity: BridgeSymbolicTargetResolvedEntityIdentity,
        target_collection: Option<BridgeSymbolicTargetCollection>,
    ) -> Self {
        Self {
            family: BridgeSymbolicTargetReferenceFamily::SameBatchDeclaredTarget,
            outcome: BridgeSymbolicTargetReferenceOutcome::SameBatchSymbolicTarget,
            symbol,
            resolved_entity_identity,
            target_collection,
        }
    }

    pub fn family(&self) -> BridgeSymbolicTargetReferenceFamily {
        self.family
    }

    pub fn outcome(&self) -> BridgeSymbolicTargetReferenceOutcome {
        self.outcome
    }

    pub fn symbol(&self) -> &str {
        self.symbol.as_str()
    }

    pub fn symbol_handle(&self) -> &BridgeSymbolicTargetSymbolIdentity {
        &self.symbol
    }

    pub fn resolved_entity_identity(&self) -> &str {
        self.resolved_entity_identity.as_str()
    }

    pub fn resolved_entity_identity_handle(&self) -> &BridgeSymbolicTargetResolvedEntityIdentity {
        &self.resolved_entity_identity
    }

    pub fn target_collection(&self) -> Option<&str> {
        self.target_collection
            .as_ref()
            .map(BridgeSymbolicTargetCollection::as_str)
    }

    pub fn target_collection_handle(&self) -> Option<&BridgeSymbolicTargetCollection> {
        self.target_collection.as_ref()
    }
}
