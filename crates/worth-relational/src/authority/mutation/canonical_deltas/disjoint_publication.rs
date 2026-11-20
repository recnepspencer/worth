//! One canonical record patch for actual disjoint coupled updates, never sequential netting.
use super::super::effect::MutationPublicationEffect;
use super::FoundationalPatchFragment as Fragment;
use crate::execution::request_stage::checkpoint;
use crate::identity::data::EntityId;
use crate::publication::patch::data::{
    PatchDetail, PublishedAuthoritativeAspectChange, RecordStructuralChange,
};
use crate::transactions::data::{
    CommitConflict, ConflictClass, ExistingRecordTarget, RecordRef, TransactionCommitError,
};
use std::collections::BTreeMap;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelContext};
use worth_foundational::facade::AuthoritativeRecordAspectPatch;
use worth_proof::TransitionOutcome;
type Group = Vec<(usize, Option<Fragment>)>;

/// Targets name the existing records whose command patches the Native composer
/// actually joined. Patch grammar and disjointness are independently checked here.
pub(crate) fn compose(
    context: &mut MapKernelContext<'_, '_>,
    execution: &ExecutionRequest<'_, '_>,
    effect: &mut MutationPublicationEffect,
    targets: &[EntityId],
) -> Result<(), TransactionCommitError> {
    let mut bytes = 0u64;
    for pair in targets.windows(2) {
        checkpoint(context, 1).map_err(stop)?;
        if pair[0] >= pair[1] {
            return Err(conflict(pair[0]));
        }
    }
    for fragment in &effect.patch_fragments {
        checkpoint(context, 1).map_err(stop)?;
        // Both original directories coexist until the canonical replacement.
        bytes = add(
            bytes,
            (size_of::<Fragment>() * 6 + size_of::<usize>() * 16) as u64,
        )?;
        if selected(fragment, targets) {
            bytes = add(
                bytes,
                fragment
                    .published_record_owned_allocation_capacity_bytes()
                    .checked_mul(4)
                    .ok_or_else(overflow)?,
            )?;
        }
    }
    let _memory = ExecutionMemoryReservation::reserve_in_scope(
        crate::authority::commit::pipeline::request_lease(execution),
        bytes,
    )
    .map_err(|cause| stop(crate::execution::denial::lease_denial(cause)))?;
    let original = std::mem::take(&mut effect.patch_fragments);
    let mut slots = Vec::new();
    slots
        .try_reserve_exact(original.len())
        .map_err(|_| allocation())?;
    let mut groups = BTreeMap::<EntityId, Group>::new();
    for fragment in original {
        checkpoint(context, 1).map_err(stop)?;
        if selected(&fragment, targets) {
            let RecordRef::Entity(entity) = fragment.target else {
                unreachable!()
            };
            groups
                .entry(entity)
                .or_default()
                .push((slots.len(), Some(fragment)));
            slots.push(None);
        } else {
            slots.push(Some(fragment));
        }
    }
    for mut group in groups.into_values() {
        let first = group[0].0;
        let fragment = if group.len() == 1 {
            group[0].1.take().expect("actual retained fragment")
        } else {
            join(&mut group, context)?.publish(context)?
        };
        slots[first] = Some(fragment);
    }
    effect.patch_fragments = slots.into_iter().flatten().collect();
    Ok(())
}
fn selected(fragment: &Fragment, targets: &[EntityId]) -> bool {
    matches!(fragment.target, RecordRef::Entity(entity) if targets.binary_search(&entity).is_ok())
}
struct Joined {
    target: EntityId,
    patch: AuthoritativeRecordAspectPatch,
    changes: Vec<PublishedAuthoritativeAspectChange>,
    detail: PatchDetail,
    opaque: bool,
}
fn join(
    group: &mut [(usize, Option<Fragment>)],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Joined, TransactionCommitError> {
    checkpoint(context, 1).map_err(stop)?;
    if group.len() == 1 {
        let fragment = group[0]
            .1
            .take()
            .expect("one transfer of each actual patch");
        let RecordRef::Entity(target) = fragment.target else {
            unreachable!()
        };
        if fragment.structural_change != RecordStructuralChange::Updated
            || fragment.relation_birth.is_some()
        {
            return Err(conflict(target));
        }
        return Ok(Joined {
            target,
            patch: fragment.patch,
            changes: fragment.semantic_changes,
            detail: fragment.detail,
            opaque: fragment.contains_opaque_aspect,
        });
    }
    // Balanced joins keep mask rebuilding logarithmic in the number of writers.
    let middle = group.len() / 2;
    let (left, right) = group.split_at_mut(middle);
    let left = join(left, context)?;
    let right = join(right, context)?;
    left.combine(right, context)
}
impl Joined {
    fn combine(
        mut self,
        right: Self,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Self, TransactionCommitError> {
        if self.target != right.target {
            return Err(conflict(self.target));
        }
        checkpoint(
            context,
            (self
                .patch
                .semantic_byte_width()
                .saturating_add(right.patch.semantic_byte_width()) as u64)
                .checked_add(1)
                .ok_or_else(overflow)?,
        )
        .map_err(stop)?;
        // Foundational combine intentionally supports whole-set succession.
        // This owner admits only disjoint updates, so repeated whole metadata
        // must be identical and set/clear pairs must never become last-wins.
        for (key, value) in right.patch.whole_aspect_sets() {
            if self.patch.whole_aspect_clears().any(|left| left == key)
                || self
                    .patch
                    .whole_aspect_sets()
                    .any(|(left, old)| left == key && old != value)
            {
                return Err(conflict(self.target));
            }
        }
        for key in right.patch.whole_aspect_clears() {
            if self.patch.whole_aspect_sets().any(|(left, _)| left == key) {
                return Err(conflict(self.target));
            }
        }
        self.patch = match AuthoritativeRecordAspectPatch::combine(self.patch, right.patch) {
            TransitionOutcome::Success(patch) => patch,
            TransitionOutcome::Denied(denial) => return Err(TransactionCommitError::conflict(
                super::data::CanonicalDeltaError::FoundationalPatchConstruction {
                    target: RecordRef::Entity(self.target),
                    denial: crate::transactions::data::AspectDeltaPatchConstructionDenial::FoundationalPatchConstructionDenied(denial),
                }.to_commit_conflict())),
        };
        self.changes.extend(right.changes);
        self.opaque |= right.opaque;
        let PatchDetail::DenseBitset(left) = &mut self.detail;
        let PatchDetail::DenseBitset(right_bits) = right.detail;
        left.resize(left.len().max(right_bits.len()), 0);
        for (left, right) in left.iter_mut().zip(right_bits) {
            *left |= right;
        }
        Ok(self)
    }
    fn publish(
        mut self,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Fragment, TransactionCommitError> {
        self.changes
            .sort_by_key(PublishedAuthoritativeAspectChange::canonical_key);
        self.changes.dedup();
        checkpoint(
            context,
            (self.patch.semantic_byte_width() as u64)
                .checked_add(1)
                .ok_or_else(overflow)?,
        )
        .map_err(stop)?;
        let published_patch =
            super::published_patch_projection::published_patch_from_foundational_patch(
                &self.patch,
                |key| {
                    let mut matching = self
                        .changes
                        .iter()
                        .filter(|change| change.aspect_key() == key);
                    let binding = matching.next()?.binding();
                    matching
                        .all(|change| change.binding() == binding)
                        .then(|| binding.clone())
                },
            )
            .map_err(|_| conflict(self.target))?;
        Ok(Fragment {
            target: RecordRef::Entity(self.target),
            structural_change: RecordStructuralChange::Updated,
            patch: self.patch,
            published_patch,
            semantic_changes: self.changes,
            contains_opaque_aspect: self.opaque,
            detail: self.detail,
            relation_birth: None,
        })
    }
}
fn conflict(entity: EntityId) -> TransactionCommitError {
    TransactionCommitError::conflict(CommitConflict::new(ConflictClass::ConflictingIntent {
        target: ExistingRecordTarget::Entity(entity),
    }))
}
fn stop(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
fn allocation() -> TransactionCommitError {
    stop(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
}
fn overflow() -> TransactionCommitError {
    stop(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
}
fn add(left: u64, right: u64) -> Result<u64, TransactionCommitError> {
    left.checked_add(right).ok_or_else(overflow)
}
