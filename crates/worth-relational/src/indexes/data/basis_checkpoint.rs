//! Authority-free checkpoint encoding of an owner's exact-basis selections.
use crate::history::data::coordinates::{BranchId, CommitId};
use crate::identity::data::VersionId;
use crate::indexes::data::definition::{DerivedIndexGenerationId, DerivedIndexId};
use crate::schema::data::SchemaVersionId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct IndexBasisCheckpoint {
    pub(crate) branch: BranchId,
    pub(crate) commit: CommitId,
    pub(crate) version: VersionId,
    pub(crate) schema: SchemaVersionId,
    pub(crate) empty: Vec<DerivedIndexId>,
    pub(crate) bindings: Vec<(DerivedIndexId, DerivedIndexGenerationId)>,
}
