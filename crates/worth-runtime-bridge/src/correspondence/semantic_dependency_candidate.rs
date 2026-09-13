use std::sync::Arc;

use worth_foundational::facade::{
    AspectBinding, AspectContract, AspectMask, AuthoritativeAspectChangeKind, ProjectionMask,
    TruthPartitionRole,
};

use super::{BridgeCorrespondenceDenial, BridgeCorrespondenceDenialKind};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BridgeSemanticLocality {
    SourceRecord,
    /// One exact source record supplied by a Bridge-managed temporal intent.
    /// The static conditional registration may widen across records, but each
    /// managed execution is rebound to one retained record identity.
    ManagedSourceRecord,
    SourcePartition(TruthPartitionRole),
    WholeLogicalGraph,
}

#[derive(Debug, Clone)]
pub struct BridgeSemanticDependencyCandidateParts {
    pub source_installation_identity: Arc<str>,
    pub source_basis: Arc<str>,
    pub source_runtime_authority: u64,
    pub source_installation_generation: u64,
    pub source_authority_binding_identity: Arc<str>,
    pub source_stage_identity: Option<Arc<str>>,
    pub source_node_identity: Arc<str>,
    pub dependency_ordinal: usize,
    pub declared_graph_role: Arc<str>,
    pub graph_participation_identity: Arc<str>,
    pub graph_adapter_identity: Arc<str>,
    pub source_record_identity:
        Option<crate::relational_identity::RelationalBridgeRecordIdentityParts>,
    pub observation_record_identity:
        Option<crate::relational_identity::RelationalBridgeRecordIdentityParts>,
    pub contract: AspectContract,
    pub projection_mask: AspectMask<ProjectionMask>,
    pub binding: AspectBinding,
    pub locality: BridgeSemanticLocality,
    pub relevant_changes: Vec<AuthoritativeAspectChangeKind>,
}

#[derive(Debug, Clone)]
pub struct BridgeSemanticDependencyCandidate {
    pub(crate) source_installation_identity: Arc<str>,
    pub(crate) source_basis: Arc<str>,
    pub(crate) source_runtime_authority: u64,
    pub(crate) source_installation_generation: u64,
    pub(crate) source_authority_binding_identity: Arc<str>,
    pub(crate) source_stage_identity: Option<Arc<str>>,
    pub(crate) source_node_identity: Arc<str>,
    pub(crate) dependency_ordinal: usize,
    pub(crate) declared_graph_role: Arc<str>,
    pub(crate) graph_participation_identity: Arc<str>,
    pub(crate) graph_adapter_identity: Arc<str>,
    pub(crate) source_record_identity:
        Option<crate::relational_identity::RelationalBridgeRecordIdentityParts>,
    pub(crate) observation_record_identity:
        Option<crate::relational_identity::RelationalBridgeRecordIdentityParts>,
    pub(crate) contract: AspectContract,
    pub(crate) projection_mask: AspectMask<ProjectionMask>,
    pub(crate) binding: AspectBinding,
    pub(crate) locality: BridgeSemanticLocality,
    pub(crate) relevant_changes: Vec<AuthoritativeAspectChangeKind>,
}

impl BridgeSemanticDependencyCandidate {
    /// Admits one owner-validated semantic dependency projection into Bridge.
    /// The source owner remains responsible for its own installation authority;
    /// this value grants only Bridge correspondence admission.
    pub fn admit(
        parts: BridgeSemanticDependencyCandidateParts,
    ) -> Result<Self, BridgeCorrespondenceDenial> {
        parts
            .contract
            .admits_projection_mask(&parts.projection_mask)
            .map_err(|_| {
                BridgeCorrespondenceDenial::without_admission(
                    BridgeCorrespondenceDenialKind::ProjectionMaskNotAdmitted,
                )
            })?;
        if parts.source_installation_identity.trim().is_empty()
            || parts.source_basis.trim().is_empty()
            || parts.source_runtime_authority == 0
            || parts.source_installation_generation == 0
            || parts.source_authority_binding_identity.trim().is_empty()
            || parts.source_node_identity.trim().is_empty()
            || parts.declared_graph_role.trim().is_empty()
            || parts.graph_participation_identity.trim().is_empty()
            || parts.graph_adapter_identity.trim().is_empty()
            || parts.relevant_changes.is_empty()
            || matches!(parts.locality, BridgeSemanticLocality::SourceRecord)
                && parts.source_record_identity.is_none()
            || !matches!(
                parts.locality,
                BridgeSemanticLocality::SourceRecord | BridgeSemanticLocality::SourcePartition(_)
            ) && parts.source_record_identity.is_some()
            || matches!(parts.locality, BridgeSemanticLocality::SourceRecord)
                && parts.observation_record_identity.is_some()
                && parts.observation_record_identity != parts.source_record_identity
            || matches!(parts.locality, BridgeSemanticLocality::ManagedSourceRecord)
                && (parts.source_record_identity.is_some()
                    || parts.observation_record_identity.is_some())
        {
            return Err(BridgeCorrespondenceDenial::without_admission(
                BridgeCorrespondenceDenialKind::InvalidPortableDependency,
            ));
        }
        Ok(Self {
            source_installation_identity: parts.source_installation_identity,
            source_basis: parts.source_basis,
            source_runtime_authority: parts.source_runtime_authority,
            source_installation_generation: parts.source_installation_generation,
            source_authority_binding_identity: parts.source_authority_binding_identity,
            source_stage_identity: parts.source_stage_identity,
            source_node_identity: parts.source_node_identity,
            dependency_ordinal: parts.dependency_ordinal,
            declared_graph_role: parts.declared_graph_role,
            graph_participation_identity: parts.graph_participation_identity,
            graph_adapter_identity: parts.graph_adapter_identity,
            source_record_identity: parts.source_record_identity,
            observation_record_identity: parts.observation_record_identity,
            contract: parts.contract,
            projection_mask: parts.projection_mask,
            binding: parts.binding,
            locality: parts.locality,
            relevant_changes: parts.relevant_changes,
        })
    }

    pub fn contract(&self) -> &AspectContract {
        &self.contract
    }

    pub fn declared_graph_role(&self) -> &str {
        &self.declared_graph_role
    }

    pub fn graph_participation_identity(&self) -> &str {
        &self.graph_participation_identity
    }

    pub fn graph_adapter_identity(&self) -> &str {
        &self.graph_adapter_identity
    }

    pub fn projection_mask(&self) -> &AspectMask<ProjectionMask> {
        &self.projection_mask
    }

    pub fn binding(&self) -> &AspectBinding {
        &self.binding
    }

    pub fn locality(&self) -> &BridgeSemanticLocality {
        &self.locality
    }

    pub fn relevant_changes(&self) -> &[AuthoritativeAspectChangeKind] {
        &self.relevant_changes
    }

    pub fn source_node_identity(&self) -> &str {
        &self.source_node_identity
    }

    pub fn source_stage_identity(&self) -> Option<&str> {
        self.source_stage_identity.as_deref()
    }

    pub const fn source_record_identity(
        &self,
    ) -> Option<crate::relational_identity::RelationalBridgeRecordIdentityParts> {
        self.source_record_identity
    }

    pub const fn dependency_ordinal(&self) -> usize {
        self.dependency_ordinal
    }

    pub const fn source_installation_generation(&self) -> u64 {
        self.source_installation_generation
    }

    pub const fn observation_record_identity(
        &self,
    ) -> Option<crate::relational_identity::RelationalBridgeRecordIdentityParts> {
        self.observation_record_identity
    }

    pub fn retains_same_source_authority_as(&self, other: &Self) -> bool {
        self.source_authority_binding_identity == other.source_authority_binding_identity
            && self.source_runtime_authority == other.source_runtime_authority
            && self.source_installation_generation == other.source_installation_generation
            && self.graph_participation_identity == other.graph_participation_identity
            && self.graph_adapter_identity == other.graph_adapter_identity
    }

    pub(crate) fn canonical_registration_key(&self) -> String {
        let locality = match &self.locality {
            BridgeSemanticLocality::SourceRecord => "record".to_string(),
            BridgeSemanticLocality::ManagedSourceRecord => "managed-record".to_string(),
            BridgeSemanticLocality::SourcePartition(partition) => {
                format!("partition:{}", partition.as_str())
            }
            BridgeSemanticLocality::WholeLogicalGraph => "graph".to_string(),
        };
        let mask = if self.projection_mask.is_whole_aspect() {
            "whole".to_string()
        } else {
            self.projection_mask
                .paths()
                .iter()
                .map(|path| {
                    path.fields()
                        .iter()
                        .map(|field| field.as_str())
                        .collect::<Vec<_>>()
                        .join(".")
                })
                .collect::<Vec<_>>()
                .join("|")
        };
        let changes = self
            .relevant_changes
            .iter()
            .map(|change| change.canonical_name())
            .collect::<Vec<_>>()
            .join("|");
        [
            self.source_installation_identity.to_string(),
            self.source_basis.to_string(),
            self.source_runtime_authority.to_string(),
            self.source_installation_generation.to_string(),
            self.source_authority_binding_identity.to_string(),
            self.source_stage_identity
                .as_deref()
                .unwrap_or("operation")
                .to_string(),
            self.source_node_identity.to_string(),
            self.dependency_ordinal.to_string(),
            self.declared_graph_role.to_string(),
            self.graph_participation_identity.to_string(),
            self.graph_adapter_identity.to_string(),
            source_record_identity_token(self.source_record_identity),
            source_record_identity_token(self.observation_record_identity),
            self.contract.key().as_str().to_string(),
            self.contract.identity().0.to_string(),
            self.contract.revision().0.to_string(),
            mask,
            self.binding.canonical_name(),
            locality,
            changes,
        ]
        .into_iter()
        .map(|field| format!("{}:{field}", field.len()))
        .collect()
    }

    pub(crate) fn owned_signal_partition(&self) -> worth_signal::facade::PartitionToken {
        let partition = match &self.locality {
            BridgeSemanticLocality::SourcePartition(role) => {
                self.source_record_identity.map_or_else(
                    || role.as_str().to_owned(),
                    |record| format!("{}:{}", role.as_str(), record.bridge_entity_identity()),
                )
            }
            BridgeSemanticLocality::SourceRecord => {
                format!("source-record:{}", self.binding.canonical_name())
            }
            BridgeSemanticLocality::ManagedSourceRecord => {
                format!("managed-source-record:{}", self.binding.canonical_name())
            }
            BridgeSemanticLocality::WholeLogicalGraph => "whole-logical-graph".to_owned(),
        };
        worth_signal::facade::PartitionToken::new(partition)
    }

    pub(crate) fn authority_registration_key(&self) -> String {
        [
            self.source_runtime_authority.to_string(),
            self.source_installation_generation.to_string(),
            self.source_authority_binding_identity.to_string(),
            self.graph_participation_identity.to_string(),
            self.graph_adapter_identity.to_string(),
            source_record_identity_token(self.source_record_identity),
            source_record_identity_token(self.observation_record_identity),
        ]
        .into_iter()
        .map(|field| format!("{}:{field}", field.len()))
        .collect()
    }
}

impl PartialEq for BridgeSemanticDependencyCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.source_authority_binding_identity == other.source_authority_binding_identity
            && self.source_installation_identity == other.source_installation_identity
            && self.source_basis == other.source_basis
            && self.source_runtime_authority == other.source_runtime_authority
            && self.source_installation_generation == other.source_installation_generation
            && self.source_stage_identity == other.source_stage_identity
            && self.source_node_identity == other.source_node_identity
            && self.dependency_ordinal == other.dependency_ordinal
            && self.declared_graph_role == other.declared_graph_role
            && self.graph_participation_identity == other.graph_participation_identity
            && self.graph_adapter_identity == other.graph_adapter_identity
            && self.source_record_identity == other.source_record_identity
            && self.observation_record_identity == other.observation_record_identity
            && self.contract == other.contract
            && self.projection_mask == other.projection_mask
            && self.binding == other.binding
            && self.locality == other.locality
            && self.relevant_changes == other.relevant_changes
    }
}

impl Eq for BridgeSemanticDependencyCandidate {}

fn source_record_identity_token(
    identity: Option<crate::relational_identity::RelationalBridgeRecordIdentityParts>,
) -> String {
    identity
        .map(|identity| identity.bridge_entity_identity())
        .unwrap_or_else(|| "not-record-local".to_string())
}
