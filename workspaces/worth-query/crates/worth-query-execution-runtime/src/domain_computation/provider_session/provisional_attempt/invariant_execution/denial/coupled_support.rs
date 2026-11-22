//! Coupled support refusals: a later coupled writer changed what a stage or
//! producer read, attributed to that writer. Every such refusal is plain (PR C
//! Q4): no change is held for an open issue.
use super::{WorthQueryInvariantExecutionDenialKind, WorthQueryInvariantExecutionFailure};
use std::sync::Arc;

impl WorthQueryInvariantExecutionFailure {
    pub fn coupled_read_support(
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
    pub fn coupled_consumption_cycle(
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
    pub fn missing_coupled_settlement(
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
    pub fn with_coupled_writer(
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
}
