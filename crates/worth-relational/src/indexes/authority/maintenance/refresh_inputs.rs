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
            .commit_evidence()
            .map(|evidence| evidence.authoring_branch());
        let mut patch_changes = None;
        let mut basis = if let Some(before) = before {
            let previous = before.root();
            if let Some(receipt) = previous.commit_receipt() {
                let authoring = previous
                    .commit_evidence()
                    .ok_or(DerivedIndexMaintenanceDenialKind::CommitMismatch)?
                    .authoring_branch();
                self.indexes
                    .prepare_basis_selection(
                        &request.branch_id,
                        request.source_commit_id,
                        after.version_id(),
                        schema_version,
                        &request.branch_id,
                        receipt.commit_id,
                        before.version_id(),
                        &mut |units, bytes| work.prepare(units, bytes),
                    )
                    .or_else(|stop| {
                        if stop == DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired
                            && authoring != &request.branch_id
                        {
                            self.indexes.prepare_basis_selection(
                                &request.branch_id,
                                request.source_commit_id,
                                after.version_id(),
                                schema_version,
                                authoring,
                                receipt.commit_id,
                                before.version_id(),
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
                if previous.id() != 0 || previous.commit_id().is_some() || previous.axes().is_some()
                {
                    return Err(DerivedIndexMaintenanceDenialKind::BeforeRootMismatch);
                }
                Some(self.indexes.prepare_empty_basis_selection(
                    &request.branch_id,
                    request.source_commit_id,
                    after.version_id(),
                    schema_version,
                    &mut |units, bytes| work.prepare(units, bytes),
                    self.prepared_catalog(),
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
                self.routes(after.root())?.as_ref(),
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
        let mut requested = if work.routes_installed_indexes() {
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

        if let Some(definitions) = self.definitions {
            for index in definitions.introduced_ids() {
                work.ordered::<DerivedIndexId, ()>(requested.len(), 1, 0)?;
                requested.insert(index);
            }
        }
        Ok(RootMaintenanceInputs {
            basis,
            inherited,
            patch_changes,
            requested,
        })
    }
}
