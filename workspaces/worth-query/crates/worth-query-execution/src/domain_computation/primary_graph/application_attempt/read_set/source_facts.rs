use std::collections::BTreeMap;

use super::super::WorthQueryApplicationFactStorageKey;
use super::{denial, WorthQueryApplicationSnapshotLease};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
};

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
    mut dependent: BTreeMap<WorthQueryApplicationFactStorageKey, WorthQueryApplicationObservedFact>,
    operation: &str,
) -> Result<Vec<WorthQueryApplicationObservedFact>, WorthQueryApplicationAttemptDenial> {
    // The projection already keyed and deduplicated its retained observations.
    // Move that allocation through the handoff; only admitted sources need keys.
    for fact in admitted {
        let locator = fact.dependency_key();
        match dependent.entry(locator) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(fact);
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                if !entry.get_mut().merge_same_source_fact(fact) {
                    return Err(denial(
                        WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch,
                        format!("{operation}: source facts conflict at {:?}", entry.key()),
                    ));
                }
            }
        }
    }
    Ok(dependent.into_values().collect())
}
