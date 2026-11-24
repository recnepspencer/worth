//! Execute one exact-root walk under the original request.
use super::{
    memory, traversal, PinnedStructuralWalk, PinnedStructuralWalkDenial,
    PinnedStructuralWalkDenialKind, PinnedStructuralWalkReads, PinnedStructuralWalkScope,
};
use crate::visibility::materialization::read_records::projection::{
    pinned::PinnedProjectionSource, VisibilityProjectionView,
};
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;

pub fn complete(
    execution: &ExecutionRequest<'_, '_>,
    scope: PinnedStructuralWalkScope<'_>,
    source: PinnedProjectionSource<'_>,
) -> Result<PinnedStructuralWalk, PinnedStructuralWalkDenial> {
    let mut reads = PinnedStructuralWalkReads::default();
    let mut work = 0;
    // Final handoff may discard a completed value. Observations therefore
    // remain here, outside the native callback's returned payload.
    let result = execution.run_native_with_checkpoint(|context| {
        traversal::checkpoint(context, &scope, &mut work, 1)?;
        let schema = source
            .resident
            .projection_root()
            .schema_authority()
            .registry();
        if !(*schema.owner_entity_kinds()).contains_key(&scope.entity_kind)
            || !(*schema.owner_relation_kinds()).contains_key(&scope.relation_kind)
        {
            return Err(PinnedStructuralWalkDenialKind::KindUnavailable);
        }
        let projection = VisibilityProjectionView::from_pinned(source);
        let memory = memory::WalkMemory::new(execution, context.has_accounting())?;
        traversal::execute(
            &projection,
            execution,
            &scope,
            context,
            &mut reads,
            &mut work,
            memory,
        )
    });
    match result {
        Ok((Ok(walk), _physical)) => {
            if let Some(control) = scope.control {
                control().map_err(|cause| PinnedStructuralWalkDenial {
                    kind: PinnedStructuralWalkDenialKind::Read(cause),
                    reads,
                    work,
                })?;
            }
            Ok(walk)
        }
        Ok((Err(kind), _physical)) => Err(PinnedStructuralWalkDenial { kind, reads, work }),
        Err(stop) => Err(PinnedStructuralWalkDenial {
            kind: PinnedStructuralWalkDenialKind::Read(
                crate::visibility::materialization::read_records::read_request::cause(stop),
            ),
            reads,
            work,
        }),
    }
}
