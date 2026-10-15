//! Paid current native field projections; no command facts or actor policy.
use super::*;
use crate::domain_computation::primary_graph::application_discovery::WorthQueryWorkflowReadDenial as Denial;
use worth_foundational::facade::{AspectFieldLocator, InternedString};
use worth_relational::facade::{
    identity::KindId,
    runtime::{RelationalPinnedRead, RelationalRuntime},
    snapshots::SnapshotHandle,
};
pub(in crate::domain_computation::primary_graph) struct ReadFields<'a, 'e, 'r> {
    pub(in crate::domain_computation::primary_graph) runtime: &'a RelationalRuntime,
    pub(in crate::domain_computation::primary_graph) execution:
        &'a worth_execution::ExecutionRequest<'e, 'r>,
    pin: RelationalPinnedRead,
}
impl<'a, 'e, 'r> ReadFields<'a, 'e, 'r> {
    pub(in crate::domain_computation::primary_graph) fn new(
        runtime: &'a RelationalRuntime,
        execution: &'a worth_execution::ExecutionRequest<'e, 'r>,
        snapshot: &SnapshotHandle,
    ) -> Result<Self, Denial> {
        Ok(Self {
            runtime,
            execution,
            pin: runtime
                .read_truth()
                .pin_read_capability(snapshot, &[], execution)
                .map_err(Denial::PinnedRead)?,
        })
    }
    pub(in crate::domain_computation::primary_graph) fn snapshot(&self) -> &SnapshotHandle {
        self.pin.snapshot()
    }
    pub(in crate::domain_computation::primary_graph) fn u64(
        &self,
        entity: EntityId,
        kind: KindId,
        field: &AspectFieldLocator,
    ) -> Result<u64, Denial> {
        super::super::observe_field_value_borrowed(
            self.runtime,
            self.execution,
            self.snapshot(),
            entity,
            kind,
            field,
            |v| match v {
                Some(AspectValue::UInt64(v)) => Some(*v),
                _ => None,
            },
        )
        .map_err(Denial::Relational)?
        .flatten()
        .ok_or(Denial::Corrupt)
    }
    pub(in crate::domain_computation::primary_graph) fn optional_text(
        &self,
        entity: EntityId,
        kind: KindId,
        field: &AspectFieldLocator,
    ) -> Result<Option<worth_execution::ExecutionImmutableBytes>, Denial> {
        super::super::observe_field_value_borrowed(
            self.runtime,
            self.execution,
            self.snapshot(),
            entity,
            kind,
            field,
            |v| {
                let text = match v {
                    None => return Ok(None),
                    Some(AspectValue::String(InternedString::Raw(v))) => v.as_str(),
                    Some(AspectValue::String(InternedString::Symbol(v))) => self
                        .pin
                        .symbols()
                        .resolve(worth_relational::facade::symbols::Symbol(v.0))
                        .ok_or(Denial::Corrupt)?,
                    _ => return Err(Denial::Corrupt),
                };
                // The existing native read quotes comparison/read work before this projection.
                let mut bytes = worth_execution::ExecutionByteBuffer::allocate(
                    text.len(),
                    self.execution.allocation_policy(),
                )
                .map_err(Denial::Allocation)?;
                bytes
                    .extend_from_slice(text.as_bytes())
                    .map_err(Denial::Allocation)?;
                bytes.seal().map(Some).map_err(Denial::Allocation)
            },
        )
        .map_err(Denial::Relational)?
        .ok_or(Denial::Corrupt)?
    }
    pub(in crate::domain_computation::primary_graph) fn text(
        &self,
        entity: EntityId,
        kind: KindId,
        field: &AspectFieldLocator,
    ) -> Result<worth_execution::ExecutionImmutableBytes, Denial> {
        self.optional_text(entity, kind, field)?
            .ok_or(Denial::Corrupt)
    }
    pub(in crate::domain_computation::primary_graph) fn one(
        &self,
        kind: worth_relational::facade::identity::KindId,
        anchor: EntityId,
    ) -> Result<Option<EntityId>, Denial> {
        let rows = observe_adjacency(
            self.runtime,
            self.execution,
            self.snapshot(),
            kind,
            anchor,
            WorthQueryApplicationAdjacencyDirection::Outgoing,
            2,
        )
        .map_err(Denial::Execution)?
        .ok_or(Denial::BasisUnavailable)?;
        match rows.as_slice() {
            [] => Ok(None),
            [one] => Ok(Some(one.to)),
            _ => Err(Denial::Corrupt),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn incoming_one(
        &self,
        kind: KindId,
        anchor: EntityId,
    ) -> Result<EntityId, Denial> {
        let rows = observe_adjacency(
            self.runtime,
            self.execution,
            self.snapshot(),
            kind,
            anchor,
            WorthQueryApplicationAdjacencyDirection::Incoming,
            2,
        )
        .map_err(Denial::Execution)?
        .ok_or(Denial::BasisUnavailable)?;
        match rows.as_slice() {
            [one] => Ok(one.from),
            _ => Err(Denial::Corrupt),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn protocol(
        &self,
        entity: EntityId,
        kind: KindId,
        field: &AspectFieldLocator,
    ) -> Result<(), Denial> {
        if self.u64(entity,kind,field)?!=crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION {return Err(Denial::Corrupt);}
        Ok(())
    }
}
pub(in crate::domain_computation::primary_graph) fn identity(
    value: &[u8],
) -> Result<[u8; 32], Denial> {
    let text = std::str::from_utf8(value).map_err(|_| Denial::Corrupt)?;
    crate::domain_computation::primary_graph::workflow::proposal::decode_identity(text)
        .ok_or(Denial::Corrupt)
}
