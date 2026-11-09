use std::collections::BTreeSet;

use crate::symbols::data::{ClientKey, ClientKeySymbolPolicy, StringInterner};

use super::{CreateIntent, EntityMutationIntent, MutationIntent, RelationMutationIntent};
use crate::transactions::data::EntityReference;

impl MutationIntent {
    pub(crate) fn collect_raw_client_keys(
        &self,
        raw_values: &mut BTreeSet<String>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), worth_execution::ExecutionAllocationDenial> {
        allocation_policy.check_live()?;
        match self {
            Self::Create(CreateIntent::Entity(spec)) => {
                if let Some(raw) = spec.client_key.as_raw_str() {
                    raw_values.insert(raw.to_string());
                }
            }
            Self::Create(CreateIntent::EntityAspects(spec)) => {
                if let Some(raw) = spec.client_key.as_raw_str() {
                    raw_values.insert(raw.to_string());
                }
            }
            Self::Create(CreateIntent::BulkEntities(spec)) => {
                collect_bulk_raw_client_keys(&spec.client_keys, raw_values, allocation_policy)?;
            }
            Self::Create(CreateIntent::BulkRelations(spec)) => {
                collect_bulk_raw_client_keys(&spec.client_keys, raw_values, allocation_policy)?;
                for (source, target) in &spec.endpoints {
                    allocation_policy.check_live()?;
                    collect_entity_reference_raw_client_key(source, raw_values);
                    collect_entity_reference_raw_client_key(target, raw_values);
                }
            }
            Self::Create(CreateIntent::Relation(spec)) => {
                if let Some(raw) = spec.client_key.as_raw_str() {
                    raw_values.insert(raw.to_string());
                }
                collect_entity_reference_raw_client_key(&spec.source, raw_values);
                collect_entity_reference_raw_client_key(&spec.target, raw_values);
            }
            Self::Create(CreateIntent::RelationAspects(spec)) => {
                if let Some(raw) = spec.client_key.as_raw_str() {
                    raw_values.insert(raw.to_string());
                }
                collect_entity_reference_raw_client_key(&spec.source, raw_values);
                collect_entity_reference_raw_client_key(&spec.target, raw_values);
            }
            Self::Entity(EntityMutationIntent::Replace(spec)) => {
                if let Some(raw) = spec.replacement.client_key.as_raw_str() {
                    raw_values.insert(raw.to_string());
                }
            }
            Self::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
                collect_entity_reference_raw_client_key(&spec.source, raw_values);
                collect_entity_reference_raw_client_key(&spec.target, raw_values);
            }
            Self::Entity(_)
            | Self::Materialization(_)
            | Self::Relation(RelationMutationIntent::ApplyAspectPatch(_))
            | Self::Relation(RelationMutationIntent::Delete(_))
            | Self::Relation(RelationMutationIntent::Revalidate(_)) => {}
        };
        Ok(())
    }

    pub(crate) fn normalize_client_keys(
        &mut self,
        interner: &mut StringInterner,
        policy: ClientKeySymbolPolicy,
    ) -> Result<(), crate::symbols::data::RelationalSymbolExhausted> {
        self.visit_client_keys_mut(|key| {
            *key = normalize_client_key(interner, policy, key.clone())?;
            Ok(())
        })
    }
}

fn collect_bulk_raw_client_keys(
    client_keys: &[ClientKey],
    raw_values: &mut BTreeSet<String>,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<(), worth_execution::ExecutionAllocationDenial> {
    for client_key in client_keys {
        allocation_policy.check_live()?;
        if let Some(raw) = client_key.as_raw_str() {
            raw_values.insert(raw.to_string());
        }
    }
    Ok(())
}

fn collect_entity_reference_raw_client_key(
    entity_reference: &EntityReference,
    raw_values: &mut BTreeSet<String>,
) {
    if let EntityReference::Created(created) = entity_reference
        && let Some(raw) = created.client_key.as_raw_str()
    {
        raw_values.insert(raw.to_string());
    }
}

fn normalize_client_key(
    interner: &mut StringInterner,
    policy: ClientKeySymbolPolicy,
    value: ClientKey,
) -> Result<ClientKey, crate::symbols::data::RelationalSymbolExhausted> {
    value.normalize_with(interner, policy)
}
