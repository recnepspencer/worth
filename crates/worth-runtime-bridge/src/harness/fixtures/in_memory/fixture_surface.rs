#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotFixture {
    identity: TruthSnapshotIdentity,
    read_result_identity: TruthSnapshotIdentity,
    records: Vec<SnapshotReadRecord>,
}

impl SnapshotFixture {
    pub fn new(identity: TruthSnapshotIdentity, records: Vec<SnapshotReadRecord>) -> Self {
        Self {
            read_result_identity: identity.clone(),
            identity,
            records,
        }
    }

    pub fn with_read_result_identity(mut self, identity: TruthSnapshotIdentity) -> Self {
        self.read_result_identity = identity;
        self
    }

    pub fn identity(&self) -> &TruthSnapshotIdentity {
        &self.identity
    }

    pub fn read_result_identity(&self) -> &TruthSnapshotIdentity {
        &self.read_result_identity
    }

    pub fn records(&self) -> &[SnapshotReadRecord] {
        &self.records
    }
}

#[derive(Debug, Clone)]
pub struct BridgeHarnessFixture {
    policy: BridgeRuntimePolicy,
    mappings: Vec<BridgeMappingRegistration>,
    aspect_mappings: Vec<BridgeAspectRegistration>,
    source_declarations: Vec<crate::source::SourceDeclaration>,
    structural_declarations: Vec<crate::structural::StructuralIdentityDeclaration>,
    source_adapter_capabilities: Option<crate::source::BridgeSourceCapabilitySet>,
    committed_patches: Vec<BridgeCommittedPatchEnvelope>,
    snapshots: Vec<SnapshotFixture>,
    lineage_context: Option<BridgeLineageContext>,
    continuity_authorities: Vec<(String, BridgeHistoricalLineageAuthority)>,
}

impl BridgeHarnessFixture {
    pub fn new(mappings: Vec<BridgeMappingRegistration>) -> Self {
        Self {
            policy: BridgeRuntimePolicy::development(),
            mappings,
            aspect_mappings: Vec::new(),
            source_declarations: Vec::new(),
            structural_declarations: Vec::new(),
            source_adapter_capabilities: None,
            committed_patches: Vec::new(),
            snapshots: Vec::new(),
            lineage_context: None,
            continuity_authorities: Vec::new(),
        }
    }

    pub fn with_policy(mut self, policy: BridgeRuntimePolicy) -> Self {
        self.policy = policy;
        self
    }

    pub fn with_committed_patch(mut self, patch: BridgeCommittedPatchEnvelope) -> Self {
        self.committed_patches.push(patch);
        self
    }

    pub fn with_aspect_mapping(mut self, aspect_mapping: BridgeAspectRegistration) -> Self {
        self.aspect_mappings.push(aspect_mapping);
        self
    }

    pub fn with_source_declaration(
        mut self,
        source_declaration: crate::source::SourceDeclaration,
    ) -> Self {
        self.source_declarations.push(source_declaration);
        self
    }

    pub fn with_structural_declaration(
        mut self,
        structural_declaration: crate::structural::StructuralIdentityDeclaration,
    ) -> Self {
        self.structural_declarations.push(structural_declaration);
        self
    }

    pub fn with_source_adapter_capabilities(
        mut self,
        capabilities: crate::source::BridgeSourceCapabilitySet,
    ) -> Self {
        self.source_adapter_capabilities = Some(capabilities);
        self
    }

    pub fn with_snapshot(mut self, snapshot: SnapshotFixture) -> Self {
        self.snapshots.push(snapshot);
        self
    }

    pub fn with_lineage_context(mut self, lineage_context: BridgeLineageContext) -> Self {
        self.lineage_context = Some(lineage_context);
        self
    }

    pub fn with_continuity_authority(
        mut self,
        entity_identity: impl Into<String>,
        authority: BridgeHistoricalLineageAuthority,
    ) -> Self {
        self.continuity_authorities
            .push((entity_identity.into(), authority));
        self
    }

    pub fn policy(&self) -> BridgeRuntimePolicy {
        self.policy
    }

    pub fn mappings(&self) -> &[BridgeMappingRegistration] {
        &self.mappings
    }

    pub fn committed_patches(&self) -> &[BridgeCommittedPatchEnvelope] {
        &self.committed_patches
    }

    pub fn aspect_mappings(&self) -> &[BridgeAspectRegistration] {
        &self.aspect_mappings
    }

    pub fn source_declarations(&self) -> &[crate::source::SourceDeclaration] {
        &self.source_declarations
    }

    pub fn structural_declarations(&self) -> &[crate::structural::StructuralIdentityDeclaration] {
        &self.structural_declarations
    }

    pub fn source_adapter_capabilities(&self) -> Option<&crate::source::BridgeSourceCapabilitySet> {
        self.source_adapter_capabilities.as_ref()
    }

    pub fn snapshots(&self) -> &[SnapshotFixture] {
        &self.snapshots
    }

    pub fn lineage_context(&self) -> Option<&BridgeLineageContext> {
        self.lineage_context.as_ref()
    }

    pub fn continuity_authorities(&self) -> &[(String, BridgeHistoricalLineageAuthority)] {
        &self.continuity_authorities
    }
}

use super::*;
