use serde::{Deserialize, Serialize};

use crate::identity::data::KindId;
use crate::merge::data::identity::IdentityBasisDeclaration;
use crate::merge::data::policy::AspectMergePolicyDeclaration;
use crate::schema::data::{
    AspectContractPlanRevision, RelationIntegrityPlanRevision, SchemaId, SchemaVersionId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MergeSchemaKindClass {
    Entity,
    Relation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeSchemaKindSemanticSnapshot {
    pub kind_class: MergeSchemaKindClass,
    pub kind_id: KindId,
    pub kind_name: String,
    pub schema_id: SchemaId,
    pub schema_version_id: SchemaVersionId,
    pub aspect_plan_revision: AspectContractPlanRevision,
    pub identity_declarations: Vec<IdentityBasisDeclaration>,
    pub merge_policy_declarations: Vec<AspectMergePolicyDeclaration>,
    pub relation_integrity_plan_revision: Option<RelationIntegrityPlanRevision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeSchemaSnapshotDigestBasis {
    pub authoritative_schema_id: Option<SchemaId>,
    pub authoritative_schema_version_id: Option<SchemaVersionId>,
    pub registry_digest: String,
    pub touched_kinds: std::sync::Arc<[MergeSchemaKindSemanticSnapshot]>,
}
