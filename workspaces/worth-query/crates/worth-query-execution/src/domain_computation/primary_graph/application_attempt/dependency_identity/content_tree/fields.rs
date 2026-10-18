//! Cold normalization lookup, shared only while refreshed addresses agree.
use super::super::super::WorthQueryApplicationObservedFact as Fact;
use super::*;
use std::collections::BTreeMap;
use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::identity::EntityId;

type FieldOrdinals = BTreeMap<EntityId, BTreeMap<AspectFieldLocator, Vec<usize>>>;
pub(super) struct Fields {
    ordinals: FieldOrdinals,
    bytes: u64,
    _memory: Option<ExecutionMemoryReservation>,
}

fn address(fact: &Fact) -> Option<(EntityId, &AspectFieldLocator)> {
    match fact {
        Fact::Field {
            entity_id, locator, ..
        }
        | Fact::AbsentField {
            entity_id, locator, ..
        } => Some((*entity_id, locator)),
        _ => None,
    }
}

impl Fields {
    pub(super) fn build(
        body: &FactPayload,
        control: StorageControl<'_, '_>,
    ) -> Result<Arc<Self>, StoreDenial> {
        let bytes = body
            .retained_resident_bytes()
            .and_then(|bytes| bytes.checked_mul(8))
            .and_then(|bytes| bytes.checked_add(u64::try_from(body.len()).ok()?.checked_mul(512)?))
            .and_then(|bytes| {
                bytes.checked_add((size_of::<Self>() + 2 * size_of::<usize>()) as u64)
            })
            .ok_or(StoreDenial::Representability)?;
        let memory = control.policy().reserve_memory(bytes);
        control.check_live()?;
        let memory = memory?;
        let mut ordinals: FieldOrdinals = BTreeMap::new();
        for (ordinal, fact) in body.iter().enumerate() {
            control.check_live()?;
            super::super::super::super::fact_verification_observation::content_field_index();
            if let Some((entity, locator)) = address(fact) {
                ordinals
                    .entry(entity)
                    .or_default()
                    .entry(locator.clone())
                    .or_default()
                    .push(ordinal);
            }
        }
        Ok(Arc::new(Self {
            ordinals,
            bytes,
            _memory: memory,
        }))
    }
    pub(super) fn ordinals(&self, entity: EntityId, locator: &AspectFieldLocator) -> &[usize] {
        self.ordinals
            .get(&entity)
            .and_then(|fields| fields.get(locator))
            .map_or(&[], Vec::as_slice)
    }
    pub(super) fn addresses_match(old: Option<&Fact>, new: Option<&Fact>) -> bool {
        old.and_then(address) == new.and_then(address)
    }
    pub(super) fn bytes(&self) -> u64 {
        self.bytes
    }
}
