//! Persisted schema commitments and recorded mutation policy for one canonical commit.
use serde::{Deserialize, Serialize};

use super::schema_carrier::RecordedBranchSchemaImage;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecordedCanonicalInputs {
    /// First in v8's canonical map: Complete admission reuses its SHA prefix.
    /// Only a schema-changing publication introduces a carrier; ordinary writes
    /// emit the fixed-size references and original policy below.
    pub(crate) result_image: Option<RecordedBranchSchemaImage>,
    pub(crate) prestate_schema: [u8; 32],
    pub(crate) result_schema: [u8; 32],
    pub(crate) mutation: crate::config::data::MutationConfig,
}
