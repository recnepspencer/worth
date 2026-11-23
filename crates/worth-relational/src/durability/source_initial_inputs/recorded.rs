//! Original producer attribution is DATA, distinct from operational schema authority.
use serde::{Deserialize, Serialize};

use crate::history::data::canonical_commit_envelope::recorded_inputs::RecordedCanonicalInputs;

use crate::branch::{RelationalBranchLifecyclePosture, RelationalBranchVersion};
use worth_foundational::FoundationalBranchReferenceGeneration;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecordedEmptySchemaObservation {
    pub(crate) generation: FoundationalBranchReferenceGeneration,
    pub(crate) truth_version: RelationalBranchVersion,
    pub(crate) lifecycle: RelationalBranchLifecyclePosture,
    pub(crate) schema_carrier: [u8; 32],
}

pub(crate) fn capture_canonical_inputs(
    original: &crate::history::OriginalCanonicalInputs,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<RecordedCanonicalInputs, crate::durability::data::RelationalRecoveryStop> {
    let prestate = original
        .prestate_schema
        .recorded_image(request)?
        .carrier_digest();
    let result = original.result_schema.recorded_image(request)?;
    Ok(RecordedCanonicalInputs {
        prestate_schema: prestate,
        result_schema: result.carrier_digest(),
        mutation: original.mutation.clone(),
        result_image: (prestate != result.carrier_digest()).then_some(result),
    })
}
