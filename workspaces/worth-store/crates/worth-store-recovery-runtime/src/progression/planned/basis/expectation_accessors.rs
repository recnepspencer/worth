use super::RecoveryPublicationExpectation;
use worth_store_physical_format::RecordArtifactFile;

impl RecoveryPublicationExpectation {
    pub(crate) fn take_release_topology(&mut self) -> Option<super::RecoveryReleaseTopologyProof> {
        self.backing.take_release_topology()
    }
    pub const fn store_identity(
        &self,
    ) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        self.payload().store
    }
    pub const fn checkpoint_identity(
        &self,
    ) -> Option<worth_store_physical_format::PhysicalCheckpointIdentity> {
        self.payload().checkpoint
    }
    pub const fn source_generation(&self) -> u64 {
        self.payload().source_generation
    }
    pub const fn staging_generation(&self) -> u64 {
        self.payload().staging_generation
    }
    pub const fn plan_identity(&self) -> [u8; 32] {
        self.payload().plan_identity
    }
    pub const fn root_protocol(
        &self,
    ) -> worth_store::physical_runtime::RecoveryRootProtocolPublicationPlan {
        self.payload().root_protocol
    }
    pub const fn current_selector(&self) -> worth_store_physical_format::DurableRootSelector {
        self.payload().current_selector
    }
    pub const fn recovered_root(
        &self,
    ) -> &worth_store_physical_format::DurablePhysicalRootManifest {
        &self.payload().recovered_root
    }
    pub fn created_artifacts(&self) -> &[RecordArtifactFile] {
        &self.payload().created_artifacts
    }
    pub fn referenced_artifacts(&self) -> &[RecordArtifactFile] {
        &self.payload().referenced_artifacts
    }
}
