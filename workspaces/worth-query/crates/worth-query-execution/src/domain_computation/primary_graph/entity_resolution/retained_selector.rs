//! Descriptive scope retained from native entity resolution, never read authority.
use super::{WorthQueryApplicationEntityIdentity, WorthQueryResolvedEntity};
use std::sync::Arc;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::EntityId;
mod funding;

pub struct WorthQueryObservedScopeSelector {
    description: ScopeDescription,
    // Last to drop: the original request remains charged through the final share.
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}

enum ScopeDescription {
    Resolved(WorthQueryResolvedEntity),
    Recorded {
        locator: AspectFieldLocator,
        value: AspectValue,
        subject: EntityId,
    },
}
impl ScopeDescription {
    fn retained_capacity_bytes(&self) -> Option<usize> {
        match self {
            Self::Resolved(value) => value.retained_capacity_bytes(),
            Self::Recorded { locator, value, .. } => locator
                .owned_allocation_capacity_bytes()
                .checked_add(value.owned_allocation_capacity_bytes()),
        }
    }
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
            .map(FieldKey::new)
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
            description: ScopeDescription::Recorded {
                locator,
                value,
                subject,
            },
            _funding: funding,
        }))
    }
    /// The original owner-sealed description is sufficient to request current
    /// authorization; it proves neither presence nor permission at a new basis.
    fn describe<Schema, Entity>(
        &self,
    ) -> Option<WorthQueryApplicationEntityIdentity<Schema, Entity>> {
        match &self.description {
            ScopeDescription::Resolved(value) => Some(value.clone().into_application_identity()),
            ScopeDescription::Recorded { .. } => None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn matches_source<Q>(
        &self,
        source: &super::super::WorthQueryObservedSource<Q>,
    ) -> bool {
        match &self.description {
            ScopeDescription::Resolved(value) => value.matches_source(source),
            ScopeDescription::Recorded { .. } => false,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn locator(&self) -> &AspectFieldLocator {
        match &self.description {
            ScopeDescription::Resolved(value) => value.locator(),
            ScopeDescription::Recorded { locator, .. } => locator,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn value(&self) -> &AspectValue {
        match &self.description {
            ScopeDescription::Resolved(value) => value.value(),
            ScopeDescription::Recorded { value, .. } => value,
        }
    }
    pub(in crate::domain_computation) fn subject(&self) -> EntityId {
        match &self.description {
            ScopeDescription::Resolved(value) => value.entity_id(),
            ScopeDescription::Recorded { subject, .. } => *subject,
        }
    }
}

impl<Schema, Entity> WorthQueryApplicationEntityIdentity<Schema, Entity> {
    pub(in crate::domain_computation) fn retain_scope_selector(
        &self,
    ) -> Arc<WorthQueryObservedScopeSelector> {
        Arc::new(WorthQueryObservedScopeSelector {
            description: ScopeDescription::Resolved(self.retain_description()),
            _funding: None,
        })
    }
}

mod court_doors_for_test;
