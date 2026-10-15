//! Funded immutable wire for native revisions, never a manufactured query source.
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::{AuthoringSourceFacts, StorageControl},
    application_checkpoint::facts::{self, FactDecodeDenial, FactDecoder},
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryApplicationObservedFact as Fact,
};
use worth_execution::{
    ExecutionArrayBuilder, ExecutionByteBuffer, ExecutionImmutableBytes, ExecutionRequest,
};
use worth_relational::facade::{
    identity::EntityId, runtime::RelationalRuntime, snapshots::SnapshotHandle,
};

#[derive(Clone, Debug)]
pub(in crate::domain_computation) struct WorkflowActorDependencies {
    wire: ExecutionImmutableBytes,
    counts: [usize; 3],
}
impl WorkflowActorDependencies {
    pub(in crate::domain_computation::primary_graph) fn capture(
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        layout: &super::super::schema::WorthQueryWorkflowLayout,
        observed: &[Fact],
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<Self, Denial> {
        let control = StorageControl::new(execution.allocation_policy(), Some(request));
        let mut store = AuthoringSourceFacts::new(control)
            .map_err(|cause| cause.into_attempt_denial("workflow actor dependencies"))?;
        for fact in observed {
            let source =
                super::revision_transport::capture(execution, runtime, snapshot, layout, fact)?;
            store
                .capture(source, control)
                .map_err(|cause| cause.into_attempt_denial("workflow actor dependencies"))?;
        }
        let retained = store
            .finish(control)
            .map_err(|cause| cause.into_attempt_denial("workflow actor dependencies"))?;
        let mut temporary =
            ExecutionArrayBuilder::allocate(retained.len(), execution.allocation_policy())
                .map_err(|cause| super::storage_denial(cause, "workflow actor encoding"))?;
        for fact in retained.into_values() {
            temporary
                .push(fact)
                .map_err(|cause| super::storage_denial(cause, "workflow actor encoding"))?;
        }
        let temporary = temporary
            .seal()
            .map_err(|cause| super::storage_denial(cause, "workflow actor encoding"))?;
        let bytes = facts::encode_funded(temporary.elements(), control).map_err(decode_denial)?;
        Self::from_wire(execution, bytes)
    }
    pub(in crate::domain_computation) fn wire(&self) -> &[u8] {
        self.wire.bytes()
    }
    pub(in crate::domain_computation) fn counts(&self) -> [usize; 3] {
        self.counts
    }
    pub(in crate::domain_computation) fn remains_current(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, Denial> {
        let mut decoder = decoder(execution, self.wire())?;
        while let Some(fact) = decoder.next_fact().map_err(decode_denial)? {
            if !fact.remains_equal_in(runtime, execution, snapshot)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    pub(in crate::domain_computation::primary_graph) fn recorded_actor(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        entity: EntityId,
        role: worth_query_declaration::facade::application_capability::ApplicationWorkflowRecordedRole,
    ) -> Result<Option<EntityId>, Denial> {
        use worth_query_declaration::facade::application_capability::ApplicationWorkflowRecordedRole as Role;
        let (aspect, role) = match role {
            Role::Proposer => ("workflow-proposal", "proposer"),
            Role::Advancer => ("workflow-transition", "advancer"),
            Role::Executor => ("workflow-transition", "executor"),
            Role::Approver => ("workflow-approval", "approver"),
        };
        let names = [
            format!("{role}-partition"),
            format!("{role}-slot"),
            format!("{role}-generation"),
        ];
        let mut locators = [None, None, None];
        let mut decoder = decoder(execution, self.wire())?;
        while let Some(fact) = decoder.next_fact().map_err(decode_denial)? {
            if let Fact::SourceFieldRevision {
                entity_id, locator, ..
            }
            | Fact::Field {
                entity_id, locator, ..
            }
            | Fact::AbsentField {
                entity_id, locator, ..
            } = fact
            {
                if entity_id == entity
                    && locator.aspect().aspect_key().as_str() == aspect
                    && locator.field_path().fields().len() == 1
                {
                    if let Some(index) = names
                        .iter()
                        .position(|name| name == locator.field_path().fields()[0].as_str())
                    {
                        locators[index] = Some(locator);
                    }
                }
            }
        }
        let [Some(partition), Some(slot), Some(generation)] = locators else {
            return Err(invalid("recorded role lacks its native field footprint"));
        };
        let fields = super::record::WorkflowActorFields {
            partition,
            slot,
            generation,
        };
        let kind = runtime
            .read_truth()
            .exact_snapshot_live_entity_kind_status(snapshot, entity)
            .flatten()
            .ok_or_else(|| {
                Denial::new(
                    Kind::StaleEntityIdentity,
                    "recorded actor entity is unavailable",
                )
            })?;
        fields.observe(runtime, execution, snapshot, entity, kind, &mut Vec::new())
    }
    pub(in crate::domain_computation::primary_graph) fn restore(
        execution: &ExecutionRequest<'_, '_>,
        bytes: &[u8],
    ) -> Result<Self, Denial> {
        let mut output = ExecutionByteBuffer::allocate(bytes.len(), execution.allocation_policy())
            .map_err(|cause| super::storage_denial(cause, "workflow actor wire"))?;
        output
            .extend_from_slice(bytes)
            .map_err(|cause| super::storage_denial(cause, "workflow actor wire"))?;
        let wire = output
            .seal()
            .map_err(|cause| super::storage_denial(cause, "workflow actor wire"))?;
        Self::from_wire(execution, wire)
    }
    fn from_wire(
        execution: &ExecutionRequest<'_, '_>,
        bytes: ExecutionImmutableBytes,
    ) -> Result<Self, Denial> {
        let mut reader = decoder(execution, bytes.bytes())?;
        let mut counts = [0; 3];
        while let Some(fact) = reader.next_fact().map_err(decode_denial)? {
            match fact {
                Fact::Entity { .. } | Fact::SourceEntity { .. } => counts[0] += 1,
                Fact::SourceAdjacencyRevision {
                    endpoints,
                    comparison_work_limit: 1,
                    ..
                } if endpoints.len() == 0 => counts[1] += 1,
                Fact::SourceFieldRevision { .. }
                | Fact::SourceAspectRevision { .. }
                | Fact::Field { .. }
                | Fact::AbsentField { .. } => counts[2] += 1,
                _ => return Err(invalid("unexpected workflow actor dependency")),
            }
        }
        Ok(Self {
            wire: bytes,
            counts,
        })
    }
}
fn decoder<'bytes, 'scope, 'authority>(
    execution: &ExecutionRequest<'scope, 'authority>,
    bytes: &'bytes [u8],
) -> Result<FactDecoder<'bytes, 'scope, 'authority>, Denial> {
    FactDecoder::new(
        bytes,
        facts::WIRE_VERSION,
        None,
        execution.allocation_policy(),
    )
    .map_err(decode_denial)
}
pub(super) fn decode_denial(cause: FactDecodeDenial) -> Denial {
    match cause {
        FactDecodeDenial::Retention(cause) => {
            cause.into_attempt_denial("workflow actor dependency wire")
        }
        FactDecodeDenial::Execution(cause) => {
            Denial::relational_read_stopped(cause, "workflow actor dependency payload")
        }
        FactDecodeDenial::Format(_) => invalid("workflow actor dependency wire is invalid"),
    }
}
fn invalid(subject: &str) -> Denial {
    Denial::new(Kind::InvalidAuthoritativeValue, subject)
}
