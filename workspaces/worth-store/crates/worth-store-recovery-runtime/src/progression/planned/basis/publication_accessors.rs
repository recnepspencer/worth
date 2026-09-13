use super::{
    RecoveryPublicationAction, RecoveryPublicationCandidateArtifact,
    RecoveryPublicationExpectation, RecoveryPublicationPlan,
};
use worth_store_physical_format::RecordArtifactFile;

impl RecoveryPublicationPlan {
    pub const fn store_identity(
        &self,
    ) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        self.store
    }
    pub const fn checkpoint_identity(
        &self,
    ) -> worth_store_physical_format::PhysicalCheckpointIdentity {
        self.checkpoint
    }
    pub const fn source_generation(&self) -> u64 {
        self.source_generation
    }
    pub const fn staging_generation(&self) -> u64 {
        self.staging_generation
    }
    pub fn actions(&self) -> &[RecoveryPublicationAction] {
        &self.actions
    }
    pub fn expected_effects(&self) -> u64 {
        self.actions.len() as u64
    }
    pub const fn plan_identity(&self) -> [u8; 32] {
        self.plan_identity
    }
    pub const fn root_protocol(
        &self,
    ) -> worth_store::physical_runtime::RecoveryRootProtocolPublicationPlan {
        self.root_protocol
    }
    pub const fn current_selector(&self) -> worth_store_physical_format::DurableRootSelector {
        self.current_selector
    }
    pub const fn recovered_root(
        &self,
    ) -> &worth_store_physical_format::DurablePhysicalRootManifest {
        &self.recovered_root
    }
    pub fn candidates(&self) -> &[RecoveryPublicationCandidateArtifact] {
        &self.candidates
    }
    pub fn referenced_artifacts(&self) -> &[RecordArtifactFile] {
        &self.referenced_artifacts
    }
    pub fn created_artifacts(&self) -> &[RecordArtifactFile] {
        &self.created_artifacts
    }

    pub(crate) fn into_command_parts(
        self,
    ) -> (
        RecoveryPublicationExpectation,
        Box<[RecoveryPublicationCandidateArtifact]>,
    ) {
        (
            RecoveryPublicationExpectation {
                store: self.store,
                checkpoint: self.checkpoint,
                source_generation: self.source_generation,
                staging_generation: self.staging_generation,
                plan_identity: self.plan_identity,
                root_protocol: self.root_protocol,
                current_selector: self.current_selector,
                recovered_root: self.recovered_root,
                referenced_artifacts: self.referenced_artifacts,
                created_artifacts: self.created_artifacts,
            },
            self.candidates,
        )
    }
}

impl RecoveryPublicationExpectation {
    pub const fn store_identity(
        &self,
    ) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        self.store
    }
    pub const fn checkpoint_identity(
        &self,
    ) -> worth_store_physical_format::PhysicalCheckpointIdentity {
        self.checkpoint
    }
    pub const fn source_generation(&self) -> u64 {
        self.source_generation
    }
    pub const fn staging_generation(&self) -> u64 {
        self.staging_generation
    }
    pub const fn plan_identity(&self) -> [u8; 32] {
        self.plan_identity
    }
    pub const fn root_protocol(
        &self,
    ) -> worth_store::physical_runtime::RecoveryRootProtocolPublicationPlan {
        self.root_protocol
    }
    pub const fn current_selector(&self) -> worth_store_physical_format::DurableRootSelector {
        self.current_selector
    }
    pub const fn recovered_root(
        &self,
    ) -> &worth_store_physical_format::DurablePhysicalRootManifest {
        &self.recovered_root
    }
    pub fn created_artifacts(&self) -> &[RecordArtifactFile] {
        &self.created_artifacts
    }
    pub fn referenced_artifacts(&self) -> &[RecordArtifactFile] {
        &self.referenced_artifacts
    }
}

impl RecoveryPublicationCandidateArtifact {
    pub const fn artifact(&self) -> RecordArtifactFile {
        self.artifact
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub const fn byte_count(&self) -> u64 {
        self.bytes.len() as u64
    }
    pub const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
    pub(crate) fn into_command_parts(self) -> (RecordArtifactFile, Box<[u8]>, [u8; 32]) {
        (self.artifact, self.bytes, self.payload_digest)
    }
}
