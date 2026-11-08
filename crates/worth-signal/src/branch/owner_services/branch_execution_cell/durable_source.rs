//! Immutable source inspection cannot acquire a second branch or mutate a head.
use super::super::{SignalBranchCellState, SignalOwner, SignalOwnerOperationAdmission};
use super::{SignalBranchCellAdmissionDenial, SignalBranchExecutionCell};
use crate::branch::owner_services::durable_source_port::SignalDurableSourceDenial as D;
use crate::branch::{AdmittedSignalBranchBasis, SignalBranchObservation};
use crate::durability::{
    SignalAuthoredImage, SignalDurableTierVocabulary, SignalRecordedSourcePoint,
    SignalRuntimeAuthoredImage,
};
use worth_execution::ExecutionRequest;

impl<Dm, I, T> SignalBranchExecutionCell<SignalBranchCellState<Dm, I, T>>
where
    Dm: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn inspect_durable_source<R>(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        inspect: impl FnOnce(&SignalBranchCellState<Dm, I, T>) -> Result<R, D>,
    ) -> Result<R, D> {
        let map =
            |denial| D::Observation(super::basis::map_basis_cell_denial(denial, self.branch_id));
        self.validate_admission(admission).map_err(map)?;
        let _hold = admission
            .hold_branch_cell()
            .map_err(SignalBranchCellAdmissionDenial::from)
            .map_err(map)?;
        self.counters.record_target_cell_contact();
        self.contacts
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let state = self
            .lock_state_after_contention_observation()
            .map_err(map)?;
        self.require_live_posture().map_err(map)?;
        inspect(&state)
    }
    pub(in crate::branch::owner_services) fn observe_durable_source(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
    ) -> Result<(SignalRecordedSourcePoint, SignalBranchObservation), D> {
        self.inspect_durable_source(admission, |state| {
            let descriptor = state
                .durable_source_point()
                .map_err(D::Frame)?
                .ok_or(D::SourceNotInstalled)?;
            let observation = state.observation().map_err(|error| {
                D::Observation(
                    crate::branch::SignalBranchBasisObservationDenial::InvalidOwnerObservation {
                        error,
                    },
                )
            })?;
            Ok((descriptor, observation))
        })
    }
    pub(in crate::branch::owner_services) fn capture_durable_source(
        &self,
        owner: &SignalOwner<Dm, I, T>,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        vocabulary: &SignalDurableTierVocabulary<T>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(SignalAuthoredImage, SignalRuntimeAuthoredImage), D> {
        self.inspect_durable_source(admission, |state| {
            let live = state.observation().map_err(|error| {
                D::Observation(
                    crate::branch::SignalBranchBasisObservationDenial::InvalidOwnerObservation {
                        error,
                    },
                )
            })?;
            if !owner.is_current_canonical_basis(
                basis,
                self.branch_id,
                self.incarnation().get(),
                &live,
            ) {
                return Err(D::ForeignBasis);
            }
            let branch = state
                .durable_descriptor()
                .map_err(D::Frame)?
                .ok_or(D::SourceNotInstalled)?
                .branch();
            let graph = state.state().graph();
            Ok((
                graph
                    .capture_authored_image(execution, branch)
                    .map_err(D::Image)?,
                state
                    .state()
                    .config()
                    .capture_authored_image(graph, vocabulary, branch, execution)
                    .map_err(D::Image)?,
            ))
        })
    }
    pub(in crate::branch::owner_services) fn recorded_source_bindings(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        descriptor: &crate::durability::SignalDurableBranchDescriptor,
        restored: bool,
    ) -> Result<
        (
            crate::data::graph::SignalGraphInstallationBindings,
            SignalBranchObservation,
        ),
        D,
    > {
        self.inspect_durable_source(admission, |state| {
            let actual = state.durable_descriptor().map_err(D::Frame)?.ok_or(D::SourceNotInstalled)?;
            if actual.cell() != descriptor.cell() {
                return Err(D::DescriptorMismatch);
            }
            let observation = state.recorded_source_observation(descriptor, restored)
                .map_err(|error| D::Observation(crate::branch::SignalBranchBasisObservationDenial::InvalidOwnerObservation { error }))?;
            Ok((crate::data::graph::SignalGraphInstallationBindings::capture(state.state().graph()), observation))
        })
    }
}
