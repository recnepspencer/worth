//! Sealed historical projection: C controls access, the predecessor owns values.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact as Fact,
    application_contribution::producer::observation_reads::{
        ProducerPredecessorReadAccess, WorthQueryHistoricalProducerRead,
        WorthQueryProducerObservationReadDenial as Denial,
    },
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationStructuredValueBinding,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationSchema, WorthQueryInstalledPackageIndex,
};
use crate::domain_computation::primary_graph::invariant_projection::WorthQueryApplicationInvariantProjectionAuthority;
mod reader;
pub use reader::{BeforeDecisionReader, BeforeEntity, BeforeRelation};

pub trait OperationReaderPredecessorExt<
    S: ApplicationSchema + 'static,
>
{
    fn observe_predecessor<R, I, T>(
        &mut self,
        operation: ApplicationOperationRef<S, R, I>,
        read: impl FnOnce(&mut BeforeDecisionReader<'_, '_, '_, S, R>) -> T,
    ) -> Result<T, Denial>
    where
        R: ApplicationOperationMarkerIdentity<S> + 'static,
        R::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: 'static;
}

impl<S: ApplicationSchema + 'static, O> OperationReaderPredecessorExt<S>
    for WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, S, O>
{
    fn observe_predecessor<R, I, T>(
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
        let result = read_predecessor(self, operation, read);
        if let Err(cause) = &result {
            access.record_observation_failure(cause);
        }
        result
    }
}

fn read_predecessor<S: ApplicationSchema + 'static, O, R, I, T>(
    reader_self: &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, S, O>,
    operation: ApplicationOperationRef<S, R, I>,
    read: impl FnOnce(&mut BeforeDecisionReader<'_, '_, '_, S, R>) -> T,
) -> Result<T, Denial>
where
    R: ApplicationOperationMarkerIdentity<S> + 'static,
    R::InputBinding: ApplicationStructuredValueBinding<Value = I>,
    I: 'static,
{
    let access = reader_self
        .source_query
        .ok_or(Denial::UndeclaredObservationRead)?;
    let scope = reader_self
        .operation_scope
        .as_ref()
        .ok_or(Denial::PredecessorMismatch)?
        .scope();
    let scope = worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::new(scope.partition_id()),
        scope.local_slot(),
        scope.generation(),
    );
    let partition = reader_self
        .reader
        .selected_source_partition_identity
        .ok_or(Denial::PredecessorMismatch)?;
    let execution = *reader_self.reader.execution;
    let retention = reader_self.reader.retention_control;
    let remaining = reader_self.reader.work_budget.remaining();
    let (schema_any, authority_any, packages_any, predecessor_any, _) =
        access.observation_components_any();
    let installed_schema = schema_any
        .downcast_ref::<WorthQueryInstalledApplicationSchema<S>>()
        .expect("source query reader matches schema");
    let authority = authority_any
        .downcast_ref::<WorthQueryApplicationInvariantProjectionAuthority<S>>()
        .expect("source query reader matches authority");
    let installed_packages = packages_any
        .downcast_ref::<WorthQueryInstalledPackageIndex>()
        .expect("source query reader carries package registry");
    let predecessor_access = predecessor_any
        .and_then(|a| a.downcast_ref::<ProducerPredecessorReadAccess>())
        .ok_or(Denial::UndeclaredObservationRead)?;
    let mut read_slot = Some(read);
    let mut project = |native: &worth_relational::facade::runtime::RelationalRuntime| {
        access.observation_control(&execution, native)?;
        let installed = installed_schema
            .installed_operation(operation)
            .map_err(Denial::Installation)?;
        installed_packages
            .validate_application_operation(&installed)
            .map_err(Denial::Installation)?;
        let authorization =
            access.authorize_observation_view(&execution, native, installed.installed_view())?;
        let contracts = installed.retain_compiled_contracts_for_admission();
        predecessor_access.require::<S, R>()?;
        let (prior, input) =
            access.select_predecessor_input(&execution, native, scope, partition)?;
        let limit = match (contracts.projection_work_budget(), remaining) {
            (Some(contract), Some(remaining)) => Some(contract.min(remaining)),
            (limit, None) | (None, limit) => limit,
        };
        let read = read_slot.take().expect("predecessor projection runs once");
        let completed = authority.project_borrowed_input(
            &execution,
            native,
            &input,
            limit,
            |reader| {
                let (output, facts) =
                    authority.inspect_reader(reader, Some(contracts), None, |reader| {
                        read(&mut BeforeDecisionReader { reader })
                    });
                (output, facts, reader.work_budget.consumed())
            },
            retention,
        );
        // Always recheck C, even when the consumer or projection refuses.
        access.observation_control(&execution, native)?;
        Ok::<_, Denial>((
            prior,
            input,
            authorization,
            completed.map_err(Denial::Projection)?,
        ))
    };
    let (prior, input, authorization, completed) = match reader_self.reader.native.committed_runtime()
    {
        Some(native) => project(native)?,
        // Scoped Native assembly holds its preparation port, not Query's
        // Native transport. This is the sole ordinary transport entrance.
        None => {
            let mut result = None;
            access.with_committed_runtime_dyn(&execution, &mut |native| {
                result = Some(project(native));
            });
            result.expect("committed runtime callback runs once")?
        }
    };
    let (output, facts, consumed) = completed.output;
    if !reader_self.reader.work_budget.consume(consumed) {
        return Err(Denial::Projection(
            WorthQueryInvariantProjectionDenial::work_budget_exceeded(completed.work),
        ));
    }
    reader_self.reader.work = reader_self
        .reader
        .work
        .checked_combined(completed.work)
        .ok_or_else(|| Denial::Decision("predecessor projection work overflow".into()))?;
    let (keys, _) = facts
        .into_expected(&execution)
        .map_err(|cause| Denial::Decision(format!("{cause:?}")))?;
    access.record_predecessor_read(WorthQueryHistoricalProducerRead::from_input(
        &prior,
        &input,
        R::IDENTIFIER,
        keys.into_historical_keys(),
    ));
    // Only C authorization becomes a current dependency. Old read keys and
    // old values are descriptive evidence and never rechecked at C.
    reader_self
        .reader
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
