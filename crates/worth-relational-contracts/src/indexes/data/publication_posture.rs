use crate::history::data::coordinates::BranchId;
use crate::identity::data::VersionId;
use crate::schema::data::SchemaVersionId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivedIndexPublicationStatus {
    Published,
    BuildFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedIndexApplicability {
    pub branch_id: BranchId,
    pub version_id: VersionId,
    pub schema_version: SchemaVersionId,
}
