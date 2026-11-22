//! Coupled support refusals: a later coupled writer changed what a stage or
//! producer read, attributed to that writer, and the #229 hold.
use super::{WorthQueryInvariantExecutionDenialKind, WorthQueryInvariantExecutionFailure};
use std::sync::Arc;

/// A coupled support refusal the platform holds for a named public issue.
/// The edit refuses whole with `CoupledReadSupportChanged`, attributed to the
/// reader and the actual later writer, and nothing commits. The hold names
/// the issue whose capability the refused shape waits on; it claims nothing
/// about what any other order of the same work would commit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCoupledSupportHold {
    /// A later authored stage changed a fact that an output derived earlier in
    /// the same envelope read, as the adjacent writer images show. Committing
    /// it needs a second derivation of that route, which waits on in-envelope
    /// producer supersession (public #229).
    ProducerSupersession,
    /// A later authored stage changed the adjacency an authored stage's
    /// discovery read. Settling it waits on discovery settlement (public #238).
    DiscoverySupportChanged,
    /// A producer whose derivation consumed a verified parent's output, which
    /// an authored stage read, changed that parent's support. Settling or
    /// superseding the parent inside the envelope waits on public #239.
    ReaderProducerSupersession,
    /// A producer whose derivation consumed another producer's output in the
    /// same envelope changed a decision fact that producer read. Settling it
    /// waits on public #246.
    ConsumerSupersession,
}

impl WorthQueryInvariantExecutionFailure {
    pub(crate) fn coupled_read_support(
        source: worth_relational::facade::identity::EntityId,
        cause: Option<
            crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind,
        >,
        detail: impl Into<Arc<str>>,
    ) -> Self {
        let kind = cause.map_or(
            WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged,
            |cause| WorthQueryInvariantExecutionDenialKind::CoupledReadSupportUnavailable { cause },
        );
        let mut failure = Self::new(kind, detail);
        failure.affected_records = Some(Arc::from([
            worth_relational::facade::transactions::RecordRef::Entity(source),
        ]));
        failure
    }
    /// Members of one coupled group consume each other's outputs in a cycle,
    /// attributed to each source entity of a member no consumption order can
    /// place, once.
    pub(crate) fn coupled_consumption_cycle(
        sources: impl IntoIterator<Item = worth_relational::facade::identity::EntityId>,
    ) -> Self {
        let mut failure = Self::new(
            WorthQueryInvariantExecutionDenialKind::CoupledConsumptionCycle,
            "coupled consumed-output settlement formed a cycle",
        );
        let mut records = Vec::<worth_relational::facade::transactions::RecordRef>::new();
        for source in sources {
            let record = worth_relational::facade::transactions::RecordRef::Entity(source);
            if !records.contains(&record) {
                records.push(record);
            }
        }
        failure.affected_records = Some(records.into());
        failure
    }
    pub(crate) fn missing_coupled_settlement(
        source: worth_relational::facade::identity::EntityId,
    ) -> Self {
        let mut denied = Self::new(
            WorthQueryInvariantExecutionDenialKind::CoupledSettlementMissing,
            "Native participant has no retained Query settlement session",
        );
        denied.affected_records = Some(Arc::from([
            worth_relational::facade::transactions::RecordRef::Entity(source),
        ]));
        denied
    }
    pub(crate) fn with_coupled_writer(
        mut self,
        writer: worth_relational::facade::identity::EntityId,
    ) -> Self {
        let mut records = self.affected_records().to_vec();
        let writer = worth_relational::facade::transactions::RecordRef::Entity(writer);
        if !records.contains(&writer) {
            records.push(writer);
        }
        self.affected_records = Some(Arc::from(records));
        self
    }
    pub(crate) fn held(mut self, hold: WorthQueryCoupledSupportHold) -> Self {
        self.hold = Some(hold);
        self
    }
    /// The public issue this refusal is held for, when it is a hold.
    pub fn coupled_support_hold(&self) -> Option<WorthQueryCoupledSupportHold> {
        self.hold
    }
}
