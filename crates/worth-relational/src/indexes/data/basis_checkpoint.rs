//! Authority-free checkpoint encoding of an owner's exact-basis selections.
use super::{DerivedIndexGenerationId, DerivedIndexId};
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::VersionId;
use crate::schema::data::SchemaVersionId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct IndexBasisCheckpoint {
    pub(crate) branch: BranchId,
    pub(crate) commit: CommitId,
    pub(crate) version: VersionId,
    pub(crate) schema: SchemaVersionId,
    pub(crate) bindings: Vec<(DerivedIndexId, DerivedIndexGenerationId)>,
}
