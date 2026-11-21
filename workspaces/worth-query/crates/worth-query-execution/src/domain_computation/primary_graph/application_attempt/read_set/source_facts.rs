pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::read_set::source_facts::*;
use super::*;
use crate::domain_computation::primary_graph::StorageControl;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub(in crate::domain_computation::primary_graph) fn validate_projected_source_facts<
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        use crate::domain_computation::provider_session::ManagedGraphWorkSessionPrimaryExt;
        let lease = admission
            .graph_work()
            .mutation_lease_typed()
            .ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::CurrentAuthorityDenied,
                admission.operation(),
            )
        })?;
        compare_source_facts(execution, admission, lease)
    }
}

pub fn merge_source_facts(
    admitted: Vec<WorthQueryApplicationObservedFact>,
    dependent: Option<RetainedSourceFacts>,
    operation: &str,
    control: StorageControl<'_, '_>,
) -> Result<SourceFacts, WorthQueryApplicationAttemptDenial> {
    control
        .check_live()
        .map_err(|denial| denial.into_attempt_denial(operation))?;
    if admitted.is_empty() {
        return Ok(SourceFacts::Projected(dependent, None));
    }
    let mut dependent = match dependent {
        Some(facts) => AuthoringSourceFacts::from_retained(facts, control),
        None => AuthoringSourceFacts::new(control),
    }
    .map_err(|denial| denial.into_attempt_denial(operation))?;
    let mut source = AuthoringSourceFacts::new(control)
        .map_err(|denial| denial.into_attempt_denial(operation))?;
    for fact in admitted {
        if let Some(fact) = dependent
            .merge_existing(fact, control)
            .map_err(|denial| denial.into_attempt_denial(operation))?
        {
            source
                .capture(fact, control)
                .map_err(|denial| denial.into_attempt_denial(operation))?;
        }
    }
    let handler = dependent
        .finish(control)
        .map_err(|denial| denial.into_attempt_denial(operation))?;
    let source = source
        .finish(control)
        .map_err(|denial| denial.into_attempt_denial(operation))?;
    Ok(SourceFacts::Projected(Some(handler), Some(source)))
}
