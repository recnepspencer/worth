//! Original admitted C supplies readonly numerical DATA; actual execution stays D.
use super::predecessor::BeforeDecisionReader;
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact as Fact,
    application_contribution::producer::observation_reads::WorthQueryProducerObservationReadDenial as Denial,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationStructuredValueBinding,
};
impl<S: ApplicationSchema, O>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, S, O>
{
    pub(in crate::domain_computation::primary_graph) fn observe_original<R, I, T>(
        &mut self,
        operation: ApplicationOperationRef<S, R, I>,
        read: impl FnOnce(&mut BeforeDecisionReader<'_, '_, '_, S, R>) -> T,
    ) -> Result<T, Denial>
    where
        R: ApplicationOperationMarkerIdentity<S> + 'static,
        R::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: 'static,
    {
        let access = self.source_query.ok_or(Denial::UndeclaredObservationRead)?;
        let result = self.read_original(operation, read);
        if let Err(cause) = &result {
            access.record_observation_failure(cause);
        }
        result
    }
    fn read_original<R, I, T>(
        &mut self,
        operation: ApplicationOperationRef<S, R, I>,
        read: impl FnOnce(&mut BeforeDecisionReader<'_, '_, '_, S, R>) -> T,
    ) -> Result<T, Denial>
    where
        R: ApplicationOperationMarkerIdentity<S> + 'static,
        R::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: 'static,
    {
        let access = self.source_query.ok_or(Denial::UndeclaredObservationRead)?;
        let product = self
            .selected_product
            .ok_or(Denial::UndeclaredObservationRead)?;
        let snapshot = self.reader.snapshot;
        let execution = *self.reader.execution;
        let retention = self.reader.retention_control;
        let remaining = self.reader.work_budget.remaining();
        let authority = &access.application.mutation_projection;
        let project = |native: &worth_relational::facade::runtime::RelationalRuntime| {
            let (contracts, authorization) =
                access.prepare_original(&execution, native, operation)?;
            let limit = match (contracts.projection_work_budget(), remaining) {
                (Some(contract), Some(remaining)) => Some(contract.min(remaining)),
                (limit, None) | (None, limit) => limit,
            };
            let completed = authority.read_native_with_access(
                &execution,
                limit.map_or_else(
                    super::super::work::WorthQueryInvariantProjectionWorkBudget::unbounded,
                    super::super::work::WorthQueryInvariantProjectionWorkBudget::bounded,
                ),
                product.relational_basis(),
                snapshot,
                super::super::locked_reader::NativeReadAccess::Borrowed(native),
                |reader| {
                    let (output, facts) =
                        authority.inspect_reader(reader, Some(contracts), None, |reader| {
                            read(&mut BeforeDecisionReader { reader })
                        });
                    (output, facts, reader.work_budget.consumed())
                },
                retention,
            );
            // Current permission/control is checked even when projection refuses.
            access.observation_control(&execution, native)?;
            Ok::<_, Denial>((authorization, completed.map_err(Denial::Projection)?))
        };
        let (authorization, completed) = match self.reader.native.committed_runtime() {
            Some(native) => project(native),
            None => access
                .application
                .runtime
                .primary_provider
                .graph
                .with_runtime(&execution, project),
        }?;
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
            .ok_or_else(|| Denial::Decision("original projection work overflow".into()))?;
        // Validate C's own data locally. None of its value keys becomes a D expectation.
        facts
            .into_expected(&execution)
            .map_err(|cause| Denial::Decision(format!("{cause:?}")))?;
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
