use std::sync::Arc;

use worth_foundational::facade::TruthPartitionRole;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAuthoritativeSourceProfile {
    runtime_instance_id: u64,
    adapter_semantic_identity: Arc<str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeAuthoritativeSourceProfileError {
    MissingRuntimeAuthority,
    InvalidAdapterSemanticIdentity,
}

impl BridgeAuthoritativeSourceProfile {
    pub fn new(
        runtime_instance_id: u64,
        adapter_semantic_identity: impl Into<Arc<str>>,
    ) -> Result<Self, BridgeAuthoritativeSourceProfileError> {
        if runtime_instance_id == 0 {
            return Err(BridgeAuthoritativeSourceProfileError::MissingRuntimeAuthority);
        }
        let adapter_semantic_identity = adapter_semantic_identity.into();
        if adapter_semantic_identity.trim().is_empty()
            || adapter_semantic_identity.trim() != adapter_semantic_identity.as_ref()
        {
            return Err(BridgeAuthoritativeSourceProfileError::InvalidAdapterSemanticIdentity);
        }
        Ok(Self {
            runtime_instance_id,
            adapter_semantic_identity,
        })
    }

    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub fn adapter_semantic_identity(&self) -> &str {
        &self.adapter_semantic_identity
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAuthoritativeSourceProvenance {
    runtime_instance_id: u64,
    graph_role: Arc<str>,
    adapter_semantic_identity: Arc<str>,
    source_basis: Arc<str>,
    partition_role: Option<TruthPartitionRole>,
}

impl BridgeAuthoritativeSourceProvenance {
    pub fn from_owner_publication(
        runtime_instance_id: u64,
        graph_role: impl Into<Arc<str>>,
        adapter_semantic_identity: impl Into<Arc<str>>,
        source_basis: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            runtime_instance_id,
            graph_role: graph_role.into(),
            adapter_semantic_identity: adapter_semantic_identity.into(),
            source_basis: source_basis.into(),
            partition_role: None,
        }
    }

    pub fn from_owner_partition_publication(
        runtime_instance_id: u64,
        graph_role: impl Into<Arc<str>>,
        adapter_semantic_identity: impl Into<Arc<str>>,
        source_basis: impl Into<Arc<str>>,
        partition_role: TruthPartitionRole,
    ) -> Self {
        Self {
            runtime_instance_id,
            graph_role: graph_role.into(),
            adapter_semantic_identity: adapter_semantic_identity.into(),
            source_basis: source_basis.into(),
            partition_role: Some(partition_role),
        }
    }

    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub fn graph_role(&self) -> &str {
        &self.graph_role
    }

    pub fn adapter_semantic_identity(&self) -> &str {
        &self.adapter_semantic_identity
    }

    pub fn source_basis(&self) -> &str {
        &self.source_basis
    }

    pub fn partition_role(&self) -> Option<&TruthPartitionRole> {
        self.partition_role.as_ref()
    }

    pub fn matches_profile(&self, profile: &BridgeAuthoritativeSourceProfile) -> bool {
        self.runtime_instance_id == profile.runtime_instance_id
            && self.adapter_semantic_identity == profile.adapter_semantic_identity
    }

    pub(crate) fn canonical_basis(&self) -> String {
        [
            self.runtime_instance_id.to_string(),
            self.graph_role.to_string(),
            self.adapter_semantic_identity.to_string(),
            self.source_basis.to_string(),
            self.partition_role
                .as_ref()
                .map_or_else(|| "none".to_string(), |role| role.as_str().to_string()),
        ]
        .into_iter()
        .map(|field| format!("{}:{field}", field.len()))
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{BridgeAuthoritativeSourceProfile, BridgeAuthoritativeSourceProvenance};

    #[test]
    fn provenance_rejects_runtime_substitution_even_when_adapter_semantics_match() {
        let profile = BridgeAuthoritativeSourceProfile::new(42, "relational-adapter-v1").unwrap();
        let replacement_profile =
            BridgeAuthoritativeSourceProfile::new(43, "relational-adapter-v1").unwrap();
        let exact = BridgeAuthoritativeSourceProvenance::from_owner_publication(
            42,
            "model",
            "relational-adapter-v1",
            "commit=7",
        );
        let substituted = BridgeAuthoritativeSourceProvenance::from_owner_publication(
            43,
            "model",
            "relational-adapter-v1",
            "commit=7",
        );

        assert_eq!(
            profile.adapter_semantic_identity(),
            replacement_profile.adapter_semantic_identity()
        );
        assert_ne!(profile, replacement_profile);
        assert!(exact.matches_profile(&profile));
        assert!(!substituted.matches_profile(&profile));
        assert_ne!(exact.canonical_basis(), substituted.canonical_basis());
    }
}
