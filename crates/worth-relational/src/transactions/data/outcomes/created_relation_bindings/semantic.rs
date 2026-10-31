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
                source: canonical_created_endpoint(&original.source, symbols)?,
                target: canonical_created_endpoint(&original.target, symbols)?,
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
                    source: stable_endpoint(&original.source),
                    target: stable_endpoint(&original.target),
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

fn stable_endpoint(original: &StableEntityReference) -> RelationalCreatedEndpoint {
    match original {
        StableEntityReference::Existing(id) => RelationalCreatedEndpoint::Existing(*id),
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
