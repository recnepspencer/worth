//! Original admitted C supplies readonly numerical DATA; actual execution stays D.
use super::predecessor::BeforeDecisionReader;
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact as Fact,
    application_contribution::producer::observation_reads::{
        ProducerOriginalReadAccess, WorthQueryProducerObservationReadDenial as Denial,
    },
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationStructuredValueBinding,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationSchema, WorthQueryInstalledPackageIndex,
};
use crate::domain_computation::primary_graph::invariant_projection::WorthQueryApplicationInvariantProjectionAuthority;

pub trait OperationReaderOriginalExt<
    S: ApplicationSchema + 'static,
>
{
    fn observe_original<R, I, T>(
        &mut self,
        operation: ApplicationOperationRef<S, R, I>,
        read: impl FnOnce(&mut BeforeDecisionReader<'_, '_, '_, S, R>) -> T,
    ) -> Result<T, Denial>
    where
        R: ApplicationOperationMarkerIdentity<S> + 'static,
        R::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: 'static;
}

impl<S: ApplicationSchema + 'static, O> OperationReaderOriginalExt<S>
    for WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, S, O>
{
    fn observe_original<R, I, T>(
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
        let result = read_original(self, operation, read);
        if let Err(cause) = &result {
            access.record_observation_failure(cause);
        }
        result
    }
}

fn read_original<S: ApplicationSchema + 'static, O, R, I, T>(
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
    let product = reader_self
        .selected_product
        .ok_or(Denial::UndeclaredObservationRead)?;
    let snapshot = reader_self.reader.snapshot;
    let execution = *reader_self.reader.execution;
    let retention = reader_self.reader.retention_control;
    let remaining = reader_self.reader.work_budget.remaining();
    let (schema_any, authority_any, packages_any, _, original_any) =
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
    let original_access = original_any
        .and_then(|a| a.downcast_ref::<ProducerOriginalReadAccess>())
        .ok_or(Denial::UndeclaredObservationRead)?;
    let mut read_slot = Some(read);
    let mut project = |native: &worth_relational::facade::runtime::RelationalRuntime| {
        original_access.require::<S, R>()?;
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
        let limit = match (contracts.projection_work_budget(), remaining) {
            (Some(contract), Some(remaining)) => Some(contract.min(remaining)),
            (limit, None) | (None, limit) => limit,
        };
        let read = read_slot.take().expect("original projection runs once");
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
    let (authorization, completed) = match reader_self.reader.native.committed_runtime() {
        Some(native) => project(native)?,
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
        .ok_or_else(|| Denial::Decision("original projection work overflow".into()))?;
    // Validate C's own data locally. None of its value keys becomes a D expectation.
    facts
        .into_expected(&execution)
        .map_err(|cause| Denial::Decision(format!("{cause:?}")))?;
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
