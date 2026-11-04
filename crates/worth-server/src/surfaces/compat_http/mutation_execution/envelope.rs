use crate::{
    WorthServerDirectContextArtifact, WorthServerQuerySupportPosture, WorthServerResponseEnvelope,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerCompatibilityMutationEnvelope {
    support_posture: WorthServerQuerySupportPosture,
    workspace_name: String,
    handoff_digest: String,
    direct_context: WorthServerDirectContextArtifact,
    response_envelope: WorthServerResponseEnvelope,
    canonical_digest: String,
}

impl WorthServerCompatibilityMutationEnvelope {
    pub(crate) fn new(
        support_posture: WorthServerQuerySupportPosture,
        workspace_name: String,
        handoff_digest: String,
        direct_context: WorthServerDirectContextArtifact,
        response_envelope: WorthServerResponseEnvelope,
    ) -> Self {
        let canonical_digest = format!(
            "worth-server-compat-mutation-envelope-v3|handoff:{}|response:{}",
            handoff_digest,
            response_envelope.canonical_digest(),
        );
        Self {
            support_posture,
            workspace_name,
            handoff_digest,
            direct_context,
            response_envelope,
            canonical_digest,
        }
    }

    pub fn support_posture(&self) -> &WorthServerQuerySupportPosture {
        &self.support_posture
    }

    pub fn workspace_name(&self) -> &str {
        &self.workspace_name
    }

    pub fn handoff_digest(&self) -> &str {
        &self.handoff_digest
    }

    pub fn direct_context(&self) -> &WorthServerDirectContextArtifact {
        &self.direct_context
    }

    pub fn response_envelope(&self) -> &WorthServerResponseEnvelope {
        &self.response_envelope
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}
