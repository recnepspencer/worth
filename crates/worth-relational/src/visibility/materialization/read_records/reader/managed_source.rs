//! Source material follows the real pinned Native root, including pending World custody.
use super::VisibilityReadContext;
use crate::validation::candidate_source::{
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram,
};
use crate::{identity::data::EntityId, snapshots::data::SnapshotHandle};

impl VisibilityReadContext<'_> {
    /// Observe exact physical Source material at this genuine selected snapshot.
    /// This supplies no principal READ or installed invariant audience.
    pub fn observe_managed_index_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &SnapshotHandle,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
    ) -> Result<crate::mvcc::ManagedIndexSourceObservation, Denial> {
        let (selected, memory) = self.managed_source_position(execution, handle)?;
        let basis =
            crate::visibility::snapshot_states::resolve_snapshot_basis(self.runtime, handle)
                .ok_or(Denial::RootUnavailable)?;
        let publication = self.runtime.publication_binding();
        let epoch = publication
            .companion_epoch()
            .map_err(Denial::Registration)?;
        let (generation, _) = epoch
            .active()
            .map_err(Denial::Registration)?
            .ok_or(Denial::MissingParticipant)?;
        basis
            .root()
            .managed_source_image
            .get()
            .ok_or(Denial::RootUnavailable)?
            .observe(selected, program, scope, memory, generation)
    }

    /// Evict only the selected current image's derived material. Retained
    /// historical images remain temporal shares; this grants no Source READ.
    pub fn evict_managed_index_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &SnapshotHandle,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
    ) -> Result<
        Option<crate::validation::candidate_source::ManagedIndexSourceCacheRetirement>,
        Denial,
    > {
        let (selected, _memory) = self.managed_source_position(execution, handle)?;
        let basis =
            crate::visibility::snapshot_states::resolve_snapshot_basis(self.runtime, handle)
                .ok_or(Denial::RootUnavailable)?;
        let publication = self.runtime.publication_binding();
        let epoch = publication
            .companion_epoch()
            .map_err(Denial::Registration)?;
        let (generation, _) = epoch
            .active()
            .map_err(Denial::Registration)?
            .ok_or(Denial::MissingParticipant)?;
        let retired = basis
            .root()
            .managed_source_image
            .get()
            .ok_or(Denial::RootUnavailable)?
            .evict_selected(execution, &selected, program, scope, generation)?;
        drop(epoch);
        Ok(retired)
    }

    /// Capture exact selected Source custody. Query separately admits Source READ.
    pub fn prepare_managed_index_rebuild(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &SnapshotHandle,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
    ) -> Result<crate::mvcc::ManagedIndexSourceRebuild, Denial> {
        let (selected, memory) = self.managed_source_position(execution, handle)?;
        let projection = self
            .project_snapshot_in_request(handle, execution)
            .map_err(|cause| {
                Denial::Source(crate::derived_relation::DerivedRelationReadDenial::Request(
                    cause,
                ))
            })?
            .ok_or(Denial::RootUnavailable)?;
        let basis =
            crate::visibility::snapshot_states::resolve_snapshot_basis(self.runtime, handle)
                .ok_or(Denial::RootUnavailable)?;
        let publication = self.runtime.publication_binding();
        let epoch = publication
            .companion_epoch()
            .map_err(Denial::Registration)?;
        let (generation, _) = epoch
            .active()
            .map_err(Denial::Registration)?
            .ok_or(Denial::MissingParticipant)?;
        basis
            .root()
            .managed_source_image
            .get()
            .ok_or(Denial::RootUnavailable)?
            .prepare_rebuild(projection, selected, program, scope, memory, generation)
    }

    /// Install only into the same selected entry under the current registration.
    pub fn finish_managed_index_rebuild(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &SnapshotHandle,
        loan: &crate::mvcc::ManagedIndexSourceRebuild,
        payload: &std::sync::Arc<dyn std::any::Any + Send + Sync>,
    ) -> Result<
        (
            crate::mvcc::ManagedIndexSourceObservation,
            crate::validation::candidate_source::ManagedIndexSourceCacheRetirement,
        ),
        Denial,
    > {
        let (selected, memory) = self.managed_source_position(execution, handle)?;
        if !loan.projection.is_from_runtime(self.runtime) {
            return Err(Denial::ForeignSource);
        }
        let publication = self.runtime.publication_binding();
        let epoch = publication
            .companion_epoch()
            .map_err(Denial::Registration)?;
        let (generation, _) = epoch
            .active()
            .map_err(Denial::Registration)?
            .ok_or(Denial::MissingParticipant)?;
        let result = loan.finish(execution, selected, generation, payload, memory);
        drop(epoch);
        result
    }

    /// Currentness reborrows the consuming request, never the original READ.
    pub fn managed_index_source_is_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &SnapshotHandle,
        observation: &crate::mvcc::ManagedIndexSourceObservation,
    ) -> Result<bool, Denial> {
        let (selected, _memory) = self.managed_source_position(execution, handle)?;
        let publication = self.runtime.publication_binding();
        let epoch = publication
            .companion_epoch()
            .map_err(Denial::Registration)?;
        let (generation, _) = epoch
            .active()
            .map_err(Denial::Registration)?
            .ok_or(Denial::MissingParticipant)?;
        Ok(observation.remains_current_at(&selected, generation))
    }

    fn managed_source_position(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &SnapshotHandle,
    ) -> Result<
        (
            super::positioned_snapshot::PositionedRelationalSnapshot,
            Option<worth_execution::ExecutionMemoryReservation>,
        ),
        Denial,
    > {
        execution.consult().map_err(|cause| {
            Denial::Source(crate::derived_relation::DerivedRelationReadDenial::Request(
                cause.into(),
            ))
        })?;
        let mut result = None;
        let mut memory = None;
        let handoff = execution.run_native_with_checkpoint(|context| {
            result = Some(self.positioned_snapshot_admitted(handle, |units, bytes| {
                crate::execution::request_stage::checkpoint(context, units)
                    .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
                if bytes != 0 {
                    memory = execution
                        .reserve_retained_memory(bytes)
                        .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
                }
                Ok::<_, crate::execution::RelationalExecutionDenialCause>(())
            }));
        });
        use super::positioned_snapshot::RelationalSnapshotPositionAdmissionStop as PositionStop;
        let request_stop = |cause| {
            Denial::Source(crate::derived_relation::DerivedRelationReadDenial::Request(
                cause,
            ))
        };
        let selected = match (handoff, result) {
            (Ok(_), Some(Ok(selected))) => selected,
            (Ok(_), Some(Err(PositionStop::Admission(cause))))
            | (
                Err(worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::NestedStopped,
                )),
                Some(Err(PositionStop::Admission(cause))),
            ) => return Err(request_stop(cause)),
            (Err(cause), _) => return Err(request_stop(cause.into())),
            (Ok(_), Some(Err(PositionStop::AccountingOverflow))) => {
                return Err(request_stop(
                    crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
                ))
            }
            (Ok(_), Some(Err(PositionStop::Position(_)))) => return Err(Denial::RootUnavailable),
            (Ok(_), None) => unreachable!("the completed original Native stage ran once"),
        };
        Ok((selected, memory))
    }
}
