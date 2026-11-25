//! Value-free, bounded projection of one sealed publication candidate.

use contracts::history::data::coordinates::BranchId;
use contracts::history::data::coordinates::CommitId;
use contracts::identity::data::primitives::KindId;
use contracts::identity::data::primitives::VersionId;
use contracts::identity::data::record_ids::EntityId;
use contracts::identity::data::record_ids::RelationId;
use contracts::publication::patch::data::published_authoritative_patch::PublishedAuthoritativePatchOperation;
use contracts::publication::patch::data::published_authoritative_patch::PublishedAuthoritativePatchOwnerPort as _;
use contracts::publication::patch::data::record_structural_change::RecordStructuralChange;
use contracts::storage::data::read_records::RecordLifecycleState;
use contracts::transactions::data::primitives::RecordRef;
use worth_foundational::facade::{AspectKey, CanonicalFieldPath};
use worth_foundational::FoundationalBranchTarget;
use worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisDescriptor;
use worth_relational_contracts::internal_port as contracts;

/// Caller-selected hard bounds for the read-only candidate summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreparedRelationalChangeSummaryBudget {
    pub max_records: usize,
    pub max_aspect_scopes: usize,
}

/// A denial leaves the prepared candidate registered, unmodified, and discardable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedRelationalChangeSummaryDenial {
    StorageUnavailable(contracts::storage::data::storage_unavailable::RelationalStorageUnavailable),
    OwnerUnavailable,
    ForeignCandidate,
    CandidateUnavailable,
    BasisMismatch,
    CanonicalEnvelopeMismatch,
    RecordBudgetExceeded,
    AspectScopeBudgetExceeded,
    MissingLiveRelationEndpoints,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreparedRelationalRelationEndpoints {
    pub kind_id: KindId,
    pub source: EntityId,
    pub target: EntityId,
}

/// One exact changed aspect or field path, with no locator or body value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PreparedRelationalAspectScope {
    pub aspect_key: AspectKey,
    /// `None` denotes a whole-aspect set or clear.
    pub field_path: Option<CanonicalFieldPath>,
}

/// Exact canonical patch row with no authoritative body values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedRelationalRecordChange {
    pub target: RecordRef,
    pub structural_change: RecordStructuralChange,
    pub aspect_scopes: Vec<PreparedRelationalAspectScope>,
    pub before_endpoints: Option<PreparedRelationalRelationEndpoints>,
    pub after_endpoints: Option<PreparedRelationalRelationEndpoints>,
}

/// Exact owner and branch basis of a sealed, still-unpublished candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedRelationalChangeSummary {
    pub runtime_instance_id: u64,
    pub branch_id: BranchId,
    pub before_commit_id: Option<CommitId>,
    pub after_commit_id: CommitId,
    pub before_version: VersionId,
    pub after_version: VersionId,
    pub before_root_identity: u64,
    pub after_root_identity: u64,
    pub records: Vec<PreparedRelationalRecordChange>,
}

pub fn summarize(
    candidate: &crate::mvcc::publication::candidate::PreparedRelationalCommitCandidate,
    budget: PreparedRelationalChangeSummaryBudget,
) -> Result<PreparedRelationalChangeSummary, PreparedRelationalChangeSummaryDenial> {
    use PreparedRelationalChangeSummaryDenial as Denial;

    let (basis, before_root, after_root) = candidate
        .change_summary_basis_and_roots()
        .ok_or(Denial::CandidateUnavailable)?;
    let (before_commit_id, before_version) = before_basis(&basis);
    if basis.runtime_instance_id() != candidate.runtime_instance_id()
        || basis.branch_id() != candidate.branch()
        || basis.root_identity() != before_root.root().id()
    {
        return Err(Denial::BasisMismatch);
    }

    let envelope = after_root
        .canonical_envelope()
        .ok_or(Denial::CanonicalEnvelopeMismatch)?;
    if envelope.branch_context != *candidate.branch()
        || envelope.commit.branch_id != *candidate.branch()
        || envelope.merged_plan.transaction_id
            != contracts::branch::reference_checkpoint_data::CANONICAL_TRANSACTION_ID
        || after_root.commit_id() != Some(envelope.commit.commit_id)
        || after_root.axes().map(|axes| axes.storage_version) != Some(envelope.commit.version_id.0)
    {
        return Err(Denial::CanonicalEnvelopeMismatch);
    }
    let patches = &envelope.patch.authoritative_record_patches;
    if patches.len() > budget.max_records {
        return Err(Denial::RecordBudgetExceeded);
    }
    let mut aspect_scope_count = 0usize;
    for patch in patches {
        for operation in patch.authoritative_patch.full_grammar_operations() {
            let added = match operation {
                PublishedAuthoritativePatchOperation::WholeAspectSet { .. }
                | PublishedAuthoritativePatchOperation::WholeAspectClear { .. } => 1,
                PublishedAuthoritativePatchOperation::FieldLevelPatch {
                    field_sets,
                    field_clears,
                    ..
                } => field_sets
                    .len()
                    .checked_add(field_clears.len())
                    .ok_or(Denial::AspectScopeBudgetExceeded)?,
            };
            aspect_scope_count = aspect_scope_count
                .checked_add(added)
                .ok_or(Denial::AspectScopeBudgetExceeded)?;
            if aspect_scope_count > budget.max_aspect_scopes {
                return Err(Denial::AspectScopeBudgetExceeded);
            }
        }
    }

    let mut records = Vec::with_capacity(patches.len());
    for patch in patches {
        let (before_endpoints, after_endpoints) = match patch.target {
            RecordRef::Entity(_) => (None, None),
            RecordRef::Relation(relation_id) => (
                live_relation_endpoints(&before_root.read(), relation_id)?,
                live_relation_endpoints(&after_root.read(), relation_id)?,
            ),
        };
        records.push(PreparedRelationalRecordChange {
            target: patch.target.clone(),
            structural_change: patch.structural_change,
            aspect_scopes: aspect_scopes(patch),
            before_endpoints,
            after_endpoints,
        });
    }

    Ok(PreparedRelationalChangeSummary {
        runtime_instance_id: candidate.runtime_instance_id(),
        branch_id: candidate.branch().clone(),
        before_commit_id,
        after_commit_id: envelope.commit.commit_id,
        before_version,
        after_version: envelope.commit.version_id,
        before_root_identity: before_root.root().id(),
        after_root_identity: after_root.id(),
        records,
    })
}

fn aspect_scopes(
    patch: &contracts::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativeRecordPatch,
) -> Vec<PreparedRelationalAspectScope> {
    let mut scopes = Vec::new();
    for operation in patch.authoritative_patch.full_grammar_operations() {
        match operation {
            PublishedAuthoritativePatchOperation::WholeAspectSet { aspect_key, .. }
            | PublishedAuthoritativePatchOperation::WholeAspectClear { aspect_key, .. } => {
                scopes.push(PreparedRelationalAspectScope {
                    aspect_key: aspect_key.clone(),
                    field_path: None,
                });
            }
            PublishedAuthoritativePatchOperation::FieldLevelPatch {
                aspect_key,
                field_sets,
                field_clears,
                ..
            } => {
                for field in field_sets.iter().map(|set| &set.field).chain(field_clears) {
                    scopes.push(PreparedRelationalAspectScope {
                        aspect_key: aspect_key.clone(),
                        field_path: Some(CanonicalFieldPath::single(field.clone())),
                    });
                }
            }
        }
    }
    scopes.sort();
    scopes.dedup();
    scopes
}

fn before_basis(basis: &RelationalBranchBasisDescriptor) -> (Option<CommitId>, VersionId) {
    match basis.reference().target() {
        FoundationalBranchTarget::Empty => (None, VersionId(0)),
        FoundationalBranchTarget::Basis(target) => (
            Some(CommitId(target.selected_commit_id())),
            VersionId(target.version_id()),
        ),
    }
}

fn live_relation_endpoints(
    root: &worth_relational_branch::internal_port::branch::root::RelationalRootRead<'_>,
    relation_id: RelationId,
) -> Result<Option<PreparedRelationalRelationEndpoints>, PreparedRelationalChangeSummaryDenial> {
    use PreparedRelationalChangeSummaryDenial::MissingLiveRelationEndpoints;

    let Some(partition) = root
        .partition_state(relation_id.partition_id)
        .map_err(PreparedRelationalChangeSummaryDenial::StorageUnavailable)?
    else {
        return Ok(None);
    };
    let Some(slot) = partition.relation_arena.get(&relation_id) else {
        return Ok(None);
    };
    if !matches!(
        slot.lifecycle(),
        RecordLifecycleState::Live | RecordLifecycleState::MaterializationUnavailable
    ) || slot.retired_at().is_some()
    {
        return Ok(None);
    }
    let endpoints = slot.extra().endpoints.as_ref().or_else(|| {
        partition
            .relation_arena
            .metadata_history_at(relation_id.slot_index())?
            .iter()
            .rev()
            .find(|entry| {
                entry.generation == relation_id.generation_value() && entry.retired_at.is_none()
            })
            .map(|entry| &entry.endpoints)
    });
    let endpoints = endpoints.ok_or(MissingLiveRelationEndpoints)?;
    let kind_id = slot.kind_id().ok_or(MissingLiveRelationEndpoints)?;
    Ok(Some(PreparedRelationalRelationEndpoints {
        kind_id,
        source: endpoints.source,
        target: endpoints.target,
    }))
}
