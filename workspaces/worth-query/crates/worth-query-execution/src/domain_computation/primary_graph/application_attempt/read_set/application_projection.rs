//! Descriptive ordinal routing established by completion, never read authority.
use super::super::{
    retained_decision_facts::{StorageControl, StoreDenial},
    WorthQueryApplicationObservedFact as Fact,
};
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::ExecutionMemoryReservation;

#[derive(Debug)]
pub(in crate::domain_computation) struct CompletedFactProjection {
    locators: BTreeMap<Arc<str>, usize>,
    by_ordinal: Vec<Arc<str>>,
    count: usize,
    physical: bool,
    bytes: u64,
    _memory: Option<ExecutionMemoryReservation>,
    scopes: super::CompletedReadScopes,
    preimages: BTreeMap<
        worth_relational::facade::identity::EntityId,
        BTreeMap<String, BTreeMap<String, BTreeMap<String, Vec<usize>>>>,
    >,
}
impl PartialEq for CompletedFactProjection {
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count && self.locators == other.locators
    }
}
impl Eq for CompletedFactProjection {}

impl CompletedFactProjection {
    pub(in crate::domain_computation::primary_graph) fn build(
        body: &FactPayload,
        scopes: super::CompletedReadScopes,
        control: StorageControl<'_, '_>,
    ) -> Result<Arc<Self>, StoreDenial> {
        let count = scopes.len();
        if count > body.len() {
            return Err(StoreDenial::ConflictingBody);
        }
        // Locator text is derived from the admitted body. Fund keys and worst
        // case B-tree node slack before constructing any owned locator.
        let mut bytes = (count as u64)
            .checked_mul(768)
            .and_then(|n| {
                n.checked_add(
                    (std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>()) as u64,
                )
            })
            .ok_or(StoreDenial::Representability)?;
        for ordinal in 0..count {
            control.check_live()?;
            if scopes.preimage_demand().is_some() {
                bytes = bytes
                    .checked_add(4096)
                    .ok_or(StoreDenial::Representability)?;
            }
            super::super::super::fact_verification_observation::provider_application_fact();
            bytes = bytes
                .checked_add(
                    body.get(ordinal)
                        .ok_or(StoreDenial::ConflictingBody)?
                        .dependency_locator_byte_bound()
                        .and_then(|n| n.checked_mul(4))
                        .ok_or(StoreDenial::Representability)?,
                )
                .ok_or(StoreDenial::Representability)?;
            if let Some(worth_query_installation::facade::WorthQueryOperationGraphReadScope::NativeProjection(scope)) = scopes.get(ordinal) {
                bytes = bytes.checked_add(scope.entity().semantic_key().len() as u64)
                    .and_then(|n| n.checked_add(scope.aspect().as_str().len() as u64))
                    .ok_or(StoreDenial::Representability)?;
            }
        }
        let memory = control.policy().reserve_memory(bytes);
        control.check_live()?;
        let memory = memory?;
        let mut locators = BTreeMap::new();
        let mut by_ordinal = Vec::with_capacity(count);
        let mut physical = true;
        let mut preimages: BTreeMap<_, BTreeMap<_, BTreeMap<_, BTreeMap<_, Vec<_>>>>> =
            BTreeMap::new();
        for ordinal in 0..count {
            control.check_live()?;
            super::super::super::fact_verification_observation::provider_application_fact();
            let fact = body.get(ordinal).ok_or(StoreDenial::ConflictingBody)?;
            physical &= matches!(
                fact,
                Fact::Entity { .. }
                    | Fact::Field { .. }
                    | Fact::AbsentField { .. }
                    | Fact::OptionalAspectAbsent { .. }
                    | Fact::Relation { .. }
                    | Fact::Adjacency { .. }
                    | Fact::StructuralWalk { .. }
            );
            physical &= matches!(scopes.get(ordinal),
                Some(worth_query_installation::facade::WorthQueryOperationGraphReadScope::Entity(_)
                    | worth_query_installation::facade::WorthQueryOperationGraphReadScope::NativeProjection(_)
                    | worth_query_installation::facade::WorthQueryOperationGraphReadScope::Relation(_)));
            let key = fact.dependency_locator_identity();
            crate::domain_computation::WorthQueryDecisionFactLocator::structural_proof(key.clone())
                .map_err(|_| StoreDenial::ConflictingBody)?;
            let key: Arc<str> = key.into();
            by_ordinal.push(key.clone());
            if locators.insert(key, ordinal).is_some() {
                return Err(StoreDenial::ConflictingBody);
            }
            if let Fact::Field { entity_id, locator, .. } = fact
                && let Some(worth_query_installation::facade::WorthQueryOperationGraphReadScope::NativeProjection(scope)) = scopes.get(ordinal)
                && scope.aspect() == locator.aspect().aspect_key()
                && scope.projection().mask().paths().contains(locator.field_path())
                && let Some(field) = crate::domain_computation::application_aftermath::demanded_field_slot(locator.field_path())
                && scopes.preimage_demand().is_some_and(|demand| demand.loci().iter().any(|locus|
                    locus.entity() == scope.entity().semantic_key()
                        && locus.aspect() == scope.aspect().as_str()
                        && locus.field() == field.as_str()))
            {
                preimages.entry(*entity_id).or_default()
                    .entry(scope.entity().semantic_key().to_owned()).or_default()
                    .entry(scope.aspect().as_str().to_owned()).or_default()
                    .entry(field.as_str().to_owned()).or_default().push(ordinal);
            }
        }
        Ok(Arc::new(Self {
            locators,
            by_ordinal,
            count,
            physical,
            bytes,
            _memory: memory,
            scopes,
            preimages,
        }))
    }
    pub(in crate::domain_computation) fn count(&self) -> usize {
        self.count
    }
    pub(in crate::domain_computation) fn physical(&self) -> bool {
        self.physical
    }
    pub(in crate::domain_computation) fn ordinal(&self, locator: &str) -> Option<usize> {
        self.locators.get(locator).copied()
    }
    pub(in crate::domain_computation) fn locator(&self, ordinal: usize) -> Option<&str> {
        self.by_ordinal.get(ordinal).map(AsRef::as_ref)
    }
    pub(in crate::domain_computation) fn locators(&self) -> impl Iterator<Item = (&str, usize)> {
        self.locators
            .iter()
            .map(|(key, ordinal)| (key.as_ref(), *ordinal))
    }
    pub(in crate::domain_computation::primary_graph) fn bytes(&self) -> u64 {
        self.bytes
    }
    pub(in crate::domain_computation::primary_graph) fn preimage_ordinals(
        &self,
        entity: worth_relational::facade::identity::EntityId,
        locus: &worth_query_installation::facade::InstalledPreImageLocus,
    ) -> &[usize] {
        self.preimages
            .get(&entity)
            .and_then(|entities| entities.get(locus.entity()))
            .and_then(|aspects| aspects.get(locus.aspect()))
            .and_then(|fields| fields.get(locus.field()))
            .map_or(&[], Vec::as_slice)
    }

    /// Only genuine native completion supplies this changed set. A changed
    /// structural key requires cold reconstruction rather than old routing.
    pub(super) fn refresh(
        self: &Arc<Self>,
        old: &FactPayload,
        new: &FactPayload,
        changed: &im::OrdSet<usize>,
        control: StorageControl<'_, '_>,
    ) -> Result<Arc<Self>, StoreDenial> {
        for &ordinal in changed {
            control.check_live()?;
            super::super::super::fact_verification_observation::provider_application_fact();
            let fact = new.get(ordinal).ok_or(StoreDenial::ConflictingBody)?;
            let scratch = control.policy().reserve_memory(
                fact.dependency_locator_byte_bound()
                    .and_then(|n| n.checked_mul(2))
                    .ok_or(StoreDenial::Representability)?,
            );
            control.check_live()?;
            let _scratch = scratch?;
            if ordinal >= self.count
                || old.get(ordinal).map(std::mem::discriminant)
                    != new.get(ordinal).map(std::mem::discriminant)
                || self.ordinal(&fact.dependency_locator_identity()) != Some(ordinal)
            {
                return Self::build(new, self.scopes.clone(), control);
            }
        }
        Ok(Arc::clone(self))
    }
}
