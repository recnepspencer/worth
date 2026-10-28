use serde::{Deserialize, Serialize};

use crate::errors::data::{ErrorContext, ErrorOperation, RelationalSubsystem, SuggestedFix};
use crate::identity::data::KindId;
use worth_foundational::facade::AspectKey;

use super::ContractId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaRegistryErrorClass {
    UnknownEntityKind(KindId),
    UnknownRelationKind(KindId),
    DuplicateEntityKind(KindId),
    DuplicateRelationKind(KindId),
    EntityRelationKindCollision(KindId),
    DuplicateAspectKey {
        kind_id: KindId,
        aspect_key: AspectKey,
    },
    DuplicateRelationContractId {
        kind_id: KindId,
        contract_id: ContractId,
    },
    InvalidAspectDeclaration {
        kind_id: KindId,
        detail: String,
    },
    InvalidRelationIntegrityDeclaration {
        kind_id: KindId,
        detail: String,
    },
    InconsistentSchemaBasis {
        detail: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaRegistryError {
    pub class: SchemaRegistryErrorClass,
    pub detail: String,
    pub context: ErrorContext,
}

impl SchemaRegistryError {
    fn new(class: SchemaRegistryErrorClass) -> Self {
        let detail = match &class {
            SchemaRegistryErrorClass::UnknownEntityKind(kind_id) => {
                format!("unknown entity kind {:?}", kind_id)
            }
            SchemaRegistryErrorClass::UnknownRelationKind(kind_id) => {
                format!("unknown relation kind {:?}", kind_id)
            }
            SchemaRegistryErrorClass::DuplicateEntityKind(kind_id) => {
                format!("entity kind {:?} is already registered", kind_id)
            }
            SchemaRegistryErrorClass::DuplicateRelationKind(kind_id) => {
                format!("relation kind {:?} is already registered", kind_id)
            }
            SchemaRegistryErrorClass::EntityRelationKindCollision(kind_id) => {
                format!(
                    "kind {:?} collides across entity and relation registries",
                    kind_id
                )
            }

            SchemaRegistryErrorClass::DuplicateAspectKey {
                kind_id,
                aspect_key,
            } => format!(
                "kind {:?} declares duplicate aspect key {:?}",
                kind_id, aspect_key
            ),
            SchemaRegistryErrorClass::DuplicateRelationContractId {
                kind_id,
                contract_id,
            } => format!(
                "kind {:?} declares duplicate relation contract id '{}'",
                kind_id, contract_id
            ),
            SchemaRegistryErrorClass::InvalidAspectDeclaration { kind_id, detail } => {
                format!(
                    "kind {:?} has invalid aspect declaration: {detail}",
                    kind_id
                )
            }
            SchemaRegistryErrorClass::InvalidRelationIntegrityDeclaration { kind_id, detail } => {
                format!(
                    "kind {:?} has invalid relation integrity declaration: {detail}",
                    kind_id
                )
            }
            SchemaRegistryErrorClass::InconsistentSchemaBasis { detail } => {
                format!("schema registry has inconsistent authoritative basis: {detail}")
            }
        };
        Self {
            class,
            detail,
            context: ErrorContext::new(RelationalSubsystem::Schema, ErrorOperation::ResolveSchema)
                .with_fix(SuggestedFix::ValidateSchemaRegistration),
        }
    }

    pub fn unknown_entity_kind(kind_id: KindId) -> Self {
        Self::new(SchemaRegistryErrorClass::UnknownEntityKind(kind_id))
    }

    pub fn unknown_relation_kind(kind_id: KindId) -> Self {
        Self::new(SchemaRegistryErrorClass::UnknownRelationKind(kind_id))
    }

    pub fn duplicate_entity_kind(kind_id: KindId) -> Self {
        Self::new(SchemaRegistryErrorClass::DuplicateEntityKind(kind_id))
    }

    pub fn duplicate_relation_kind(kind_id: KindId) -> Self {
        Self::new(SchemaRegistryErrorClass::DuplicateRelationKind(kind_id))
    }

    pub fn entity_relation_kind_collision(kind_id: KindId) -> Self {
        Self::new(SchemaRegistryErrorClass::EntityRelationKindCollision(
            kind_id,
        ))
    }

    pub fn duplicate_aspect_key(kind_id: KindId, aspect_key: AspectKey) -> Self {
        Self::new(SchemaRegistryErrorClass::DuplicateAspectKey {
            kind_id,
            aspect_key,
        })
    }

    pub fn invalid_aspect_declaration(kind_id: KindId, detail: impl Into<String>) -> Self {
        Self::new(SchemaRegistryErrorClass::InvalidAspectDeclaration {
            kind_id,
            detail: detail.into(),
        })
    }

    pub fn duplicate_relation_contract_id(
        kind_id: KindId,
        contract_id: impl Into<ContractId>,
    ) -> Self {
        Self::new(SchemaRegistryErrorClass::DuplicateRelationContractId {
            kind_id,
            contract_id: contract_id.into(),
        })
    }

    pub fn invalid_relation_integrity_declaration(
        kind_id: KindId,
        detail: impl Into<String>,
    ) -> Self {
        Self::new(
            SchemaRegistryErrorClass::InvalidRelationIntegrityDeclaration {
                kind_id,
                detail: detail.into(),
            },
        )
    }

    pub fn inconsistent_schema_basis(detail: impl Into<String>) -> Self {
        Self::new(SchemaRegistryErrorClass::InconsistentSchemaBasis {
            detail: detail.into(),
        })
    }
}
