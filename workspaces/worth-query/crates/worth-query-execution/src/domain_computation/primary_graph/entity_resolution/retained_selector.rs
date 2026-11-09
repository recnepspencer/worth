//! Descriptive scope retained from native entity resolution, never read authority.
use super::WorthQueryApplicationEntityIdentity;
use std::sync::Arc;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::EntityId;
mod funding;

pub struct WorthQueryObservedScopeSelector {
    locator: AspectFieldLocator,
    value: AspectValue,
    subject: EntityId,
    // Last to drop: the original request remains charged through the final share.
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}

impl WorthQueryObservedScopeSelector {
    pub(in crate::domain_computation::primary_graph) fn readmit_recorded(
        aspect: &str,
        fields: &[String],
        value: AspectValue,
        subject: EntityId,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, crate::facade::correction_material::QueryCorrectionMaterialDenial> {
        use crate::facade::correction_material::QueryCorrectionMaterialDenial as D;
        use worth_foundational::facade::{
            AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority,
        };
        let path = fields
            .iter()
            .map(|key| FieldKey::new(key))
            .collect::<Option<Vec<_>>>()
            .ok_or(D::MalformedFrame)?;
        let locator = AspectFieldLocator::new(
            LocatorAuthority::Planned,
            AspectKey::new(aspect).ok_or(D::MalformedFrame)?,
            CanonicalFieldPath::new(path).ok_or(D::MalformedFrame)?,
        );
        Self::from_recorded(locator, value, subject, execution).map_err(D::Execution)
    }
    /// Reconstitute descriptive scope DATA; fresh entity resolution still checks it.
    pub(in crate::domain_computation::primary_graph) fn from_recorded(
        locator: AspectFieldLocator,
        value: AspectValue,
        subject: EntityId,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, worth_execution::WorkCeilingDenial> {
        let bytes = std::mem::size_of::<Self>()
            .checked_add(2 * std::mem::size_of::<usize>())
            .and_then(|bytes| bytes.checked_add(locator.owned_allocation_capacity_bytes()))
            .and_then(|bytes| bytes.checked_add(value.owned_allocation_capacity_bytes()))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(worth_execution::WorkCeilingDenial::Admission(
                worth_execution::LeaseDenial::ChargedBytesOverflow,
            ))?;
        execution.consult()?;
        let funding = execution.reserve_retained_memory(bytes)?;
        Ok(Arc::new(Self {
            locator,
            value,
            subject,
            _funding: funding,
        }))
    }
    pub(in crate::domain_computation::primary_graph) fn locator(&self) -> &AspectFieldLocator {
        &self.locator
    }
    pub(in crate::domain_computation::primary_graph) fn value(&self) -> &AspectValue {
        &self.value
    }
    pub(in crate::domain_computation) fn subject(&self) -> EntityId {
        self.subject
    }
}

impl<Schema, Entity> WorthQueryApplicationEntityIdentity<Schema, Entity> {
    pub(in crate::domain_computation) fn retain_scope_selector(
        &self,
    ) -> Arc<WorthQueryObservedScopeSelector> {
        Arc::new(WorthQueryObservedScopeSelector {
            locator: self.identity_locator().clone(),
            value: self.identity_value().clone(),
            subject: self.entity_id(),
            _funding: None,
        })
    }
}

mod court_doors_for_test;
