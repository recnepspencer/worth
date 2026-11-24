//! Authority-free checkpoint encoding of an owner's exact-basis selections.
use crate::history::data::coordinates::{BranchId, CommitId};
use crate::identity::data::VersionId;
use crate::indexes::data::definition::{DerivedIndexGenerationId, DerivedIndexId};
use crate::schema::data::SchemaVersionId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexBasisCheckpoint {
    pub branch: BranchId,
    pub commit: CommitId,
    pub version: VersionId,
    pub schema: SchemaVersionId,
    pub empty: Vec<DerivedIndexId>,
    pub bindings: Vec<(DerivedIndexId, DerivedIndexGenerationId)>,
}
