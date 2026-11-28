use serde::{Deserialize, Serialize};

use crate::recipe::model::{KeyedRecipeFamilySpec, KeyedSourceFamilySpec, RecipeSpec, SourceSpec};
use crate::runtime::compute_callbacks::CapturedHostCapabilityRead;
use crate::runtime::policy::RuntimePolicySpec;
use crate::runtime::summaries::{RuntimeSnapshotEnvelope, RuntimeStoreSnapshot};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostCapabilityTransportArtifact {
    pub family: String,
    pub registration_id: String,
    pub compatibility: String,
    pub exact_restore_outcome: String,
    pub portable_import_outcome: String,
    pub portable_import_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnavailableCallbackArtifact {
    pub id: String,
    pub signal_kind: String,
    pub reason: String,
    pub current_reads: Vec<String>,
    #[serde(default)]
    pub host_capability_reads: Vec<CapturedHostCapabilityRead>,
    #[serde(default)]
    pub host_capability_transports: Vec<HostCapabilityTransportArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDefinitionEnvelope {
    pub policy: RuntimePolicySpec,
    pub sources: Vec<SourceSpec>,
    pub recipes: Vec<RecipeSpec>,
    pub source_families: Vec<KeyedSourceFamilySpec>,
    pub recipe_families: Vec<KeyedRecipeFamilySpec>,
    #[serde(default)]
    pub worker_public_output_ids: Vec<String>,
    #[serde(default)]
    pub unavailable_callbacks: Vec<UnavailableCallbackArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(not(test), allow(dead_code))]
pub struct RuntimeEnvelope {
    pub definitions: RuntimeDefinitionEnvelope,
    pub snapshot: RuntimeSnapshotEnvelope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortableRuntimeEnvelopeArtifact {
    pub definitions: RuntimeDefinitionEnvelope,
    pub state: RuntimeStoreSnapshot,
}
