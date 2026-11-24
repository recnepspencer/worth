//! The read-only projection of a genuine, still-unpublished Source loan.
use super::*;

pub fn applied_rows<'a>(
    projection: &'a VisibilityProjectionView,
    affinity: &'a Arc<AppliedSourceAffinity>,
    execution: ExecutionRequest<'a, 'a>,
) -> ManagedIndexSourceRows<'a> {
    ManagedIndexSourceRows {
        scope: affinity.scope,
        projection: ManagedIndexSourceProjection {
            program: &affinity.program,
            backing: SourceProjectionBacking::Applied {
                projection,
                affinity,
                execution,
                memory: RefCell::new(None),
                bytes: Cell::new(0),
            },
        },
    }
}
