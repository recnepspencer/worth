//! Definition-qualified stored identity lookup and merge-only current resolution.
use super::super::{AggregateLineageEvent, AggregateReadLineageEvent};
use super::*;
use std::collections::BTreeSet;
use std::sync::Arc;
use worth_execution::{ChargedBytes, ExecutionMemoryReservation};
use worth_foundational::facade::{AggregateDefinitionIdentity, AggregateIdentity};
use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;

/// Selection values only. The exact pin and original request remain authority.
pub struct PinnedAggregateLineageScope<'input> {
    selection: AggregateSelection,
    pub(super) identity: AggregateIdentity,
    limits: coordination::Limits<'input>,
}
impl<'input> PinnedAggregateLineageScope<'input> {
    pub fn new(
        relation: KindId,
        definition: AggregateDefinitionIdentity,
        identity: AggregateIdentity,
    ) -> Self {
        Self {
            selection: AggregateSelection::Orbit {
                relation,
                definition: Some(definition),
            },
            identity,
            limits: coordination::Limits::default(),
        }
    }
    pub fn for_definition(
        definition: AggregateDefinitionIdentity,
        identity: AggregateIdentity,
    ) -> Self {
        Self {
            selection: AggregateSelection::Definition(definition),
            identity,
            limits: coordination::Limits::default(),
        }
    }
    pub fn with_control(mut self, control: &'input dyn Fn() -> Result<(), Cause>) -> Self {
        self.limits.control = Some(control);
        self
    }
}

pub(super) struct Value {
    pub(super) definition: AggregateDefinitionIdentity,
    pub(super) identity: AggregateIdentity,
    pub(super) retired: Option<CommitId>,
    pub(super) incident: BTreeSet<AggregateLineageEvent>,
    pub(super) steps: Vec<AggregateLineageEvent>,
    pub(super) latest: Option<AggregateIdentity>,
}
struct Data {
    value: Value,
    reads: AggregateReadObservations,
    work: u64,
    _retention: Option<ExecutionMemoryReservation>,
}
impl ChargedBytes for Value {
    fn additional_charged_bytes(&self) -> u64 {
        (self.incident.len() as u64)
            .saturating_mul(extent::tree_entry::<AggregateLineageEvent, ()>())
            .saturating_add(
                (self.steps.capacity() as u64)
                    .saturating_mul(std::mem::size_of::<AggregateLineageEvent>() as u64),
            )
            .saturating_add(extent::arc_backing::<Data>())
    }
}

/// Actual incident history and validated merge-resolution steps at one source.
/// Missing identity is a denial; latest_identity()==None means a real retired
/// terminal without a successor. This value is not an empty membership group.
#[derive(Clone)]
pub struct PinnedAggregateLineageRead(Arc<Data>);
impl PinnedAggregateLineageRead {
    pub fn definition(&self) -> AggregateDefinitionIdentity {
        self.0.value.definition
    }
    pub fn identity(&self) -> AggregateIdentity {
        self.0.value.identity
    }
    pub fn retired_at(&self) -> Option<CommitId> {
        self.0.value.retired
    }
    pub fn latest_identity(&self) -> Option<AggregateIdentity> {
        self.0.value.latest
    }
    pub fn lineage(&self) -> impl ExactSizeIterator<Item = AggregateReadLineageEvent<'_>> {
        self.0
            .value
            .incident
            .iter()
            .map(AggregateReadLineageEvent::new)
    }
    /// Only the actual MergedInto continuation path. Split history never advances it.
    pub fn resolution_steps(&self) -> impl ExactSizeIterator<Item = AggregateReadLineageEvent<'_>> {
        self.0
            .value
            .steps
            .iter()
            .map(AggregateReadLineageEvent::new)
    }
    pub fn reads(&self) -> AggregateReadObservations {
        self.0.reads
    }
    pub fn work_units(&self) -> u64 {
        self.0.work
    }
}
impl std::fmt::Debug for PinnedAggregateLineageRead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PinnedAggregateLineageRead")
            .field("definition", &self.definition())
            .field("identity", &self.identity())
            .field("retired", &self.retired_at())
            .field("latest", &self.latest_identity())
            .field("reads", &self.reads())
            .finish()
    }
}

pub fn read_aggregate_lineage(
    basis: &VisibilitySnapshotBasis,
    execution: &ExecutionRequest<'_, '_>,
    scope: PinnedAggregateLineageScope<'_>,
    named_read: impl Fn(
        &super::super::AggregateGroupScheme,
        AggregateIdentity,
    ) -> Result<
        crate::contracts::aggregate_identity::group_lineage::AggregateGroupHistory,
        named::NamedAggregateLookupDenial,
    >,
) -> Result<PinnedAggregateLineageRead, PinnedAggregateReadDenial> {
    coordination::read(
        basis,
        execution,
        scope.limits,
        |root, view, work| {
            let contract = contract::resolve(root, scope.selection, work)?;
            if let Some(scheme) = contract.group_scheme(root.schema_authority().registry()) {
                return named::lineage(
                    contract.definition,
                    scope.identity,
                    |identity| named_read(&scheme, identity),
                    work,
                );
            }
            let selected = lookup::find(root, view, contract, scope.identity, work)?;
            let resolved = resolution::resolve(
                view,
                selected.clone(),
                contract,
                root.schema_authority().registry(),
                work,
            )?;
            work.checkpoint(0)?;
            Ok(Value {
                definition: contract.definition,
                identity: selected.record.identity,
                retired: selected.record.retired,
                incident: resolved.incident,
                steps: resolved.steps,
                latest: resolved.latest,
            })
        },
        |value, reads, work, retention| {
            PinnedAggregateLineageRead(Arc::new(Data {
                value,
                reads,
                work,
                _retention: retention,
            }))
        },
    )
}
