use super::super::retained_decision_facts::{
    AuthoringSourceFacts, RetainedSourceFacts, StorageControl, StoreDenial,
};
use super::{denial, WorthQueryApplicationSnapshotLease};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
};

mod storage;
pub(super) use storage::SourceFacts;

#[cfg(test)]
mod indexed_selection;
#[cfg(test)]
mod merging;

pub(super) fn validate_source_facts<Schema, Operation, Input, Scope>(
    admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    lease: &WorthQueryApplicationSnapshotLease,
) -> Result<Vec<WorthQueryApplicationObservedFact>, WorthQueryApplicationAttemptDenial> {
    let facts = admission.take_source_facts();
    for fact in &facts {
        if let Some(stop) = admission.publication_request().interruption() {
            return Err(
                StoreDenial::RequestInterruption(stop).into_attempt_denial(admission.operation())
            );
        }
        let fresh = lease
            .handle()
            .with_runtime(|runtime| fact.remains_equal_in(runtime, lease.snapshot()));
        if !fresh {
            let kind = if matches!(fact, WorthQueryApplicationObservedFact::SourceEntity { .. }) {
                WorthQueryApplicationAttemptDenialKind::SourceRetired
            } else {
                WorthQueryApplicationAttemptDenialKind::SourceChanged
            };
            return Err(denial(kind, admission.operation()));
        }
    }
    Ok(facts)
}

pub(super) fn merge_source_facts(
    admitted: Vec<WorthQueryApplicationObservedFact>,
    dependent: Option<RetainedSourceFacts>,
    operation: &str,
    control: StorageControl<'_, '_>,
) -> Result<SourceFacts, WorthQueryApplicationAttemptDenial> {
    control
        .check_live()
        .map_err(|denial| denial.into_attempt_denial(operation))?;
    if admitted.is_empty() {
        return Ok(SourceFacts::Projected(dependent));
    }
    let mut dependent = match dependent {
        Some(facts) => AuthoringSourceFacts::from_retained(facts, control),
        None => AuthoringSourceFacts::new(control),
    }
    .map_err(|denial| denial.into_attempt_denial(operation))?;
    for fact in admitted {
        dependent
            .capture(fact, control)
            .map_err(|denial| denial.into_attempt_denial(operation))?;
    }
    let facts = dependent
        .finish(control)
        .map_err(|denial| denial.into_attempt_denial(operation))?;
    Ok(SourceFacts::Projected(Some(facts)))
}
