//! Original producer attribution is DATA, distinct from operational schema authority.
use serde::{Deserialize, Serialize};

use crate::branch::{
    lifecycle::RelationalBranchLifecyclePosture, version::RelationalBranchVersion,
};
use worth_foundational::FoundationalBranchReferenceGeneration;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecordedEmptySchemaObservation {
    pub(crate) generation: FoundationalBranchReferenceGeneration,
    pub(crate) truth_version: RelationalBranchVersion,
    pub(crate) lifecycle: RelationalBranchLifecyclePosture,
    pub(crate) schema_carrier: [u8; 32],
}
