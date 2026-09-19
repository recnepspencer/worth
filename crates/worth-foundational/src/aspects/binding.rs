use serde::{Deserialize, Serialize};

use super::FieldKey;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TruthPartitionRole(String);

impl TruthPartitionRole {
    pub fn new(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if value.is_empty() || value.trim() != value || value.chars().any(char::is_whitespace) {
            Err("invalid-truth-partition-role")
        } else {
            Ok(Self(value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AspectBinding {
    EntityField { field: FieldKey },
    RelationField { field: FieldKey },
    RelationSourceEndpoint,
    RelationTargetEndpoint,
    StructuralRegion,
    StructuralPartition,
    StructuralFacet,
    LifecycleTransition,
}

impl AspectBinding {
    pub fn canonical_name(&self) -> String {
        match self {
            Self::EntityField { field } => format!("entity-field:{}", field.as_str()),
            Self::RelationField { field } => format!("relation-field:{}", field.as_str()),
            Self::RelationSourceEndpoint => "relation-source-endpoint".to_string(),
            Self::RelationTargetEndpoint => "relation-target-endpoint".to_string(),
            Self::StructuralRegion => "structural-region".to_string(),
            Self::StructuralPartition => "structural-partition".to_string(),
            Self::StructuralFacet => "structural-facet".to_string(),
            Self::LifecycleTransition => "lifecycle-transition".to_string(),
        }
    }

    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        match self {
            Self::EntityField { field } | Self::RelationField { field } => {
                field.owned_allocation_capacity_bytes()
            }
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AuthoritativeAspectChangeKind {
    WholeAspectSet,
    WholeAspectClear,
    FieldSet,
    FieldClear,
    RelationSourceEndpoint,
    RelationTargetEndpoint,
    StructuralCreate,
    StructuralUpdate,
    StructuralDelete,
    StructuralRetainForAudit,
    StructuralMaterializationSuspended,
    StructuralRematerialized,
    LifecycleCreate,
    LifecycleDelete,
    LifecycleRetainForAudit,
    Opaque,
}

impl AuthoritativeAspectChangeKind {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::WholeAspectSet => "whole-aspect-set",
            Self::WholeAspectClear => "whole-aspect-clear",
            Self::FieldSet => "field-set",
            Self::FieldClear => "field-clear",
            Self::RelationSourceEndpoint => "relation-source-endpoint",
            Self::RelationTargetEndpoint => "relation-target-endpoint",
            Self::StructuralCreate => "structural-create",
            Self::StructuralUpdate => "structural-update",
            Self::StructuralDelete => "structural-delete",
            Self::StructuralRetainForAudit => "structural-retain-for-audit",
            Self::StructuralMaterializationSuspended => "structural-materialization-suspended",
            Self::StructuralRematerialized => "structural-rematerialized",
            Self::LifecycleCreate => "lifecycle-create",
            Self::LifecycleDelete => "lifecycle-delete",
            Self::LifecycleRetainForAudit => "lifecycle-retain-for-audit",
            Self::Opaque => "opaque",
        }
    }
}
