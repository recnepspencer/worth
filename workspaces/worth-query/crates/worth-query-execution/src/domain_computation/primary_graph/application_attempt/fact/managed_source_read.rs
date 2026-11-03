//! Managed expectations retain owner-issued support, never an Execution loan.
use crate::domain_computation::primary_graph::access_strategy::realized_footprint::{
    CapturedManagedInfluence, CheckedRealizedSource, ManagedHistoryRelevanceDenial as HistoryDenial,
};
use std::sync::Arc;
use worth_relational::facade::{
    history::{
        RelationalAuthoredManagedSourceImpact, RelationalAuthoredWriteDenial,
        RelationalAuthoredWriteOccurrence,
    },
    identity::EntityId,
    runtime::{ManagedIndexSourceObservation, RelationalRuntime},
    snapshots::SnapshotHandle,
};

/// Descriptive locator within the already checked schema/runtime/branch affinity.
/// The retained Native observation, not this key, supplies Source currentness.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct ManagedSourceReadKey {
    pub obligation: [u8; 32],
    pub authorization: [u8; 32],
    pub scope: EntityId,
    pub influence: Option<ManagedInfluenceOrigin>,
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct ManagedInfluenceOrigin {
    pub parameters: [u8; 32],
    pub audience: [u8; 32],
    pub content: worth_foundational::facade::ProjectedCandidateContentV1,
}

#[derive(Clone)]
pub(in crate::domain_computation) struct ManagedSourceRead {
    key: ManagedSourceReadKey,
    authorization: Arc<super::SourceReadAuthorization>,
    native: Arc<ManagedIndexSourceObservation>,
    influence: Option<Arc<CapturedManagedInfluence>>,
    _allocation: Arc<crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation>,
}
pub(in crate::domain_computation::primary_graph) struct CheckedManagedSource(CheckedSource);
enum CheckedSource {
    Whole(Arc<ManagedIndexSourceObservation>),
    Realized(CheckedRealizedSource),
}
impl CheckedManagedSource {
    pub(in crate::domain_computation::primary_graph) fn is_realized(&self) -> bool {
        matches!(self.0, CheckedSource::Realized(_))
    }
    pub(in crate::domain_computation::primary_graph) fn native(
        &self,
    ) -> &ManagedIndexSourceObservation {
        match &self.0 {
            CheckedSource::Whole(value) => value,
            CheckedSource::Realized(value) => value.native(),
        }
    }
}
impl ManagedSourceRead {
    pub(in crate::domain_computation::primary_graph) fn new(
        obligation: [u8; 32],
        scope: EntityId,
        native: Arc<ManagedIndexSourceObservation>,
        allocation: Arc<
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
        >,
        influence: Option<Arc<CapturedManagedInfluence>>,
        parameters: [u8; 32],
        authorization: Arc<super::SourceReadAuthorization>,
    ) -> Self {
        Self {
            key: ManagedSourceReadKey {
                obligation,
                authorization: authorization.identity(),
                scope,
                influence: influence.as_ref().map(|capture| ManagedInfluenceOrigin {
                    parameters,
                    audience: capture.audience(),
                    content: capture.content(),
                }),
            },
            native,
            authorization,
            influence,
            _allocation: allocation,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn authorization(
        &self,
    ) -> &Arc<super::SourceReadAuthorization> {
        &self.authorization
    }
    pub(in crate::domain_computation::primary_graph) fn key(&self) -> &ManagedSourceReadKey {
        &self.key
    }
    pub(in crate::domain_computation::primary_graph) fn native(
        &self,
    ) -> &ManagedIndexSourceObservation {
        &self.native
    }
    pub(in crate::domain_computation::primary_graph) fn has_verified_realized_footprint(
        &self,
    ) -> bool {
        self.influence.is_some()
    }
    pub(in crate::domain_computation::primary_graph) fn history_selector(
        &self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Option<worth_relational::facade::history::RelationalAuthoredManagedSourceSelector>,
        RelationalAuthoredWriteDenial,
    > {
        match &self.influence {
            Some(capture) => capture.history_selector(runtime, execution),
            None => Ok(None),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn history_impact(
        &self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        occurrence: &RelationalAuthoredWriteOccurrence,
    ) -> Result<RelationalAuthoredManagedSourceImpact, HistoryDenial> {
        self.history_capture()?
            .history_impact(runtime, execution, occurrence)
    }
    /// Caller must have released runtime/attempt gates before opaque acquisition.
    pub(in crate::domain_computation::primary_graph) fn history_relevant(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        impact: RelationalAuthoredManagedSourceImpact,
    ) -> Result<bool, HistoryDenial> {
        self.history_capture()?.history_relevant(execution, impact)
    }
    fn history_capture(&self) -> Result<&CapturedManagedInfluence, HistoryDenial> {
        self.influence.as_deref().ok_or(HistoryDenial::Native(
            RelationalAuthoredWriteDenial::UnsupportedManagedSourceSelector,
        ))
    }
    pub(in crate::domain_computation::primary_graph) fn same_truth<E>(
        &self,
        other: &Self,
        admit: &mut impl FnMut(u64) -> Result<(), E>,
        reserve: &mut impl FnMut(u64) -> Result<Option<worth_execution::ExecutionMemoryReservation>, E>,
        fault: &mut impl FnMut(crate::domain_computation::primary_graph::application_query::ProjectionContentComparisonFault) -> E,
        frames: &mut dyn FnMut(
            &worth_relational::facade::content::RelationalContentFrame,
            &worth_relational::facade::content::RelationalContentFrame,
        ) -> Result<bool, E>,
    ) -> Result<bool, E> {
        if self.key != other.key || !self.authorization.same_truth(&other.authorization, admit)? {
            return Ok(false);
        }
        Ok(match (&self.influence, &other.influence) {
            (None, None) => self.native.same_source(&other.native),
            (Some(left), Some(right)) if Arc::ptr_eq(left, right) => true,
            (Some(left), Some(right)) => {
                return left.same_inputs(right, admit, reserve, fault, frames);
            }
            _ => false,
        })
    }
    pub(in crate::domain_computation::primary_graph) fn same_selected_membership<E>(
        &self,
        other: &Self,
        admit: &mut impl FnMut(u64) -> Result<(), E>,
    ) -> Result<bool, E> {
        if !self.authorization.same_truth(&other.authorization, admit)? {
            return Ok(false);
        }
        let (Some(left_origin), Some(right_origin), Some(left), Some(right)) = (
            self.key.influence,
            other.key.influence,
            &self.influence,
            &other.influence,
        ) else {
            return Ok(false);
        };
        if self.key.obligation != other.key.obligation
            || self.key.scope != other.key.scope
            || left_origin.parameters != right_origin.parameters
            || left_origin.audience != right_origin.audience
        {
            return Ok(false);
        }
        left.same_selected_membership(right, admit)
    }
    pub(in crate::domain_computation::primary_graph) fn remains_equal(
        &self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, super::WorthQuerySourceCurrentnessFailure> {
        self.checked_at(runtime, execution, snapshot)
            .map(|checked| checked.is_some())
    }
    pub(in crate::domain_computation::primary_graph) fn checked_at(
        &self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
    ) -> Result<Option<CheckedManagedSource>, super::WorthQuerySourceCurrentnessFailure> {
        self.authorization
            .validate_current(runtime, execution, snapshot)
            .map_err(super::WorthQuerySourceCurrentnessFailure::OutputObservation)?;
        if self.whole_source_is_current(runtime, execution, snapshot)? {
            return Ok(Some(CheckedManagedSource(CheckedSource::Whole(
                Arc::clone(&self.native),
            ))));
        }
        match &self.influence {
            Some(influence) => influence
                .revalidate(runtime, execution, snapshot)
                .map(|checked| {
                    checked.map(|value| CheckedManagedSource(CheckedSource::Realized(value)))
                }),
            None => Ok(None),
        }
    }
    fn whole_source_is_current(
        &self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, super::WorthQuerySourceCurrentnessFailure> {
        use super::WorthQuerySourceCurrentnessFailure as Stop;
        runtime
            .read_truth()
            .managed_index_source_is_current(execution, snapshot, &self.native)
            .map_err(|cause| match cause {
                worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial::Source(
                    worth_relational::facade::runtime::DerivedRelationReadDenial::Request(cause),
                ) => Stop::Execution(cause),
                _ => Stop::Unavailable,
            })
    }
}
impl PartialEq for ManagedSourceRead {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.authorization == other.authorization
            && self.native.same_source(&other.native)
            && match (&self.influence, &other.influence) {
                (None, None) => true,
                (Some(left), Some(right)) => Arc::ptr_eq(left, right),
                _ => false,
            }
    }
}
impl Eq for ManagedSourceRead {}
impl std::fmt::Debug for ManagedSourceRead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManagedSourceRead")
            .field("key", &self.key)
            .field("revision", &self.native.source_revision())
            .finish_non_exhaustive()
    }
}
