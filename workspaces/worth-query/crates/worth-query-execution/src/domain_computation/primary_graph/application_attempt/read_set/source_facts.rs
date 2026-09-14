use std::collections::BTreeMap;

use super::{denial, WorthQueryApplicationSnapshotLease};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
};

pub(super) fn validate_source_facts<Schema, Operation, Input, Scope>(
    admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    lease: &WorthQueryApplicationSnapshotLease,
) -> Result<Vec<WorthQueryApplicationObservedFact>, WorthQueryApplicationAttemptDenial> {
    let facts = admission.take_source_facts();
    for fact in &facts {
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
    dependent: Vec<WorthQueryApplicationObservedFact>,
    operation: &str,
) -> Result<Vec<WorthQueryApplicationObservedFact>, WorthQueryApplicationAttemptDenial> {
    let mut merged = BTreeMap::new();
    for fact in admitted.into_iter().chain(dependent) {
        let locator = fact.locator_identity();
        if merged
            .insert(locator, fact.clone())
            .is_some_and(|existing| existing != fact)
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch,
                operation,
            ));
        }
    }
    Ok(merged.into_values().collect())
}
