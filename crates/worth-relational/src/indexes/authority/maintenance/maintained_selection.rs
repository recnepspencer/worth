//! Synchronous index selection adopter; the native batch owns allocation custody.
use super::{
    candidate_publication::CandidateIndexPreparation, owner::IndexMaintenanceOwner,
    publication::PreparedRefresh, refresh_inputs::RootMaintenanceInputs, root_view::IndexRootView,
    work::MaintenanceWork,
};
use crate::indexes::data::*;
use crate::runtime::IndexSelectionState;
use crate::storage::substrate::maintenance::{
    ChangeWork, CommitChange, Maintained, MaintenanceConsistency, MaintenanceRoute,
    ReconstructionLane, ReconstructionWork,
};
use std::{collections::BTreeSet, sync::Arc};
use DerivedIndexMaintenanceDenialKind as Denial;

pub(super) struct IndexSelectionMaintainer<
    'a,
    'index,
    'payer,
    'run,
    'work_lease,
    'request,
    'request_lease,
> {
    pub(super) owner: &'a IndexMaintenanceOwner<'index>,
    pub(super) execution: &'a worth_execution::ExecutionRequest<'request, 'request_lease>,
    pub(super) before: &'a IndexRootView,
    pub(super) after: &'a IndexRootView,
    pub(super) request: &'a DerivedIndexBuildRequest,
    pub(super) native: &'a mut MaintenanceWork<'payer, 'run, 'work_lease>,
    pub(super) routes: Arc<InstalledIndexRouting>,
    pub(super) inputs: Option<RootMaintenanceInputs>,
    pub(super) prepared: Option<CandidateIndexPreparation>,
}
impl IndexSelectionMaintainer<'_, '_, '_, '_, '_, '_, '_> {
    fn capture(&mut self, refresh: PreparedRefresh) -> Result<Arc<IndexSelectionState>, Denial> {
        let prepared = CandidateIndexPreparation::prepare(
            self.owner.indexes,
            self.after.root(),
            refresh,
            self.native,
        )?;
        let state = prepared
            .selection_state()
            .ok_or(Denial::ColdReconstructionRequired)?;
        self.prepared = Some(prepared);
        Ok(state)
    }
    pub(super) fn finish(
        mut self,
        state: Arc<IndexSelectionState>,
    ) -> Result<CandidateIndexPreparation, Denial> {
        if let Some(prepared) = self.prepared.take() {
            return Ok(prepared);
        }
        let mut inputs = self.inputs.take().ok_or(Denial::BeforeRootMismatch)?;
        inputs
            .basis
            .as_mut()
            .ok_or(Denial::ColdReconstructionRequired)?
            .retain_state(state);
        let refresh = self.owner.prepare_refresh_with_inputs(
            self.execution,
            self.request,
            Some(self.before),
            self.after,
            self.native,
            Some(inputs),
        )?;
        CandidateIndexPreparation::prepare(
            self.owner.indexes,
            self.after.root(),
            refresh,
            self.native,
        )
    }
}
impl Maintained for IndexSelectionMaintainer<'_, '_, '_, '_, '_, '_, '_> {
    type State = IndexSelectionState;
    // Generations are immutable; the indexing catalog retires whole states by actual root custody.
    type Retirement = std::convert::Infallible;
    type Error = Denial;
    fn routes(&self) -> &BTreeSet<MaintenanceRoute> {
        self.routes.maintenance_routes()
    }
    fn consistency(&self) -> MaintenanceConsistency {
        MaintenanceConsistency::Synchronous
    }
    fn apply(
        &mut self,
        previous: &Arc<Self::State>,
        _change: &CommitChange,
        work: &mut ChangeWork<'_>,
    ) -> Result<Arc<Self::State>, Self::Error> {
        work.prepare(1, 0)?;
        let inputs = self.inputs.take().ok_or(Denial::BeforeRootMismatch)?;
        if !inputs
            .inherited
            .as_ref()
            .is_some_and(|basis| Arc::ptr_eq(&basis.selection_state(), previous))
        {
            return Err(Denial::BeforeRootMismatch);
        }
        let mut refresh = self.owner.prepare_refresh_with_inputs(
            self.execution,
            self.request,
            Some(self.before),
            self.after,
            self.native,
            Some(inputs),
        )?;
        if refresh.entries.is_empty() {
            refresh
                .basis
                .as_mut()
                .ok_or(Denial::ColdReconstructionRequired)?
                .retain_state(Arc::clone(previous));
        }
        self.capture(refresh)
    }
    fn cold(
        &mut self,
        lane: &ReconstructionLane<'_>,
        work: &mut ReconstructionWork<'_, '_>,
    ) -> Result<Arc<Self::State>, Self::Error> {
        work.prepare(1, 0)?;
        let refresh = self.owner.prepare_recovery(
            lane,
            self.execution,
            self.request,
            self.after,
            self.native,
        )?;
        self.capture(refresh)
    }
    fn retire(
        &mut self,
        _: &Arc<Self::State>,
        horizon: &Self::Retirement,
        _: &mut ChangeWork<'_>,
    ) -> Result<Arc<Self::State>, Self::Error> {
        match *horizon {}
    }
}
