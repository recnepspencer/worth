//! Only the snapshot executor issues this complete original Native observation.
use super::super::WorthQueryApplicationInvariantProjectionReader;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, MapKernelContext, WorkCeilingDenial};
use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_relational::facade::{
    identity::EntityId, runtime::PinnedStructuralWalk, snapshots::SnapshotHandle,
    transactions::RelationalExecutionDenialCause as Cause,
};
mod currentness;
mod key;
pub(in crate::domain_computation::primary_graph) use key::StructuralWalkKey;

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct StructuralWalkObservation {
    query: StructuralWalkKey,
    native: PinnedStructuralWalk,
    snapshot: SnapshotHandle,
    cookie: u64,
    _memory: Option<ExecutionMemoryReservation>,
}
impl StructuralWalkObservation {
    /// Fallible storage comparison; graph currentness still performs a fresh Native read.
    pub(in crate::domain_computation::primary_graph) fn same_truth_with_poll<E>(
        &self,
        other: &Self,
        mut poll: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        poll()?;
        let (a, b) = (self.key(), other.key());
        if a.relation() != b.relation()
            || a.entity_kind() != b.entity_kind()
            || a.relation_kind() != b.relation_kind()
            || a.direction() != b.direction()
            || a.roots().len() != b.roots().len()
            || self.native.visits().len() != other.native.visits().len()
            || self.native.relations().len() != other.native.relations().len()
        {
            return Ok(false);
        }
        for (a, b) in a.roots().iter().zip(b.roots()) {
            poll()?;
            if a != b {
                return Ok(false);
            }
        }
        for (a, b) in self.native.visits().iter().zip(other.native.visits()) {
            poll()?;
            if a != b {
                return Ok(false);
            }
        }
        for (a, b) in self.native.relations().iter().zip(other.native.relations()) {
            poll()?;
            if a != b {
                return Ok(false);
            }
        }
        poll()?;
        Ok(true)
    }
    pub(super) fn issue<Schema>(
        native: PinnedStructuralWalk,
        relation: &str,
        direction: RelationDirection,
        reader: &WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Arc<Self>, Cause> {
        let layout = reader
            .layout
            .relation(relation)
            .expect("completed walk validated its relation");
        let mut count = 0usize;
        for visit in native.visits() {
            Self::checkpoint(context)?;
            if visit.distance != 0 {
                break;
            }
            count = count.checked_add(1).ok_or(Cause::ChargedBytesOverflow)?;
        }
        let bytes = count
            .checked_mul(size_of::<EntityId>())
            .and_then(|n| n.checked_add(relation.len()))
            .and_then(|n| n.checked_add(size_of::<key::Parameters>() + 4 * size_of::<usize>()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        let memory = super::memory::reserve(reader.execution, context.has_accounting(), bytes)?;
        let mut roots = Vec::new();
        roots
            .try_reserve_exact(count)
            .map_err(|_| Cause::AllocationFailed)?;
        for visit in &native.visits()[..count] {
            Self::checkpoint(context)?;
            roots.push(visit.entity.entity_id);
        }
        let query = StructuralWalkKey(Arc::new(key::Parameters {
            relation: Arc::from(relation),
            entity_kind: layout.from,
            relation_kind: layout.kind,
            direction,
            roots,
            _memory: memory,
        }));
        let memory = super::memory::reserve(
            reader.execution,
            context.has_accounting(),
            u64::try_from(size_of::<Self>() + 2 * size_of::<usize>())
                .ok()
                .and_then(|n| n.checked_add(reader.snapshot.branch_id().0.len() as u64))
                .ok_or(Cause::ChargedBytesOverflow)?,
        )?;
        Self::checkpoint(context)?;
        Ok(Arc::new(Self {
            query,
            native,
            snapshot: reader.snapshot.clone(),
            cookie: reader.authority_identity,
            _memory: memory,
        }))
    }
    pub(in crate::domain_computation::primary_graph) fn key(&self) -> &StructuralWalkKey {
        &self.query
    }
    pub(in crate::domain_computation::primary_graph) fn native(&self) -> &PinnedStructuralWalk {
        &self.native
    }
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn issued_at(
        &self,
        snapshot: &SnapshotHandle,
        cookie: u64,
    ) -> bool {
        self.snapshot == *snapshot && self.cookie == cookie
    }
    pub(in crate::domain_computation::primary_graph) fn source(&self) -> &SnapshotHandle {
        &self.snapshot
    }
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn cookie(&self) -> u64 {
        self.cookie
    }
    pub(in crate::domain_computation::primary_graph) fn checkpoint(
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Cause> {
        (if context.has_accounting() {
            context.checkpoint(1)
        } else {
            context.poll()
        })
        .map_err(|stop| Cause::from(WorkCeilingDenial::Stopped(stop)))
    }
}
impl PartialEq for StructuralWalkObservation {
    fn eq(&self, other: &Self) -> bool {
        self.query == other.query
            && self.native.visits() == other.native.visits()
            && self.native.relations() == other.native.relations()
    }
}
impl Eq for StructuralWalkObservation {}
