//! A merge writes unique values under the same law as a program: before the
//! merge commits, every unique value it writes is looked up in the field's
//! equality index at the target head, and must be free or held only by the
//! entity the merge writes. A merge writes each unique value at most once.
//! The lookups are proportional to the merge's own writes: one per unique
//! value it writes, and nothing else.

use std::collections::BTreeSet;
#[cfg(test)]
thread_local! { static LOOKUPS: std::cell::RefCell<Vec<usize>> = const { std::cell::RefCell::new(Vec::new()) }; }
#[cfg(test)]
pub(super) fn take_lookup_limits() -> Vec<usize> {
    LOOKUPS.with(|calls| std::mem::take(&mut *calls.borrow_mut()))
}

use worth_foundational::facade::{
    prepare_aspect_value_identity_basis, AspectFieldLocator, CanonicalAspectValueIdentityBasis,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::merge::PreparedMergeExecution;
use worth_relational::facade::runtime::{ProjectionAspectScope, RelationalRuntime};
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_relational::facade::storage::RecordLifecycleState;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, EntityMutationIntent, MutationIntent,
};

use super::application_attempt::{observe_indexed_candidates, WorthQueryIndexedSelectionRefusal};
use super::schema_layout::{WorthQueryUniqueFieldIndex, WorthQueryUniqueFields};
use super::WorthQueryPrimaryGraphIntegrationHandle;

/// Two candidates tell a free value from a taken one, and a held value from
/// a duplicated one.
const MERGE_UNIQUE_CANDIDATE_LIMIT: usize = 2;

/// Why a merge's unique values refused it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryMergeUniqueValueDenialKind {
    /// Another live entity at the target head holds a value the merge
    /// writes, or the merge writes one value twice.
    ValueTaken,
    /// A unique field's equality index could not answer at the target head.
    IndexUnavailable,
    /// The target head could not be opened or projected.
    TargetHeadUnavailable,
    /// The merge updates an entity that is not live at the target head.
    MergedEntityNotLive,
    /// The merge writes an entity in a shape this lookup has no reading for.
    /// The merge derives entity writes only as creates and field updates, so
    /// any other shape is refused rather than passed as free.
    UnreadWriteShape,
}

/// A merge refused by the unique values it writes; `field` names the field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryMergeUniqueValueDenial {
    kind: WorthQueryMergeUniqueValueDenialKind,
    field: String,
}

impl WorthQueryMergeUniqueValueDenial {
    pub const fn kind(&self) -> WorthQueryMergeUniqueValueDenialKind {
        self.kind
    }

    pub fn field(&self) -> &str {
        &self.field
    }
}

impl WorthQueryPrimaryGraphIntegrationHandle {
    /// Admits the unique values `prepared` writes, read at its target head.
    /// Call between preparing and executing the merge, on one runtime
    /// borrow, so the head read is the head the merge was planned against.
    #[doc(hidden)]
    pub fn admit_merge_unique_values(
        &self,
        runtime: &RelationalRuntime,
        prepared: &PreparedMergeExecution,
    ) -> Result<(), WorthQueryMergeUniqueValueDenial> {
        admit_merge_unique_values(self.layout.unique_fields(), runtime, prepared)
    }
}

pub(in crate::domain_computation::primary_graph) fn admit_merge_unique_values(
    unique: WorthQueryUniqueFields<'_>,
    runtime: &RelationalRuntime,
    prepared: &PreparedMergeExecution,
) -> Result<(), WorthQueryMergeUniqueValueDenial> {
    if unique.is_empty() {
        return Ok(());
    }
    let mut head = TargetHead {
        runtime,
        branch: prepared.request().target_branch(),
        snapshot: None,
    };
    let mut written = BTreeSet::<(
        KindId,
        AspectFieldLocator,
        CanonicalAspectValueIdentityBasis,
    )>::new();
    for intent in prepared.merged_intents() {
        let (writer, kind, fields) = match intent {
            MutationIntent::Create(CreateIntent::Entity(spec)) => {
                (None, spec.kind_id, &spec.fields)
            }
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(update)) => {
                let kind = head.entity_kind(update.entity_id)?;
                (Some(update.entity_id), kind, &update.fields)
            }
            // Relations carry no unique field; deletes and revalidations write
            // no value.
            MutationIntent::Create(
                CreateIntent::Relation(_)
                | CreateIntent::RelationAspects(_)
                | CreateIntent::BulkRelations(_),
            )
            | MutationIntent::Relation(_)
            | MutationIntent::Entity(
                EntityMutationIntent::Delete(_) | EntityMutationIntent::Revalidate(_),
            ) => continue,
            MutationIntent::Create(
                CreateIntent::EntityAspects(_) | CreateIntent::BulkEntities(_),
            )
            | MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(_) | EntityMutationIntent::Replace(_),
            )
            | MutationIntent::Materialization(_) => {
                return Err(denial(
                    WorthQueryMergeUniqueValueDenialKind::UnreadWriteShape,
                    "merge write shape",
                ))
            }
        };
        admit_fields(unique, &mut head, &mut written, writer, kind, fields)?;
    }
    Ok(())
}

fn admit_fields(
    unique: WorthQueryUniqueFields<'_>,
    head: &mut TargetHead<'_>,
    written: &mut BTreeSet<(
        KindId,
        AspectFieldLocator,
        CanonicalAspectValueIdentityBasis,
    )>,
    writer: Option<EntityId>,
    kind: KindId,
    fields: &AspectFieldPatch,
) -> Result<(), WorthQueryMergeUniqueValueDenial> {
    for (locator, value) in fields.iter() {
        let index_id = match unique.index(kind, locator) {
            None => continue,
            Some(WorthQueryUniqueFieldIndex::Unavailable) => {
                return Err(field_denial(
                    WorthQueryMergeUniqueValueDenialKind::IndexUnavailable,
                    locator,
                ))
            }
            Some(WorthQueryUniqueFieldIndex::Installed(index_id)) => index_id,
        };
        if !written.insert((
            kind,
            locator.clone(),
            prepare_aspect_value_identity_basis(value),
        )) {
            return Err(field_denial(
                WorthQueryMergeUniqueValueDenialKind::ValueTaken,
                locator,
            ));
        }
        let runtime = head.runtime;
        let snapshot = head.snapshot()?;
        #[cfg(test)]
        LOOKUPS.with(|calls| calls.borrow_mut().push(MERGE_UNIQUE_CANDIDATE_LIMIT));
        let selection = observe_indexed_candidates(
            runtime,
            snapshot,
            index_id,
            kind,
            locator,
            value,
            MERGE_UNIQUE_CANDIDATE_LIMIT,
        );
        let candidates = match selection {
            Ok(candidates) => candidates,
            // More holders than the limit: the value is held.
            Err(WorthQueryIndexedSelectionRefusal::Overflowed) => {
                return Err(field_denial(
                    WorthQueryMergeUniqueValueDenialKind::ValueTaken,
                    locator,
                ))
            }
            Err(WorthQueryIndexedSelectionRefusal::Unavailable) => {
                return Err(field_denial(
                    WorthQueryMergeUniqueValueDenialKind::IndexUnavailable,
                    locator,
                ))
            }
        };
        if candidates
            .iter()
            .any(|candidate| Some(*candidate) != writer)
        {
            return Err(field_denial(
                WorthQueryMergeUniqueValueDenialKind::ValueTaken,
                locator,
            ));
        }
    }
    Ok(())
}

/// The target branch's head, opened on first use and closed on drop.
struct TargetHead<'runtime> {
    runtime: &'runtime RelationalRuntime,
    branch: &'runtime BranchId,
    snapshot: Option<SnapshotHandle>,
}

impl TargetHead<'_> {
    fn snapshot(&mut self) -> Result<&SnapshotHandle, WorthQueryMergeUniqueValueDenial> {
        if self.snapshot.is_none() {
            let runtime = self.runtime;
            let opened = runtime
                .branch_identity(self.branch)
                .ok()
                .and_then(|identity| runtime.observe_branch(&identity).ok())
                .and_then(|(_, basis)| {
                    runtime
                        .snapshots()
                        .snapshot_for_observation(&basis.observation())
                        .ok()
                })
                .ok_or_else(target_head_unavailable)?;
            self.snapshot = Some(opened);
        }
        Ok(self
            .snapshot
            .as_ref()
            .expect("the target head snapshot was opened above"))
    }

    fn entity_kind(
        &mut self,
        entity_id: EntityId,
    ) -> Result<KindId, WorthQueryMergeUniqueValueDenial> {
        let runtime = self.runtime;
        let snapshot = self.snapshot()?;
        let truth = runtime.read_truth();
        let view = truth
            .project_snapshot(snapshot)
            .ok_or_else(target_head_unavailable)?;
        view.entity_record_with_projection_scope(
            entity_id,
            ProjectionAspectScope::empty(),
            |record| (record.lifecycle() == RecordLifecycleState::Live).then(|| record.kind_id()),
        )
        .ok_or_else(|| {
            denial(
                WorthQueryMergeUniqueValueDenialKind::MergedEntityNotLive,
                format!("{entity_id:?}"),
            )
        })
    }
}

impl Drop for TargetHead<'_> {
    fn drop(&mut self) {
        if let Some(snapshot) = self.snapshot.take() {
            self.runtime
                .snapshots()
                .release_snapshot(&snapshot)
                .expect("the merge's target head snapshot closes exactly once");
        }
    }
}

fn target_head_unavailable() -> WorthQueryMergeUniqueValueDenial {
    denial(
        WorthQueryMergeUniqueValueDenialKind::TargetHeadUnavailable,
        "target head",
    )
}

fn field_denial(
    kind: WorthQueryMergeUniqueValueDenialKind,
    locator: &AspectFieldLocator,
) -> WorthQueryMergeUniqueValueDenial {
    denial(kind, format!("{locator:?}"))
}

fn denial(
    kind: WorthQueryMergeUniqueValueDenialKind,
    field: impl Into<String>,
) -> WorthQueryMergeUniqueValueDenial {
    WorthQueryMergeUniqueValueDenial {
        kind,
        field: field.into(),
    }
}
