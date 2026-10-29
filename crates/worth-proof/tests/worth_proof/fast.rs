//! Fast integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "fast/blob_release_proof_contract.rs"]
pub(crate) mod blob_release_proof_contract;

#[path = "fast/core_artifact_and_proof_substrate_certification.rs"]
pub(crate) mod core_artifact_and_proof_substrate_certification;

#[path = "fast/dx_entrypoint_certification.rs"]
pub(crate) mod dx_entrypoint_certification;

#[path = "fast/supported_public_workflows.rs"]
pub(crate) mod supported_public_workflows;

#[path = "fast/upfront_contract_vocabulary_certification.rs"]
pub(crate) mod upfront_contract_vocabulary_certification;
