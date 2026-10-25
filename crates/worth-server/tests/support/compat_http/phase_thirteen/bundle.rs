use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorthServerPhaseThirteenBundle {
    digests: BTreeMap<&'static str, String>,
}

impl WorthServerPhaseThirteenBundle {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_digest(mut self, label: &'static str, digest: impl Into<String>) -> Self {
        self.digests.insert(label, digest.into());
        self
    }

    pub(crate) fn with_optional_digest(
        mut self,
        label: &'static str,
        digest: Option<impl Into<String>>,
    ) -> Self {
        if let Some(digest) = digest {
            self.digests.insert(label, digest.into());
        }
        self
    }

    pub(crate) fn digest(&self, label: &'static str) -> Option<&str> {
        self.digests.get(label).map(String::as_str)
    }
}

pub(crate) const DECLARATION_DIGEST: &str = "declaration_digest";
pub(crate) const RESPONSE_DIGEST: &str = "response_digest";
pub(crate) const BASIS_DIGEST: &str = "basis_digest";
pub(crate) const SUPPORT_POSTURE_DIGEST: &str = "support_posture_digest";
pub(crate) const PROVENANCE_DIGEST: &str = "provenance_digest";
pub(crate) const FAILURE_DIGEST: &str = "failure_digest";
pub(crate) const AUDIT_EVIDENCE_DIGEST: &str = "audit_evidence_digest";
pub(crate) const POLICY_DIGEST: &str = "policy_digest";
pub(crate) const CACHEABILITY_DIGEST: &str = "cacheability_digest";
pub(crate) const METADATA_IDENTITY_DIGEST: &str = "metadata_identity_digest";
pub(crate) const FILE_ENVELOPE_DIGEST: &str = "file_envelope_digest";
pub(crate) const MUTATION_RESULT_DIGEST: &str = "mutation_result_digest";
pub(crate) const REQUEST_CONTRACT_DIGEST: &str = "request_contract_digest";
