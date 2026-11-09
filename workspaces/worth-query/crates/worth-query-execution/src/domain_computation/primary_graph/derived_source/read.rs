//! Only the successful controlled Native acquisition can construct an observation.
use super::*;
use crate::domain_computation::primary_graph::structural_walk::memory;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::RelationDirection;
use worth_relational::facade::{
    identity::EntityId,
    runtime::{
        DerivedRelationReadDenial, DerivedRelationReadShape, PinnedDerivedRelationDenial,
        PinnedDerivedRelationScope, RelationalPinnedRead,
    },
};
pub(in crate::domain_computation::primary_graph) struct DerivedSourceQuery<'a> {
    pub relation: &'a str,
    pub definition: &'a DerivedRelationHandle,
    pub root: EntityId,
    pub direction: RelationDirection,
    pub control: Option<&'a dyn Fn() -> Result<(), Cause>>,
}
impl DerivedSourceObservation {
    pub(in crate::domain_computation::primary_graph) fn read_projected<Phase>(
        pin: &RelationalPinnedRead<Phase>,
        origin: super::super::projection_read_origin::ProjectionReadOriginRef<'_>,
        execution: &ExecutionRequest<'_, '_>,
        query: DerivedSourceQuery<'_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Arc<Self>, PinnedDerivedRelationDenial> {
        let mut scope = PinnedDerivedRelationScope::new(
            query.definition,
            query.root,
            DerivedRelationReadShape::Adjacency {
                reverse: query.direction == RelationDirection::Reverse,
            },
        );
        if let Some(control) = query.control {
            scope = scope.with_control(control);
        }
        let native = pin.read_derived_source(execution, scope)?;
        let reads = native.data().reads();
        let work = native.work_units();
        Self::retain(native, origin, execution, query, context).map_err(|cause| {
            PinnedDerivedRelationDenial {
                kind: DerivedRelationReadDenial::Request(cause),
                reads,
                work,
            }
        })
    }
    fn retain(
        native: PinnedDerivedRelationRead,
        origin: super::super::projection_read_origin::ProjectionReadOriginRef<'_>,
        execution: &ExecutionRequest<'_, '_>,
        query: DerivedSourceQuery<'_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Arc<Self>, Cause> {
        Self::checkpoint(context)?;
        if let Some(control) = query.control {
            control()?;
        }
        let id = &query.definition.parts().identity;
        let key_bytes = query
            .relation
            .len()
            .checked_add(id.schema.0.len())
            .and_then(|n| n.checked_add(id.name.len()))
            .and_then(|n| n.checked_add(size_of::<key::Parameters>() + 2 * size_of::<usize>()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        // Clone only after the actual widths have been admitted on the same
        // original request. This bound covers all three owned name copies.
        for name in [query.relation, id.schema.0.as_str(), id.name.as_str()] {
            let units = u64::try_from(name.len())
                .ok()
                .and_then(|n| n.checked_mul(11))
                .ok_or(Cause::WorkCounterOverflow)?;
            (if context.has_accounting() {
                context.checkpoint(units)
            } else {
                context.poll()
            })
            .map_err(Cause::from)?;
            if let Some(control) = query.control {
                control()?;
            }
        }
        let key_memory = memory::reserve(execution, context.has_accounting(), key_bytes)?;
        let key = DerivedSourceKey(Arc::new(key::Parameters {
            relation: query.relation.to_owned(),
            identity: id.clone(),
            root: query.root,
            direction: query.direction,
            _memory: key_memory,
        }));
        let header = size_of::<Self>()
            .checked_add(2 * size_of::<usize>())
            .and_then(|n| {
                n.checked_add(
                    origin
                        .committed_snapshot()
                        .map_or(0, |snapshot| snapshot.branch_id().0.len()),
                )
            })
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        let memory = memory::reserve(execution, context.has_accounting(), header)?;
        let commitment = commitment::derive(&key, native.data(), context, query.control)?;
        Self::checkpoint(context)?;
        if let Some(control) = query.control {
            control()?;
        }
        Ok(Arc::new(Self {
            key,
            definition: query.definition.clone(),
            native,
            commitment,
            snapshot: origin.retained(),
            _memory: memory,
        }))
    }
}
