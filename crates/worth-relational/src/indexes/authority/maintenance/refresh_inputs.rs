//! Select only dependent maintainers and carry the actual predecessor selection.
use super::*;
pub(super) struct RootMaintenanceInputs {
    pub(super) basis: Option<crate::runtime::PreparedIndexBasisSelection>,
    pub(super) inherited: Option<crate::runtime::PreparedIndexBasisSelection>,
    pub(super) patch_changes: Option<changes::ChangedRecords>,
    pub(super) requested: std::collections::BTreeSet<DerivedIndexId>,
}
impl super::owner::IndexMaintenanceOwner<'_> {
    pub(super) fn refresh_inputs(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &DerivedIndexBuildRequest,
        before: Option<&root_view::IndexRootView>,
        after: &root_view::IndexRootView,
        work: &mut MaintenanceWork,
    ) -> Result<RootMaintenanceInputs, DerivedIndexMaintenanceDenialKind> {
        let root = after.root();
        let schema_version = root.schema_authority().schema_version();
        let authoring = root
            .canonical_envelope()
            .map(|envelope| &envelope.branch_context);
        let mut patch_changes = None;
        let mut basis = if let Some(before) = before {
            let previous = before.root();
            if let Some(envelope) = previous.canonical_envelope() {
                self.indexes
                    .prepare_basis_selection(
                        &request.branch_id,
                        request.source_commit_id,
                        after.version_id(),
                        schema_version,
                        &request.branch_id,
                        envelope.commit.commit_id,
                        before.version_id(),
                        work.publication_is_deferred(),
                        &mut |units, bytes| work.prepare(units, bytes),
                    )
                    .or_else(|stop| {
                        if stop == DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired
                            && envelope.branch_context != request.branch_id
                        {
                            self.indexes.prepare_basis_selection(
                                &request.branch_id,
                                request.source_commit_id,
                                after.version_id(),
                                schema_version,
                                &envelope.branch_context,
                                envelope.commit.commit_id,
                                before.version_id(),
                                work.publication_is_deferred(),
                                &mut |units, bytes| work.prepare(units, bytes),
                            )
                        } else {
                            Err(stop)
                        }
                    })
                    .map(Some)
                    .or_else(|stop| {
                        if stop == DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired {
                            Ok(None)
                        } else {
                            Err(stop)
                        }
                    })?
            } else {
                Some(self.indexes.prepare_empty_basis_selection(
                    &request.branch_id,
                    request.source_commit_id,
                    after.version_id(),
                    schema_version,
                    work.publication_is_deferred(),
                    &mut |units, bytes| work.prepare(units, bytes),
                )?)
            }
        } else if let Some(authoring) = authoring.filter(|branch| *branch != &request.branch_id) {
            self.indexes
                .prepare_basis_selection(
                    &request.branch_id,
                    request.source_commit_id,
                    after.version_id(),
                    schema_version,
                    authoring,
                    request.source_commit_id,
                    after.version_id(),
                    work.publication_is_deferred(),
                    &mut |units, bytes| work.prepare(units, bytes),
                )
                .map(Some)
                .or_else(|stop| {
                    if stop == DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired {
                        Ok(None)
                    } else {
                        Err(stop)
                    }
                })?
        } else {
            self.indexes
                .prepare_basis_selection(
                    &request.branch_id,
                    request.source_commit_id,
                    after.version_id(),
                    schema_version,
                    &request.branch_id,
                    request.source_commit_id,
                    after.version_id(),
                    work.publication_is_deferred(),
                    &mut |units, bytes| work.prepare(units, bytes),
                )
                .map(Some)
                .or_else(|stop| {
                    if stop == DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired {
                        Ok(None)
                    } else {
                        Err(stop)
                    }
                })?
        };
        let affected = if let Some(before) = before {
            let change = root
                .commit_change()
                .ok_or(DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired)?;
            let changes = changes::ChangedRecords::patch(change, work)?;
            let affected = affected_indexes::select(
                self.indexes,
                change,
                execution,
                &changes,
                before,
                after,
                work,
            )?;
            patch_changes = Some(changes);
            Some(affected)
        } else {
            None
        };
        let inherited = basis.clone();
        if let (Some(basis), Some(affected)) = (&mut basis, &affected) {
            for index in affected {
                basis.remove_admitted(*index, &mut |units, bytes| work.prepare(units, bytes))?;
            }
        }
        let requested = if work.routes_installed_indexes() {
            if basis.is_none() {
                return Err(DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired);
            }
            affected.ok_or(DerivedIndexMaintenanceDenialKind::BeforeRootMismatch)?
        } else {
            let mut requested = std::collections::BTreeSet::new();
            for index_id in &request.index_ids {
                work.ordered::<DerivedIndexId, ()>(requested.len(), 1, 0)?;
                requested.insert(*index_id);
            }
            requested
        };

        Ok(RootMaintenanceInputs {
            basis,
            inherited,
            patch_changes,
            requested,
        })
    }
}
