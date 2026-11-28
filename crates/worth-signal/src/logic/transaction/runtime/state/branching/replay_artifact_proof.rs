use serde::{Deserialize, Serialize};

use super::state_proof::ReplayMismatchClass;
use crate::logic::transaction::runtime::state::canonical_digest;

const PROOF_SCHEMA_VERSION: &str = "worth-signal-proof-v2";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeProofReport {
    pub proof_schema_version: String,
    pub schema_registry_digest: String,
    pub registry_bundle_digest: String,
}

pub fn runtime_proof_report(schema_registry_digest: &str) -> RuntimeProofReport {
    RuntimeProofReport {
        proof_schema_version: PROOF_SCHEMA_VERSION.to_owned(),
        schema_registry_digest: schema_registry_digest.to_owned(),
        registry_bundle_digest: canonical_digest(&(PROOF_SCHEMA_VERSION, schema_registry_digest)),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplayArtifactProofInput {
    pub proof_schema_version: String,
    pub registry_bundle_digest: Option<String>,
    pub branch_state_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayArtifactProofReport {
    pub proof_schema_version: String,
    pub expected: ReplayArtifactProofInput,
    pub replayed: ReplayArtifactProofInput,
    pub parity: bool,
    pub mismatch_classes: Vec<ReplayMismatchClass>,
}

pub fn replay_artifact_proof_report(
    expected: ReplayArtifactProofInput,
    replayed: ReplayArtifactProofInput,
) -> ReplayArtifactProofReport {
    use ReplayMismatchClass::*;

    let mut mismatch_classes = Vec::new();
    if expected.proof_schema_version != PROOF_SCHEMA_VERSION
        || replayed.proof_schema_version != PROOF_SCHEMA_VERSION
    {
        mismatch_classes.push(ProofSchemaVersionMismatch);
    }
    match (
        &expected.registry_bundle_digest,
        &replayed.registry_bundle_digest,
    ) {
        (Some(left), Some(right)) if left != right => {
            mismatch_classes.push(RegistryBundleDigestMismatch);
        }
        (None, Some(_)) | (Some(_), None) => mismatch_classes.push(MissingRegistryBundleDigest),
        _ => {}
    }
    if expected.branch_state_digest != replayed.branch_state_digest {
        mismatch_classes.push(BranchStateDigestMismatch);
    }
    ReplayArtifactProofReport {
        proof_schema_version: PROOF_SCHEMA_VERSION.to_owned(),
        expected,
        replayed,
        parity: mismatch_classes.is_empty(),
        mismatch_classes,
    }
}
