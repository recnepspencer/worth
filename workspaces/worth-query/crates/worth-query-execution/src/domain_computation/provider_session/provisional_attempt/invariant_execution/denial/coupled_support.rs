//! Coupled support refusals: a later coupled writer changed what a stage or
//! producer read, attributed to that writer, and the #229 hold.
use super::{WorthQueryInvariantExecutionDenialKind, WorthQueryInvariantExecutionFailure};
use std::sync::Arc;

/// A coupled support refusal the platform holds for a named public issue.
/// The edit is valid sequentially; it refuses atomically until the issue lands.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCoupledSupportHold {
    /// A later authored stage changed a fact that an output derived earlier in
    /// the same envelope read, as the adjacent writer images show. Committing
    /// it needs a second derivation of that route, which waits on in-envelope
    /// producer supersession (public #229).
    ProducerSupersession,
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
