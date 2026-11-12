//! Original producer attribution is DATA, distinct from operational schema authority.
use serde::{Deserialize, Serialize};

use crate::branch::{RelationalBranchLifecyclePosture, RelationalBranchVersion};
use crate::durability::data::DurableBranchRootSchemaImage;
use worth_foundational::FoundationalBranchReferenceGeneration;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecordedEmptySchemaObservation {
    pub(crate) generation: FoundationalBranchReferenceGeneration,
    pub(crate) truth_version: RelationalBranchVersion,
    pub(crate) lifecycle: RelationalBranchLifecyclePosture,
    pub(crate) schema_carrier: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecordedCanonicalInputs {
    /// First in v8's canonical map: Complete admission reuses its SHA prefix.
    /// Only a schema-changing publication introduces a carrier; ordinary writes
    /// emit the fixed-size references and original policy below.
    pub(crate) result_image: Option<DurableBranchRootSchemaImage>,
    pub(crate) prestate_schema: [u8; 32],
    pub(crate) result_schema: [u8; 32],
    pub(crate) mutation: crate::config::data::MutationConfig,
}

impl RecordedCanonicalInputs {
    pub(crate) fn capture(
        original: &crate::history::OriginalCanonicalInputs,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::durability::data::RelationalRecoveryStop> {
        let prestate = original.prestate_schema.recorded_image(request)?;
        let result = original.result_schema.recorded_image(request)?;
        Ok(Self {
            prestate_schema: prestate.carrier_digest(),
            result_schema: result.carrier_digest(),
            mutation: original.mutation.clone(),
            result_image: (prestate.carrier_digest() != result.carrier_digest()).then_some(result),
        })
    }
}
