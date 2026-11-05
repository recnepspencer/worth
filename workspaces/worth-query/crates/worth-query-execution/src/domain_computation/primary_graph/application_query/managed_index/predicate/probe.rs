//! Actual admitted Parent parameters retain their original funded raw probe.
use crate::domain_computation::primary_graph::{
    application_query::one_shot::outcome::material::MaterialCompletion,
    source_facts::SourceFactsAllocation,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::AspectValue;
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters as Parameters;
use worth_query_installation::facade::WorthQueryInstalledManagedIndexParent as Parent;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
use {
    crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial as Denial,
    crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop as Stop,
};

pub(in crate::domain_computation::primary_graph::application_query) struct RetainedPredicateProbe {
    pub(super) parent: Parent,
    pub(super) parameters: Parameters,
    ordinal: usize,
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) retained_bytes:
        u64,
    _allocation: Arc<SourceFactsAllocation>,
}
impl RetainedPredicateProbe {
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn parent(
        &self,
    ) -> &Parent {
        &self.parent
    }
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn parameters(
        &self,
    ) -> &Parameters {
        &self.parameters
    }
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn matches_probe_borrow(
        &self,
        parameters: &Parameters,
        probe: &AspectValue,
    ) -> bool {
        self.parameters.identity() == parameters.identity()
            && parameters
                .bindings()
                .get(self.ordinal)
                .is_some_and(|(_, actual)| std::ptr::eq(actual, probe))
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn matches_parameters(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        parent: &Parent,
        parameters: &Parameters,
    ) -> Result<bool, Cause> {
        let work = self
            .parameters
            .retained_owned_capacity_bytes()
            .and_then(|a| {
                parameters
                    .retained_owned_capacity_bytes()
                    .and_then(|b| a.checked_add(b))
            })
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::WorkCounterOverflow)?;
        SourceFactsAllocation::work(execution, work)?;
        Ok(self.parent.same_association(parent)
            && self.parameters.bindings() == parameters.bindings())
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn capture(
        execution: &ExecutionRequest<'_, '_>,
        parent: &Parent,
        parameters: &Parameters,
    ) -> Result<Arc<Self>, Denial> {
        let allocation = SourceFactsAllocation::open(execution).map_err(storage)?;
        let bytes = parameters
            .retained_owned_capacity_bytes()
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(|| storage(Cause::ChargedBytesOverflow))?;
        let material = MaterialCompletion::new(execution, &allocation);
        material.arc::<Self>().map_err(storage)?;
        allocation.claim(execution, bytes).map_err(storage)?;
        material.work(bytes).map_err(storage)?;
        let name = &parent
            .access_contract()
            .meaning()
            .parent_graph()
            .managed_index_selection()
            .ok_or(Stop::MembershipUnavailable)?
            .parts()
            .probe_parameter;
        let work = parameters
            .bindings()
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_mul(name.len().checked_add(1)?))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Stop::CapacityOverflow)?;
        material.work(work).map_err(storage)?;
        let ordinal = parameters
            .bindings()
            .iter()
            .position(|(key, _)| *key == name)
            .ok_or(Stop::MembershipUnavailable)?;
        Ok(Arc::new(Self {
            parent: parent.clone(),
            parameters: parameters.clone(),
            ordinal,
            retained_bytes: bytes,
            _allocation: allocation,
        }))
    }
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn raw(
        &self,
    ) -> &AspectValue {
        // Capture resolved this ordinal against the actual immutable full parameter set.
        &self.parameters.bindings()[self.ordinal].1
    }
}
pub(super) fn storage(cause: Cause) -> Denial {
    Stop::MaterialStorage(cause).into()
}
