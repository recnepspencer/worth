use super::CommitCreatedEntityBindings;
use crate::symbols::data::StringInterner;
use crate::transactions::data::outcomes::commit_semantic_result::{
    canonical_created_entity_key, RelationalCommitSemanticResultRefusal as Refusal,
    RelationalCreatedEntityCorrespondence, RelationalCreatedEntityKey,
};
use std::collections::BTreeMap;

impl CommitCreatedEntityBindings {
    pub(crate) fn canonical_meaning(
        &self,
        symbols: &StringInterner,
    ) -> Result<Vec<RelationalCreatedEntityCorrespondence>, Refusal> {
        let mut canonical = BTreeMap::new();
        for (original, assigned) in &self.by_exact_reference {
            let key = canonical_created_entity_key(original, symbols)?;
            insert(&mut canonical, key, *assigned)?;
        }
        for (stable, assigned) in &self.by_stable_reference {
            insert(
                &mut canonical,
                RelationalCreatedEntityKey {
                    partition: stable.partition_id,
                    kind: stable.kind_id,
                    client_key: stable.client_key.clone(),
                },
                *assigned,
            )?;
        }
        Ok(canonical
            .into_iter()
            .map(|(key, assigned)| RelationalCreatedEntityCorrespondence { key, assigned })
            .collect())
    }
}

fn insert(
    map: &mut BTreeMap<RelationalCreatedEntityKey, crate::identity::data::EntityId>,
    key: RelationalCreatedEntityKey,
    assigned: crate::identity::data::EntityId,
) -> Result<(), Refusal> {
    if let Some(previous) = map.insert(key, assigned)
        && previous != assigned
    {
        return Err(Refusal::ConflictingCreateCorrespondence);
    }
    Ok(())
}
