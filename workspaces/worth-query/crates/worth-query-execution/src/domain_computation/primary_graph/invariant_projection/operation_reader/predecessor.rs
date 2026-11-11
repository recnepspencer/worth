//! Sealed historical projection: C controls access, the predecessor owns values.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact as Fact,
    application_contribution::producer::predecessor_reads::{
        WorthQueryHistoricalProducerRead, WorthQueryPredecessorReadDenial as Denial,
    },
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationStructuredValueBinding,
};
mod reader;
pub use reader::{PredecessorDecisionReader, PredecessorEntity, PredecessorRelation};

impl<S: ApplicationSchema, O>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, S, O>
{
    pub(in crate::domain_computation::primary_graph) fn observe_predecessor<R, I, T>(
        &mut self,
        operation: ApplicationOperationRef<S, R, I>,
        read: impl FnOnce(&mut PredecessorDecisionReader<'_, '_, '_, S, R>) -> T,
    ) -> Result<T, Denial>
    where
        R: ApplicationOperationMarkerIdentity<S> + 'static,
        R::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: 'static,
    {
        let access = self.source_query.ok_or(Denial::UndeclaredPredecessorRead)?;
        let result = self.read_predecessor(operation, read);
        if let Err(cause) = &result {
            access.record_predecessor_failure(cause);
        }
        result
    }
    fn read_predecessor<R, I, T>(
        &mut self,
        operation: ApplicationOperationRef<S, R, I>,
        read: impl FnOnce(&mut PredecessorDecisionReader<'_, '_, '_, S, R>) -> T,
    ) -> Result<T, Denial>
    where
        R: ApplicationOperationMarkerIdentity<S> + 'static,
        R::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: 'static,
    {
        let access = self.source_query.ok_or(Denial::UndeclaredPredecessorRead)?;
        let scope = self
            .operation_scope
            .as_ref()
            .ok_or(Denial::PredecessorMismatch)?
            .scope();
        let scope = worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::new(scope.partition_id()),
            scope.local_slot(),
            scope.generation(),
        );
        let partition = self
            .reader
            .selected_source_partition_identity
            .ok_or(Denial::PredecessorMismatch)?;
        let execution = *self.reader.execution;
        let retention = self.reader.retention_control;
        let remaining = self.reader.work_budget.remaining();
        let native = self
            .reader
            .native
            .committed_runtime()
            .ok_or(Denial::PredecessorMismatch)?;
        let (prior, basis, contracts, authorization) =
            access.prepare_predecessor(&execution, native, operation, scope, partition)?;
        let authority = &access.application.mutation_projection;
        let limit = match (contracts.projection_work_budget(), remaining) {
            (Some(contract), Some(remaining)) => Some(contract.min(remaining)),
            (limit, None) | (None, limit) => limit,
        };
        let completed = authority.project_borrowed_basis(
            &execution,
            native,
            &basis,
            limit,
            |reader| {
                let (output, facts) =
                    authority.inspect_reader(reader, Some(contracts), None, |reader| {
                        read(&mut PredecessorDecisionReader { reader })
                    });
                (output, facts, reader.work_budget.consumed())
            },
            retention,
        );
        // Always recheck C, even when the consumer or projection refuses.
        access.predecessor_control(&execution, native)?;
        let completed = completed.map_err(Denial::Projection)?;
        let (output, facts, consumed) = completed.output;
        if !self.reader.work_budget.consume(consumed) {
            return Err(Denial::Projection(
                WorthQueryInvariantProjectionDenial::work_budget_exceeded(completed.work),
            ));
        }
        self.reader.work = self
            .reader
            .work
            .checked_combined(completed.work)
            .ok_or_else(|| Denial::Decision("predecessor projection work overflow".into()))?;
        let (keys, _) = facts
            .into_expected(&execution)
            .map_err(|cause| Denial::Decision(format!("{cause:?}")))?;
        access.record_predecessor_read(WorthQueryHistoricalProducerRead::from_read(
            &prior,
            &basis,
            R::IDENTIFIER,
            keys.into_historical_keys(),
        ));
        // Only C authorization becomes a current dependency. Old read keys and
        // old values are descriptive evidence and never rechecked at C.
        self.reader
            .dependent_source_facts
            .capture(
                Fact::SourceReadAuthorization {
                    observation: authorization,
                },
                retention,
            )
            .map_err(|cause| Denial::Decision(format!("{cause:?}")))?;
        Ok(output)
    }
}
