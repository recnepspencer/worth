use super::*;
use crate::visibility::snapshot_states::build_visibility_state;

impl<'runtime> VisibilityReadContext<'runtime> {
    pub fn inspect_snapshot(&self, handle: &SnapshotHandle) -> Option<SnapshotInspectionSummary> {
        resolve_snapshot_inspection(self.runtime, handle)
    }

    pub fn read_snapshot(&self, handle: &SnapshotHandle) -> Option<RelationalReadView> {
        let resolved = resolve_snapshot_state(self.runtime, handle)?;
        if resolved
            .state
            .basis
            .root()
            .is_some_and(|root| root.has_materialization_unavailable())
        {
            return None;
        }
        Some(read_view_from_snapshot_state(
            self.runtime,
            &resolved.state,
            &resolved.handle,
        ))
    }

    pub(crate) fn read_version(
        &self,
        version_id: crate::identity::data::VersionId,
    ) -> RelationalReadView {
        self.try_read_version(version_id)
            .expect("internal historical read requires retained MVCC coverage")
    }

    pub(crate) fn try_read_version(
        &self,
        version_id: crate::identity::data::VersionId,
    ) -> Result<RelationalReadView, crate::visibility::snapshot_states::HistoricalVisibilityDenial>
    {
        let state = materialize_historical_visibility(self.runtime, version_id, true).ok_or_else(
            || {
                crate::visibility::cache_state::historical_basis_for_retained_version(
                    self.runtime,
                    version_id,
                )
                .err()
                .unwrap_or(
                    crate::visibility::snapshot_states::HistoricalVisibilityDenial::CertificationReconstructionRequired,
                )
            },
        )?;
        let handle = state.handle.clone();
        Ok(read_view_from_snapshot_state(self.runtime, &state, &handle))
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
        observation: &crate::mvcc::RelationalBranchObservation,
    ) -> Result<RelationalReadView, crate::branch::RelationalBranchBasisDenial> {
        self.require_local_observation(observation)?;
        if observation
            .selected_root()
            .has_materialization_unavailable()
        {
            return Err(crate::branch::RelationalBranchBasisDenial::MaterializationUnavailable);
        }
        let state = build_visibility_state(
            self.runtime,
            crate::visibility::snapshot_states::VisibilitySnapshotBasis::from_observation(
                observation,
            ),
            crate::snapshots::data::SnapshotId(0),
            crate::snapshots::data::SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
        );
        let handle = state.handle.clone();
        Ok(read_view_from_snapshot_state(self.runtime, &state, &handle))
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

fn query_schema_context_for_root(
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
