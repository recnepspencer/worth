use super::super::retained_decision_facts::{
    AuthoringSourceFacts, RetainedSourceFacts, StorageControl, StoreDenial,
};
use super::{denial, WorthQueryApplicationSnapshotLease};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
};

pub mod storage;
pub use storage::SourceFacts;

pub fn validate_source_facts<Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    lease: &WorthQueryApplicationSnapshotLease,
) -> Result<Vec<WorthQueryApplicationObservedFact>, WorthQueryApplicationAttemptDenial> {
    compare_source_facts(execution, admission, lease)?;
    Ok(admission.take_source_facts())
}

pub fn compare_source_facts<Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    lease: &WorthQueryApplicationSnapshotLease,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    if let Some(expected) = lease.applied_source_qualification() {
        // Source expansion was sealed from this exact immutable Native image.
        // Its qualification is separate from the before authorization lease.
        return if lease.applied_qualification() == Some(expected) {
            Ok(())
        } else {
            Err(denial(
                WorthQueryApplicationAttemptDenialKind::SourceChanged,
                admission.operation(),
            ))
        };
    }
    for fact in admission.source_facts() {
        super::super::super::fact_verification_observation::source_admission();
        if let Some(stop) = admission.publication_request().interruption() {
            return Err(
                StoreDenial::RequestInterruption(stop).into_attempt_denial(admission.operation())
            );
        }
        let fresh = lease.handle().with_runtime(execution, |runtime| {
            fact.remains_equal_at(runtime, execution, lease.certification_basis())
        })?;
        if !fresh {
            let kind = if matches!(fact, WorthQueryApplicationObservedFact::SourceEntity { .. }) {
                WorthQueryApplicationAttemptDenialKind::SourceRetired
            } else {
                WorthQueryApplicationAttemptDenialKind::SourceChanged
            };
            return Err(denial(kind, admission.operation()));
        }
    }
    Ok(())
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
