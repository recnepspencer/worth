mod basis;
mod changes;
mod entry_edits;
mod field;
mod join;
mod ordering;
mod reads;
mod work;

use super::{publish_prepared_generation, IndexAuthority, IndexGenerationPublicationBasis};
use crate::branch::AdmittedRelationalBranchBasis;
use crate::indexes::data::*;
use crate::runtime::VisibilityProjectionView;
use crate::snapshots::data::SnapshotHandle;
use work::MaintenanceWork;

impl IndexAuthority<'_> {
    /// Explicit bounded reconstruction for a committed version that is no
    /// longer a live branch head. Retention selects its actual immutable root;
    /// the root's envelope must match the requested commit and branch.
    pub fn reconstruct_for_commit(
        &self,
        request: DerivedIndexBuildRequest,
        budget: DerivedIndexMaintenanceBudget,
    ) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceDenial> {
        let mut work = MaintenanceWork::new(budget);
        let result = (|| {
            work.charge(request.index_ids.len())?;
            let recorded = self
                .runtime
                .history
                .recorded_commit_envelope(request.source_commit_id)
                .ok_or(DerivedIndexMaintenanceDenialKind::CommitMismatch)?;
            if recorded.branch_context != request.branch_id {
                return Err(DerivedIndexMaintenanceDenialKind::CommitMismatch);
            }
            let projection = self
                .runtime
                .read_truth()
                .try_project_retained_commit(
                    request.source_commit_id,
                    request.branch_id.clone(),
                    recorded.commit.version_id,
                )
                .map_err(DerivedIndexMaintenanceDenialKind::Basis)?;
            let root = projection
                .selected_root()
                .ok_or(DerivedIndexMaintenanceDenialKind::CommitMismatch)?;
            if root.commit_id() != Some(request.source_commit_id)
                || root
                    .canonical_envelope()
                    .is_none_or(|envelope| envelope.branch_context != request.branch_id)
            {
                return Err(DerivedIndexMaintenanceDenialKind::CommitMismatch);
            }
            self.prepare_refresh(&request, None, &projection, &mut work)
        })();
        result
            .map(|generations| DerivedIndexMaintenanceOutcome {
                generations,
                work: work.counts,
            })
            .map_err(|kind| work.deny(kind))
    }

    /// Refresh one exact published basis. The optional before snapshot enables
    /// patch-local maintenance only when the canonical pre-commit root matches.
    /// Missing generations/snapshots require the explicit cold budget. All work
    /// is prepared before any generation is published; denial publishes none.
    pub fn refresh_for_basis(
        &self,
        request: DerivedIndexBuildRequest,
        basis: &AdmittedRelationalBranchBasis,
        before: Option<&SnapshotHandle>,
        budget: DerivedIndexMaintenanceBudget,
    ) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceDenial> {
        let mut work = MaintenanceWork::new(budget);
        let result = (|| {
            work.charge(request.index_ids.len())?;
            let after = self
                .runtime
                .read_truth()
                .project_observation(&basis.observation())
                .map_err(DerivedIndexMaintenanceDenialKind::Basis)?;
            let before = before
                .map(|snapshot| {
                    if snapshot.branch_id() != &request.branch_id {
                        return Err(DerivedIndexMaintenanceDenialKind::BeforeRootMismatch);
                    }
                    self.runtime
                        .read_truth()
                        .project_snapshot(snapshot)
                        .ok_or(DerivedIndexMaintenanceDenialKind::SnapshotUnavailable)
                })
                .transpose()?;
            basis::validate(&request, &basis.observation(), before.as_ref(), &after)?;
            self.prepare_refresh(&request, before.as_ref(), &after, &mut work)
        })();
        result
            .map(|generations| DerivedIndexMaintenanceOutcome {
                generations,
                work: work.counts,
            })
            .map_err(|kind| work.deny(kind))
    }

    fn prepare_refresh(
        &self,
        request: &DerivedIndexBuildRequest,
        before: Option<&VisibilityProjectionView<'_>>,
        after: &VisibilityProjectionView<'_>,
        work: &mut MaintenanceWork,
    ) -> Result<Vec<DerivedIndexGeneration>, DerivedIndexMaintenanceDenialKind> {
        let root = after.selected_root().expect("validated exact root");
        let schema_version = root.schema_authority().schema_version();
        let mut prepared = Vec::new();
        let mut pending_fields = Vec::new();
        let mut reused = Vec::new();
        let mut cold_changes = None;
        let mut patch_changes = None;
        for index_id in request
            .index_ids
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
        {
            let definition = self.runtime.indexes.definition(index_id).ok_or(
                DerivedIndexMaintenanceDenialKind::IndexUnavailable(index_id),
            )?;
            if let Some(generation) = self.runtime.indexes.published_generation_for_commit(
                index_id,
                Some(&request.branch_id),
                request.source_commit_id,
                after.version_id(),
            ) {
                if generation.applicability.schema_version == schema_version {
                    reused.push(generation.as_ref().clone());
                    work.counts.reused_generations += 1;
                    continue;
                }
            }
            let prior = before
                .filter(|view| {
                    view.selected_schema_authority().map(|s| s.schema_version())
                        == Some(schema_version)
                })
                .and_then(|view| {
                    self.runtime
                        .indexes
                        .exact_generation(
                            index_id,
                            Some(&request.branch_id),
                            view.version_id(),
                            schema_version,
                        )
                        .filter(|generation| {
                            view.selected_root().and_then(|root| root.commit_id())
                                == Some(generation.source_commit_id)
                        })
                });
            let (entries, changes, old) = if let Some(prior) = prior {
                if patch_changes.is_none() {
                    patch_changes = Some(changes::ChangedRecords::patch(root, work)?);
                }
                (
                    prior.entries.clone(),
                    patch_changes.as_ref().unwrap(),
                    before,
                )
            } else {
                if cold_changes.is_none() {
                    cold_changes = Some(changes::ChangedRecords::cold(after, work)?);
                }
                (
                    empty_entries(&definition.kind),
                    cold_changes.as_ref().unwrap(),
                    None,
                )
            };
            match (&definition.kind, entries) {
                (
                    DerivedIndexKind::EntityField { field_locator },
                    DerivedIndexEntries::EntityField(entries),
                ) => pending_fields.push(field::PendingField::Entity {
                    index_id,
                    locator: field_locator.clone(),
                    entries,
                    patch: old.is_some(),
                }),
                (
                    DerivedIndexKind::RelationField { field_locator },
                    DerivedIndexEntries::RelationField(entries),
                ) => pending_fields.push(field::PendingField::Relation {
                    index_id,
                    locator: field_locator.clone(),
                    entries,
                    patch: old.is_some(),
                }),
                (_, mut entries) => {
                    update_entries(&definition, &mut entries, changes, old, after, work)?;
                    prepared.push((index_id, entries));
                }
            }
        }
        field::refresh(
            &mut pending_fields,
            patch_changes.as_ref(),
            cold_changes.as_ref(),
            before,
            after,
            work,
        )?;
        prepared.extend(
            pending_fields
                .into_iter()
                .map(field::PendingField::into_prepared),
        );
        let publication = IndexGenerationPublicationBasis::new(
            request,
            request.branch_id.clone(),
            after.version_id(),
            schema_version,
        );
        reused.extend(prepared.into_iter().map(|(index_id, entries)| {
            publish_prepared_generation(self.runtime, &publication, index_id, entries)
        }));
        reused.sort_by_key(|generation| generation.index_id);
        Ok(reused)
    }
}

fn empty_entries(kind: &DerivedIndexKind) -> DerivedIndexEntries {
    match kind {
        DerivedIndexKind::EntityField { .. } => {
            DerivedIndexEntries::EntityField(Default::default())
        }
        DerivedIndexKind::RelationField { .. } => {
            DerivedIndexEntries::RelationField(Default::default())
        }
        DerivedIndexKind::RelatedEntityOrdering { .. } => {
            DerivedIndexEntries::RelatedEntityOrdering(Default::default())
        }
        DerivedIndexKind::RelationJoin(_) => DerivedIndexEntries::RelationJoin(Default::default()),
    }
}

fn update_entries(
    definition: &DerivedIndexDefinition,
    entries: &mut DerivedIndexEntries,
    changes: &changes::ChangedRecords,
    before: Option<&VisibilityProjectionView<'_>>,
    after: &VisibilityProjectionView<'_>,
    work: &mut MaintenanceWork,
) -> Result<(), DerivedIndexMaintenanceDenialKind> {
    match (&definition.kind, entries) {
        (
            DerivedIndexKind::RelatedEntityOrdering {
                relation_kind,
                parent_endpoint,
                child_kind,
                ordering,
            },
            DerivedIndexEntries::RelatedEntityOrdering(entries),
        ) => ordering::refresh(
            entries,
            (*relation_kind, *parent_endpoint, *child_kind, ordering),
            changes,
            before,
            after,
            work,
        ),
        (DerivedIndexKind::RelationJoin(join), DerivedIndexEntries::RelationJoin(entries)) => {
            join::refresh(entries, *join, changes, before, after, work)
        }
        _ => Err(DerivedIndexMaintenanceDenialKind::GenerationKindMismatch(
            definition.index_id,
        )),
    }
}
