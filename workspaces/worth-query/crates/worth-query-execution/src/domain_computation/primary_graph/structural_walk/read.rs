//! Invoke Native and retain only the complete body returned by that invocation.
use super::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::RelationDirection;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{
        PinnedStructuralWalkDenial, PinnedStructuralWalkDenialKind, PinnedStructuralWalkScope,
        RelationalPinnedRead,
    },
};

/// Exact query meaning, not an admission or permission to manufacture evidence.
pub(in crate::domain_computation::primary_graph) struct StructuralWalkQuery<'a> {
    pub(in crate::domain_computation::primary_graph) relation: &'a str,
    pub(in crate::domain_computation::primary_graph) roots: &'a [EntityId],
    pub(in crate::domain_computation::primary_graph) entity_kind: KindId,
    pub(in crate::domain_computation::primary_graph) relation_kind: KindId,
    pub(in crate::domain_computation::primary_graph) direction: RelationDirection,
    pub(in crate::domain_computation::primary_graph) maximum_work: Option<usize>,
    pub(in crate::domain_computation::primary_graph) control:
        Option<&'a dyn Fn() -> Result<(), Cause>>,
}

impl StructuralWalkObservation {
    pub(in crate::domain_computation::primary_graph) fn read(
        pin: &RelationalPinnedRead,
        execution: &ExecutionRequest<'_, '_>,
        query: StructuralWalkQuery<'_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Arc<Self>, PinnedStructuralWalkDenial> {
        let mut scope = PinnedStructuralWalkScope::new(
            query.roots,
            query.entity_kind,
            query.relation_kind,
            query.direction,
        )
        .with_work_limit(query.maximum_work);
        if let Some(control) = query.control {
            scope = scope.with_control(control);
        }
        let native = pin.complete_relation_walk(execution, scope)?;
        // These observations survive any later header or final-handoff failure.
        let reads = native.reads();
        let work = native.work_units();
        Self::retain(native, pin.snapshot(), execution, &query, context).map_err(|cause| {
            PinnedStructuralWalkDenial {
                kind: PinnedStructuralWalkDenialKind::Read(cause),
                reads,
                work,
            }
        })
    }

    // The only constructor is private to the actual-read owner. It accepts only
    // the successful Native body from read(), never caller-supplied evidence.
    fn retain(
        native: PinnedStructuralWalk,
        snapshot: &SnapshotHandle,
        execution: &ExecutionRequest<'_, '_>,
        query: &StructuralWalkQuery<'_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Arc<Self>, Cause> {
        let mut count = 0usize;
        for visit in native.visits() {
            Self::checkpoint(context)?;
            if let Some(control) = query.control {
                control()?;
            }
            if visit.distance != 0 {
                break;
            }
            count = count.checked_add(1).ok_or(Cause::ChargedBytesOverflow)?;
        }
        let bytes = count
            .checked_mul(size_of::<EntityId>())
            .and_then(|n| n.checked_add(query.relation.len()))
            .and_then(|n| n.checked_add(size_of::<key::Parameters>() + 4 * size_of::<usize>()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        let memory = memory::reserve(execution, context.has_accounting(), bytes)?;
        let mut roots = Vec::new();
        roots
            .try_reserve_exact(count)
            .map_err(|_| Cause::AllocationFailed)?;
        for visit in &native.visits()[..count] {
            Self::checkpoint(context)?;
            if let Some(control) = query.control {
                control()?;
            }
            roots.push(visit.entity.entity_id);
        }
        let control = query.control;
        let query = StructuralWalkKey(Arc::new(key::Parameters {
            relation: Arc::from(query.relation),
            entity_kind: query.entity_kind,
            relation_kind: query.relation_kind,
            direction: query.direction,
            roots,
            _memory: memory,
        }));
        let memory = memory::reserve(
            execution,
            context.has_accounting(),
            u64::try_from(size_of::<Self>() + 2 * size_of::<usize>())
                .ok()
                .and_then(|n| n.checked_add(snapshot.branch_id().0.len() as u64))
                .ok_or(Cause::ChargedBytesOverflow)?,
        )?;
        Self::checkpoint(context)?;
        if let Some(control) = control {
            control()?;
        }
        let commitment = super::commitment::derive(&query, &native, context, control)?;
        Ok(Arc::new(Self {
            query,
            native,
            commitment,
            snapshot: snapshot.clone(),
            _memory: memory,
        }))
    }
}
