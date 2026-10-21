//! Once-prepared scalar values retain validation and memory into every sink.
mod installation;
use super::super::{
    application_contribution::{QueryMemoryReservation, QueryRequestExecution},
    HandlerExecutionDenial, WorthQueryAdvancementDenial, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
    WorthQueryManagedComputationResourceDenial,
};
pub(super) use installation::install;
use std::sync::Arc;
use worth_foundational::{expression_api::ExpressionValue, facade::AspectValue};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyValueProgram, ApplicationOperationBodyValueSnapshot,
};
pub(in crate::domain_computation::primary_graph) struct InstalledBodyValue {
    pub(in crate::domain_computation::primary_graph) program: ApplicationOperationBodyValueProgram,
    bindings: Vec<worth_query_installation::facade::WorthQueryInstalledApplicationValueBinding>,
}
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct PreparedBodyValues(Arc<RetainedValues>);
struct RetainedValues {
    values: Vec<(String, AspectValue)>,
    snapshot: ApplicationOperationBodyValueSnapshot,
    _reservation: QueryMemoryReservation,
}
impl PreparedBodyValues {
    pub(in crate::domain_computation::primary_graph) fn value(&self, name: &str) -> &AspectValue {
        &self
            .0
            .values
            .iter()
            .find(|(key, _)| key == name)
            .expect("installed value reference")
            .1
    }
    pub(in crate::domain_computation::primary_graph) fn snapshot(
        &self,
    ) -> &ApplicationOperationBodyValueSnapshot {
        &self.0.snapshot
    }
}
pub(in crate::domain_computation::primary_graph) fn prepare<
    S: worth_query_installation::facade::ApplicationSchema,
    O,
    E,
>(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, S, O>,
    scope: &WorthQueryInvariantEntityIdentity<S, E>,
    programs: &[InstalledBodyValue],
    execution: &QueryRequestExecution<'_>,
) -> Result<Option<PreparedBodyValues>, HandlerExecutionDenial> {
    if programs.is_empty() {
        return Ok(None);
    }
    let bytes = programs
        .iter()
        .try_fold(
            size_of::<RetainedValues>() + 4 * size_of::<usize>(),
            |sum, value| {
                sum.checked_add(2 * value.program.identity().len())?
                    .checked_add(
                        size_of::<(String, AspectValue)>() + size_of::<(String, ExpressionValue)>(),
                    )
            },
        )
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(overflow)?;
    let reservation = execution.reserve(bytes).map_err(|cause| {
        HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
    })?;
    let mut values = Vec::with_capacity(programs.len());
    let mut snapshot = Vec::with_capacity(programs.len());
    for installed in programs {
        let program = &installed.program;
        let result = super::expression::evaluate(
            reader,
            scope,
            program.operands(),
            program.expression(),
            execution,
            super::expression::Purpose::Value(program.identity()),
        )?;
        let value = result
            .as_integer()
            .and_then(|value| u64::try_from(value).ok())
            .map(AspectValue::UInt64)
            .ok_or_else(|| invalid(program.identity()))?;
        execution.decision_stage(|context| -> Result<bool, WorthQueryAdvancementDenial> {
            for binding in &installed.bindings {
                let profile = match binding.native_input_resources(&value) {
                    Ok(profile) => profile,
                    Err(worth_query_declaration::facade::application_schema::ApplicationNativeInputAdmissionDenial::InvalidValue) => return Ok(false),
                    Err(worth_query_declaration::facade::application_schema::ApplicationNativeInputAdmissionDenial::CapacityOverflow) => return Err(WorthQueryAdvancementDenial::Resource(WorthQueryManagedComputationResourceDenial::CapacityOverflow)),
                };
                if !super::input::validate_admitted(execution, context, profile, || binding.validate_encoded(&value).is_ok())? { return Ok(false); }
            }
            Ok(true)
        }).map_err(HandlerExecutionDenial::new)?.map_err(HandlerExecutionDenial::new)?
            .then_some(()).ok_or_else(|| invalid(program.identity()))?;
        snapshot.push((program.identity().to_owned(), result));
        values.push((program.identity().to_owned(), value));
    }
    Ok(Some(PreparedBodyValues(Arc::new(RetainedValues {
        values,
        snapshot: ApplicationOperationBodyValueSnapshot::from_untrusted_values(snapshot),
        _reservation: reservation,
    }))))
}
fn invalid(name: &str) -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryApplicationAttemptDenial::new(
        Kind::InvalidEffectValue,
        name,
    ))
}
fn overflow() -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(
        WorthQueryManagedComputationResourceDenial::CapacityOverflow,
    ))
}
