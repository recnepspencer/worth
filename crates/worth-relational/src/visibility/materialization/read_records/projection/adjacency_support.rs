//! Complete live membership at two exact roots, independent of revision numbers.
use super::{RelationalAdjacencyDirection as Direction, VisibilityProjectionView};
use crate::execution::request_stage::{checkpoint, optional_stage};
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::identity::data::{EntityId, KindId};
use crate::schema::data::RelationalSchemaRegistry;
use worth_execution::{ExecutionRequest, MapKernelContext};

mod membership;

#[derive(Clone, Debug, PartialEq, Eq)]
/// A refusal is not an empty adjacency population or evidence of cancellation.
pub enum RelationalAdjacencySupportDenial {
    ForeignOwner,
    ExactBasisRequired,
    EntityMismatch,
    IncompatibleMeaning,
    SourceUnavailable,
    UnsupportedVisibility,
    Execution(Cause),
    Allocation(worth_execution::ExecutionAllocationDenial),
}
use RelationalAdjacencySupportDenial as Denial;

impl VisibilityProjectionView {
    /// Compare complete live relation identity, multiplicity and both full-GEN
    /// endpoints. Equal tuples preserve the ordinary deterministic observation
    /// order (source, target, relation), not merely merged endpoint coverage.
    /// Relation values require their own field facts. Both exact roots and the
    /// installed relation/endpoint meaning must agree. This proves no ancestry,
    /// canonical cancellation, current permission or ordinary revision reuse.
    /// Work follows the two selected buckets and their actual regions; only
    /// fixed metadata is copied under the original request, never rich records.
    pub fn compare_adjacency_support(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        target: &Self,
        anchor: EntityId,
        kind: KindId,
        direction: Direction,
    ) -> Result<bool, Denial> {
        optional_stage(
            execution,
            |context, _memory| {
                consult(execution)?;
                if self.runtime_identity != target.runtime_identity {
                    return Err(Denial::ForeignOwner);
                }
                let before = registry(self)?
                    .relation_registration(kind)
                    .map_err(|_| Denial::IncompatibleMeaning)?;
                let after = registry(target)?
                    .relation_registration(kind)
                    .map_err(|_| Denial::IncompatibleMeaning)?;
                if before != after {
                    return Err(Denial::IncompatibleMeaning);
                }
                entity_pair(self, target, execution, context, anchor)?;
                let original = membership::read(self, execution, anchor, kind, direction, context)?;
                let current =
                    membership::read(target, execution, anchor, kind, direction, context)?;
                // The substrate supplies unique relation IDs in deterministic order.
                // Preserve identities and cardinality, including parallel edges.
                let mut left = original.iter().flatten();
                let mut right = current.iter().flatten();
                loop {
                    admit(context, 1)?;
                    match (left.next(), right.next()) {
                        (None, None) => return Ok(true),
                        (Some(left), Some(right)) if left == right => {
                            entity_pair(self, target, execution, context, left.source)?;
                            entity_pair(self, target, execution, context, left.target)?;
                        }
                        _ => return Ok(false),
                    }
                }
            },
            |_| 0,
        )
        .map_err(Denial::Execution)?
    }
}

fn registry(view: &VisibilityProjectionView) -> Result<&RelationalSchemaRegistry, Denial> {
    if !view.is_exact_basis() {
        return Err(Denial::ExactBasisRequired);
    }
    Ok(view
        .basis
        .root()
        .ok_or(Denial::SourceUnavailable)?
        .schema_authority()
        .registry())
}

fn entity_pair(
    original: &VisibilityProjectionView,
    target: &VisibilityProjectionView,
    execution: &ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    entity: EntityId,
) -> Result<(), Denial> {
    if entity.generation_value() == 0 {
        return Err(Denial::EntityMismatch);
    }
    let mut read = |view: &VisibilityProjectionView| {
        admit(
            context,
            view.exact_entity_state_read_work_bound()
                .ok_or(Denial::ExactBasisRequired)?,
        )?;
        view.read_exact_entity_state(execution, entity, |metadata, _| metadata)
            .map_err(borrowed_denial)?
            .ok_or(Denial::EntityMismatch)
    };
    let before = read(original)?;
    let after = read(target)?;
    if before != after || before.lifecycle != crate::storage::data::RecordLifecycleState::Live {
        return Err(Denial::EntityMismatch);
    }
    let before = registry(original)?
        .entity_registration(before.kind_id)
        .map_err(|_| Denial::IncompatibleMeaning)?;
    let after = registry(target)?
        .entity_registration(after.kind_id)
        .map_err(|_| Denial::IncompatibleMeaning)?;
    if before != after {
        return Err(Denial::IncompatibleMeaning);
    }
    Ok(())
}

fn consult(execution: &ExecutionRequest<'_, '_>) -> Result<(), Denial> {
    execution
        .consult()
        .map_err(|cause| Denial::Execution(cause.into()))
}

fn borrowed_denial(cause: super::RelationalBorrowedRecordReadDenial) -> Denial {
    match cause {
        super::RelationalBorrowedRecordReadDenial::ExactBasisRequired => Denial::ExactBasisRequired,
        super::RelationalBorrowedRecordReadDenial::Execution(cause) => Denial::Execution(cause),
    }
}

fn admit(context: &mut MapKernelContext<'_, '_>, work: u64) -> Result<(), Denial> {
    checkpoint(context, work)
        .map_err(|stop| Denial::Execution(crate::execution::denial::kernel_stop(stop)))
}
