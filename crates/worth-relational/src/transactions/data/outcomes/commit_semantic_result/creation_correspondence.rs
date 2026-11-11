//! Canonical create names retain original text and exact endpoint meaning.
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalCreatedEntityKey {
    pub partition: PartitionId,
    pub kind: KindId,
    pub client_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RelationalCreatedEndpoint {
    Existing(EntityId),
    Created(RelationalCreatedEntityKey),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalCreatedRelationKey {
    pub partition: PartitionId,
    pub kind: KindId,
    pub client_key: String,
    pub source: RelationalCreatedEndpoint,
    pub target: RelationalCreatedEndpoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalCreatedEntityCorrespondence {
    pub key: RelationalCreatedEntityKey,
    pub assigned: EntityId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalCreatedRelationCorrespondence {
    pub key: RelationalCreatedRelationKey,
    pub assigned: RelationId,
}

pub(crate) fn entity_key(
    created: &crate::transactions::data::CreatedEntityRef,
    symbols: &crate::symbols::data::StringInterner,
) -> Result<RelationalCreatedEntityKey, super::RelationalCommitSemanticResultRefusal> {
    Ok(RelationalCreatedEntityKey {
        partition: created.partition_id,
        kind: created.kind_id,
        client_key: created
            .client_key
            .resolve_with_interner(symbols)
            .ok_or(super::RelationalCommitSemanticResultRefusal::MissingClientKeyText)?
            .to_owned(),
    })
}

pub(crate) fn endpoint(
    reference: &crate::transactions::data::EntityReference,
    symbols: &crate::symbols::data::StringInterner,
) -> Result<RelationalCreatedEndpoint, super::RelationalCommitSemanticResultRefusal> {
    match reference {
        crate::transactions::data::EntityReference::Existing(id) => {
            Ok(RelationalCreatedEndpoint::Existing(*id))
        }
        crate::transactions::data::EntityReference::Created(created) => Ok(
            RelationalCreatedEndpoint::Created(entity_key(created, symbols)?),
        ),
    }
}
