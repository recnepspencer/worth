//! Completion's routing over an admitted body: the attempt builds and refreshes the descriptive
//! ordinal projection (its data lives in the attempt's vocabulary), reading its own observed facts.
use super::super::WorthQueryApplicationObservedFact as Fact;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::application_projection::CompletedFactProjection;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::{
    control::StorageControl, StoreDenial,
};
use crate::domain_computation::primary_graph::vocabulary::application_attempt::scope_bindings::CompletedReadScopes;
use std::{collections::BTreeMap, sync::Arc};

/// Building and refreshing a completed projection reads the attempt's observed facts, so it
/// stays with the attempt. Implemented for the one vocabulary type and called statically.
pub(in crate::domain_computation::primary_graph) trait CompletedFactProjectionRouting {
    fn build(
        body: &FactPayload,
        scopes: CompletedReadScopes,
        control: StorageControl<'_, '_>,
    ) -> Result<Arc<CompletedFactProjection>, StoreDenial>;

    /// Only genuine native completion supplies this changed set. A changed
    /// structural key requires cold reconstruction rather than old routing.
    fn refresh(
        self: &Arc<Self>,
        old: &FactPayload,
        new: &FactPayload,
        changed: &im::OrdSet<usize>,
        control: StorageControl<'_, '_>,
    ) -> Result<Arc<CompletedFactProjection>, StoreDenial>;
}

impl CompletedFactProjectionRouting for CompletedFactProjection {
    fn build(
        body: &FactPayload,
        scopes: CompletedReadScopes,
        control: StorageControl<'_, '_>,
    ) -> Result<Arc<CompletedFactProjection>, StoreDenial> {
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
                    (std::mem::size_of::<CompletedFactProjection>() + 2 * std::mem::size_of::<usize>()) as u64,
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
            crate::domain_computation::primary_graph::fact_verification_observation::provider_application_fact();
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
            crate::domain_computation::primary_graph::fact_verification_observation::provider_application_fact();
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
                && let Some(field) = crate::domain_computation::application_aftermath::demanded_field_slot(locator.field_path())
                && scope.admits_field(field)
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
        Ok(Arc::new(CompletedFactProjection::from_routing(
            locators, by_ordinal, physical, bytes, memory, scopes, preimages,
        )))
    }
    fn refresh(
        self: &Arc<Self>,
        old: &FactPayload,
        new: &FactPayload,
        changed: &im::OrdSet<usize>,
        control: StorageControl<'_, '_>,
    ) -> Result<Arc<CompletedFactProjection>, StoreDenial> {
        for &ordinal in changed {
            control.check_live()?;
            crate::domain_computation::primary_graph::fact_verification_observation::provider_application_fact();
            let fact = new.get(ordinal).ok_or(StoreDenial::ConflictingBody)?;
            let scratch = control.policy().reserve_memory(
                fact.dependency_locator_byte_bound()
                    .and_then(|n| n.checked_mul(2))
                    .ok_or(StoreDenial::Representability)?,
            );
            control.check_live()?;
            let _scratch = scratch?;
            if ordinal >= self.count()
                || old.get(ordinal).map(std::mem::discriminant)
                    != new.get(ordinal).map(std::mem::discriminant)
                || self.ordinal(&fact.dependency_locator_identity()) != Some(ordinal)
            {
                return Self::build(new, self.scopes().clone(), control);
            }
        }
        Ok(Arc::clone(self))
    }
}
