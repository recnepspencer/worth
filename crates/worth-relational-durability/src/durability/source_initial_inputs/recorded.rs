//! Original producer attribution is DATA, distinct from operational schema authority.
use serde::{Deserialize, Serialize};

use worth_foundational::FoundationalBranchReferenceGeneration;
use worth_relational_branch::internal_port::branch::lifecycle::RelationalBranchLifecyclePosture;
use worth_relational_contracts::internal_port::branch::version::RelationalBranchVersion;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecordedEmptySchemaObservation {
    pub(crate) generation: FoundationalBranchReferenceGeneration,
    pub(crate) truth_version: RelationalBranchVersion,
    pub(crate) lifecycle: RelationalBranchLifecyclePosture,
    pub(crate) schema_carrier: [u8; 32],
}
