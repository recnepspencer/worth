use crate::runtime::RelationalRuntime;
use crate::visibility::exact_commit_snapshot::{
    open_retained_commit_snapshot, projection_binding_denial,
    RelationalRetainedCommitEntityProjection, RelationalRetainedCommitSnapshot,
    RelationalRetainedCommitSnapshotDenial, RelationalRetainedCommitSnapshotDenialKind,
};

pub struct VisibilityAuthority<'runtime> {
    pub(super) runtime: &'runtime RelationalRuntime,
}

impl RelationalRuntime {
    pub(crate) fn visibility_authority(&self) -> VisibilityAuthority<'_> {
        VisibilityAuthority::new(self)
    }
}

impl<'runtime> VisibilityAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    /// Observes the already-published snapshot for one exact canonical commit.
    ///
    /// This does not open a current branch head, allocate a snapshot identity,
    /// reconstruct historical state, or acquire replay retention. A pruned
    /// publication is therefore a typed denial rather than a reconstruction
    /// request.
    pub fn retained_snapshot_for_commit(
        &self,
        expected_runtime_instance_id: u64,
        commit: &crate::history::data::RelationalCommitReceipt,
    ) -> Result<RelationalRetainedCommitSnapshot, RelationalRetainedCommitSnapshotDenial> {
        open_retained_commit_snapshot(self.runtime, expected_runtime_instance_id, commit)
    }

    /// Projects one entity directly from an exact retained canonical commit and reports the
    /// structural work performed by this owner operation.
    pub fn project_retained_entity_for_commit<T>(
        &self,
        expected_runtime_instance_id: u64,
        commit: &crate::history::data::RelationalCommitReceipt,
        retained_basis: &crate::history::retention::RelationalBranchRetentionLease,
        entity_id: crate::identity::data::EntityId,
        expected_kind_id: crate::identity::data::KindId,
        projection_scope: crate::visibility::materialization::read_records::ProjectionAspectScope,
        project: impl FnMut(
            crate::visibility::materialization::read_records::EntityProjectionRecord<'_>,
        ) -> Option<T>,
    ) -> Result<RelationalRetainedCommitEntityProjection<T>, RelationalRetainedCommitSnapshotDenial>
    {
        if expected_runtime_instance_id != self.runtime.runtime_instance_id() {
            return Err(RelationalRetainedCommitSnapshotDenial::new(
                RelationalRetainedCommitSnapshotDenialKind::ForeignRuntime,
                "retained commit basis belongs to a different runtime instance",
            ));
        }
        let basis = self
            .runtime
            .readmit_retained_branch_basis(retained_basis.descriptor(), retained_basis)
            .map_err(|denial| {
                let kind = match denial {
                    crate::branch::RelationalBranchBasisDenial::ForeignRuntime { .. } => {
                        RelationalRetainedCommitSnapshotDenialKind::ForeignRuntime
                    }
                    _ => RelationalRetainedCommitSnapshotDenialKind::SnapshotNotRetained,
                };
                RelationalRetainedCommitSnapshotDenial::new(
                    kind,
                    "retained commit basis could not be readmitted",
                )
            })?;
        let observation = basis.observation();
        if observation.commit_receipt() != Some(commit) {
            return Err(RelationalRetainedCommitSnapshotDenial::new(
                RelationalRetainedCommitSnapshotDenialKind::CommitMismatch,
                "retained basis does not select the requested canonical commit",
            ));
        }
        let snapshot = self.snapshot_for_observation(&observation).map_err(|denial| {
            let kind = match denial {
                crate::visibility::RelationalSnapshotAdmissionDenial::ForeignRuntime { .. } => {
                    RelationalRetainedCommitSnapshotDenialKind::ForeignRuntime
                }
                crate::visibility::RelationalSnapshotAdmissionDenial::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                } => {
                    RelationalRetainedCommitSnapshotDenialKind::ActiveSnapshotCapacityExhausted {
                        maximum_active_snapshots,
                    }
                }
                crate::visibility::RelationalSnapshotAdmissionDenial::SnapshotIdentityExhausted => {
                    RelationalRetainedCommitSnapshotDenialKind::SnapshotIdentityExhausted
                }
            };
            RelationalRetainedCommitSnapshotDenial::new(
                kind,
                "retained commit basis could not open an operation-local snapshot",
            )
        })?;
        let projected_fields = projection_scope
            .requirements()
            .iter()
            .map(|requirement| requirement.mask().paths().len())
            .sum();
        let projection = self
            .runtime
            .read_truth()
            .project_snapshot(&snapshot)
            .ok_or_else(projection_binding_denial)
            .and_then(|view| {
                view.entity_record_of_expected_kind_with_projection_scope(
                    entity_id,
                    expected_kind_id,
                    projection_scope,
                    project,
                )
                .map_err(|_| {
                    RelationalRetainedCommitSnapshotDenial::new(
                        RelationalRetainedCommitSnapshotDenialKind::EntityKindMismatch,
                        "retained entity kind differs from the requested owner projection",
                    )
                })
            });
        self.runtime
            .snapshots()
            .release_snapshot(&snapshot)
            .expect("retained-basis projection releases its operation-local snapshot once");
        let value = projection?;
        let work = crate::visibility::exact_commit_snapshot::RelationalRetainedCommitProjectionWork::opened_snapshot()
            .record_projection(usize::from(value.is_some()), projected_fields);
        Ok(RelationalRetainedCommitEntityProjection::new(value, work))
    }
}
