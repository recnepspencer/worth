//! The artifact retained by the workflow readmission fixture.
use super::*;

pub(in crate::domain_computation::managed_run::tests) struct ReadmissionArtifact;

impl WorthQueryArtifactProviderResource for ReadmissionArtifact {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.affinity.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"workflow-readmission-artifact".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        9
    }

    fn dispose(&mut self) {}
}
