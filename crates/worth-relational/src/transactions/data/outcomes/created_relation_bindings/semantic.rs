use super::{CommitCreatedRelationBindings, StableEntityReference};
use crate::symbols::data::StringInterner;
use crate::transactions::data::outcomes::commit_semantic_result::{
    canonical_created_endpoint, RelationalCommitSemanticResultRefusal as Refusal,
    RelationalCreatedEndpoint, RelationalCreatedEntityKey, RelationalCreatedRelationCorrespondence,
    RelationalCreatedRelationKey,
};
use std::collections::BTreeMap;

impl CommitCreatedRelationBindings {
    pub(crate) fn canonical_meaning(
        &self,
        symbols: &StringInterner,
        created_entities: &BTreeMap<crate::identity::data::EntityId, &RelationalCreatedEntityKey>,
    ) -> Result<Vec<RelationalCreatedRelationCorrespondence>, Refusal> {
        let mut canonical = BTreeMap::new();
        for (original, assigned) in &self.by_exact_reference {
            let key = RelationalCreatedRelationKey {
                partition: original.partition_id,
                kind: original.kind_id,
                client_key: original
                    .client_key
                    .resolve_with_interner(symbols)
                    .ok_or(Refusal::MissingClientKeyText)?
                    .to_owned(),
                source: endpoint(&original.source, symbols, created_entities)?,
                target: endpoint(&original.target, symbols, created_entities)?,
            };
            insert(&mut canonical, key, *assigned)?;
        }
        for (original, assigned) in &self.by_stable_reference {
            insert(
                &mut canonical,
                RelationalCreatedRelationKey {
                    partition: original.partition_id,
                    kind: original.kind_id,
                    client_key: original.client_key.clone(),
                    source: stable_endpoint(&original.source, created_entities),
                    target: stable_endpoint(&original.target, created_entities),
                },
                *assigned,
            )?;
        }
        Ok(canonical
            .into_iter()
            .map(|(key, assigned)| RelationalCreatedRelationCorrespondence { key, assigned })
            .collect())
    }
}

// A suffix applies through actual prefix IDs, while the enclosing original
// still names their Create coordinates. Normalize only IDs in this commit's
// actual allocation roster; existing committed endpoints retain Existing.
fn endpoint(
    reference: &crate::transactions::data::EntityReference,
    symbols: &StringInterner,
    created: &BTreeMap<crate::identity::data::EntityId, &RelationalCreatedEntityKey>,
) -> Result<RelationalCreatedEndpoint, Refusal> {
    if let crate::transactions::data::EntityReference::Existing(id) = reference
        && let Some(key) = created.get(id)
    {
        return Ok(RelationalCreatedEndpoint::Created((*key).clone()));
    }
    canonical_created_endpoint(reference, symbols)
}
fn stable_endpoint(
    original: &StableEntityReference,
    created: &BTreeMap<crate::identity::data::EntityId, &RelationalCreatedEntityKey>,
) -> RelationalCreatedEndpoint {
    match original {
        StableEntityReference::Existing(id) => created.get(id).map_or_else(
            || RelationalCreatedEndpoint::Existing(*id),
            |key| RelationalCreatedEndpoint::Created((*key).clone()),
        ),
        StableEntityReference::Created {
            partition_id,
            kind_id,
            client_key,
        } => RelationalCreatedEndpoint::Created(RelationalCreatedEntityKey {
            partition: *partition_id,
            kind: *kind_id,
            client_key: client_key.clone(),
        }),
    }
}

fn insert(
    map: &mut BTreeMap<RelationalCreatedRelationKey, crate::identity::data::RelationId>,
    key: RelationalCreatedRelationKey,
    assigned: crate::identity::data::RelationId,
) -> Result<(), Refusal> {
    if let Some(previous) = map.insert(key, assigned)
        && previous != assigned
    {
        return Err(Refusal::ConflictingCreateCorrespondence);
    }
    Ok(())
}
