use super::*;
use crate::visibility::snapshot_states::build_visibility_state;

impl<'runtime> VisibilityReadContext<'runtime> {
    pub fn inspect_snapshot(&self, handle: &SnapshotHandle) -> Option<SnapshotInspectionSummary> {
        resolve_snapshot_inspection(self.runtime, handle)
    }

    /// A snapshot read requires the caller's borrowed execution custody.
    /// ```compile_fail,E0061
    /// use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};
    /// use worth_execution::ExecutionRequest;
    /// fn read(runtime: &RelationalRuntime, snapshot: &SnapshotHandle, request: &ExecutionRequest<'_, '_>) {
    ///     let _ = runtime.read_truth().read_snapshot(snapshot);
    /// }
    /// ```
    /// ```
    /// use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};
    /// use worth_execution::ExecutionRequest;
    /// fn read(runtime: &RelationalRuntime, snapshot: &SnapshotHandle, request: &ExecutionRequest<'_, '_>) {
    ///     let _ = runtime.read_truth().read_snapshot(snapshot, request);
    /// }
    /// ```
    pub fn read_snapshot(
        &self,
        handle: &SnapshotHandle,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<RelationalReadView>, crate::execution::RelationalExecutionDenialCause> {
        if let Some(basis) = resolve_snapshot_basis(self.runtime, handle) {
            basis
                .root()
                .demand_all_full(execution)
                .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
        }
        super::super::read_work::metered_with_memory(
            execution,
            |context, memory| {
                let Some(resolved) = resolve_snapshot_state(self.runtime, handle) else {
                    return Ok(None);
                };
                if resolved
                    .state
                    .basis
                    .root()
                    .is_some_and(|root| root.has_materialization_unavailable())
                {
                    return Ok(None);
                }
                read_view_from_snapshot_state(
                    self.runtime,
                    &resolved.state,
                    &resolved.handle,
                    &mut |units| context.checkpoint(units),
                    memory,
                )
                .map(Some)
            },
            |answer| {
                answer
                    .as_ref()
                    .ok()
                    .and_then(Option::as_ref)
                    .map_or(0, read_view_bytes)
            },
        )?
    }

    pub(crate) fn read_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        version_id: crate::identity::data::VersionId,
    ) -> Result<RelationalReadView, crate::execution::RelationalExecutionDenialCause> {
        Ok(self
            .try_read_version(execution, version_id)?
            .expect("internal historical read requires retained MVCC coverage"))
    }

    pub(crate) fn try_read_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        version_id: crate::identity::data::VersionId,
    ) -> Result<
        Result<RelationalReadView, crate::visibility::snapshot_states::HistoricalVisibilityDenial>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        let state = materialize_historical_visibility(execution, self.runtime, version_id, true)?;
        let Some(state) = state else {
            return Ok(Err(crate::visibility::cache_state::historical_basis_for_retained_version(self.runtime, version_id).err().unwrap_or(crate::visibility::snapshot_states::HistoricalVisibilityDenial::CertificationReconstructionRequired)));
        };
        let view = super::super::read_work::metered_with_memory(
            execution,
            |context, memory| {
                read_view_from_snapshot_state(
                    self.runtime,
                    &state,
                    &state.handle,
                    &mut |units| context.checkpoint(units),
                    memory,
                )
            },
            |answer| answer.as_ref().map_or(0, read_view_bytes),
        )??;
        Ok(Ok(view))
    }

    pub fn query_plan_context(&self, handle: &SnapshotHandle) -> Option<QueryPlanContextId> {
        let snapshot = self.resolved_snapshot_handle(handle)?;
        let basis = resolve_snapshot_basis(self.runtime, &snapshot)?;
        if basis.root().has_materialization_unavailable() {
            return None;
        }
        let root = basis.root();
        let (schema_version, descriptor_semantics_version, evidence_basis) =
            query_schema_context_for_root(root);
        Some(QueryPlanContextId {
            runtime_instance_id: self.runtime.runtime_instance_id(),
            snapshot_id: snapshot.snapshot_id,
            version_id: snapshot.version_id,
            schema_version,
            descriptor_semantics_version,
            evidence_basis,
        })
    }

    /// Read the immutable root carried by one owner-admitted observation.
    pub fn read_observation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &crate::mvcc::RelationalBranchObservation,
    ) -> Result<
        Result<RelationalReadView, crate::branch::RelationalBranchBasisDenial>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        crate::execution::check_request(execution)?;
        if let Err(denial) = self.require_local_observation(observation) {
            return Ok(Err(denial));
        }

        if observation
            .selected_root()
            .has_materialization_unavailable()
        {
            return Ok(Err(
                crate::branch::RelationalBranchBasisDenial::MaterializationUnavailable,
            ));
        }
        observation
            .selected_root()
            .demand_all_full(execution)
            .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
        let state = build_visibility_state(
            self.runtime,
            crate::visibility::snapshot_states::VisibilitySnapshotBasis::from_observation(
                observation,
            ),
            crate::snapshots::data::SnapshotId(0),
            SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
        );
        let view = super::super::read_work::metered_with_memory(
            execution,
            |context, memory| {
                read_view_from_snapshot_state(
                    self.runtime,
                    &state,
                    &state.handle,
                    &mut |units| context.checkpoint(units),
                    memory,
                )
            },
            |answer| answer.as_ref().map_or(0, read_view_bytes),
        )??;
        Ok(Ok(view))
    }

    /// Schema version carried by the observation's selected canonical root.
    pub fn observation_schema_version(
        &self,
        observation: &crate::mvcc::RelationalBranchObservation,
    ) -> Result<crate::schema::data::SchemaVersionId, crate::branch::RelationalBranchBasisDenial>
    {
        self.require_local_observation(observation)?;
        Ok(observation
            .selected_root()
            .schema_authority()
            .schema_version())
    }

    /// Observe the schema version carried by the exact canonical snapshot
    /// basis. This is descriptive read evidence and grants no schema or
    /// publication authority.
    pub fn snapshot_schema_version(
        &self,
        handle: &SnapshotHandle,
    ) -> Option<crate::schema::data::SchemaVersionId> {
        let basis = resolve_snapshot_basis(self.runtime, handle)?;
        Some(basis.root().schema_authority().schema_version())
    }

    pub(super) fn resolved_snapshot_handle(
        &self,
        handle: &SnapshotHandle,
    ) -> Option<SnapshotHandle> {
        resolve_snapshot_handle(self.runtime, handle)
    }

    fn require_local_observation(
        &self,
        observation: &crate::mvcc::RelationalBranchObservation,
    ) -> Result<(), crate::branch::RelationalBranchBasisDenial> {
        if observation.identity().runtime_instance_id() != self.runtime.runtime_instance_id() {
            return Err(crate::branch::RelationalBranchBasisDenial::ForeignRuntime {
                expected_runtime_instance_id: self.runtime.runtime_instance_id(),
                actual_runtime_instance_id: observation.identity().runtime_instance_id(),
            });
        }
        Ok(())
    }
}

pub(super) fn query_schema_context_for_root(
    root: &crate::branch::RelationalBranchRoot,
) -> (
    crate::schema::data::SchemaVersionId,
    crate::schema::data::DescriptorSemanticsVersion,
    QueryPlanEvidenceBasis,
) {
    let evidence_basis = root.canonical_envelope().map_or(
        QueryPlanEvidenceBasis::GenesisRuntimeBootstrap,
        |envelope| QueryPlanEvidenceBasis::CanonicalCommitEnvelope {
            commit_id: envelope.commit.commit_id,
        },
    );
    (
        root.schema_authority().schema_version(),
        root.schema_authority().descriptor_semantics_version(),
        evidence_basis,
    )
}

fn read_view_bytes(view: &RelationalReadView) -> u64 {
    super::super::read_footprint::entities(&view.entities)
        .saturating_add(super::super::read_footprint::relations(&view.relations))
        .saturating_add(view.snapshot.branch_id.0.capacity() as u64)
}
