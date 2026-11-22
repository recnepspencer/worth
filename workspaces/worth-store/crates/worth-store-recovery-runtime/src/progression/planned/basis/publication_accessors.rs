use super::{
    ExpectationPayload, RecoveryPublicationAction, RecoveryPublicationCandidateArtifact,
    RecoveryPublicationExpectation, RecoveryPublicationPlan,
};
use worth_store_physical_format::RecordArtifactFile;

impl RecoveryPublicationPlan {
    pub const fn store_identity(
        &self,
    ) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        self.body().store
    }
    pub const fn checkpoint_identity(
        &self,
    ) -> Option<worth_store_physical_format::PhysicalCheckpointIdentity> {
        self.body().checkpoint
    }
    pub const fn source_generation(&self) -> u64 {
        self.body().source_generation
    }
    pub const fn staging_generation(&self) -> u64 {
        self.body().staging_generation
    }
    pub fn actions(&self) -> &[RecoveryPublicationAction] {
        &self.body().actions
    }
    pub fn expected_effects(&self) -> u64 {
        self.body().actions.len() as u64
    }
    pub const fn plan_identity(&self) -> [u8; 32] {
        self.body().plan_identity
    }
    pub const fn root_protocol(
        &self,
    ) -> worth_store::physical_runtime::RecoveryRootProtocolPublicationPlan {
        self.body().root_protocol
    }
    pub const fn current_selector(&self) -> worth_store_physical_format::DurableRootSelector {
        self.body().current_selector
    }
    pub const fn recovered_root(
        &self,
    ) -> &worth_store_physical_format::DurablePhysicalRootManifest {
        &self.body().recovered_root
    }
    pub fn candidates(&self) -> &[RecoveryPublicationCandidateArtifact] {
        &self.body().candidates
    }
    pub fn referenced_artifacts(&self) -> &[RecordArtifactFile] {
        &self.body().referenced_artifacts
    }
    pub fn created_artifacts(&self) -> &[RecordArtifactFile] {
        &self.body().created_artifacts
    }

    pub(crate) fn into_command_parts(
        self,
    ) -> (
        RecoveryPublicationExpectation,
        Box<[RecoveryPublicationCandidateArtifact]>,
    ) {
        let body = self.into_body();
        (
            body.expectation_backing.finish(ExpectationPayload {
                store: body.store,
                checkpoint: body.checkpoint,
                source_generation: body.source_generation,
                staging_generation: body.staging_generation,
                plan_identity: body.plan_identity,
                root_protocol: body.root_protocol,
                current_selector: body.current_selector,
                recovered_root: body.recovered_root,
                referenced_artifacts: body.referenced_artifacts,
                created_artifacts: body.created_artifacts,
                release_topology: body.release_topology,
            }),
            body.candidates,
        )
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
