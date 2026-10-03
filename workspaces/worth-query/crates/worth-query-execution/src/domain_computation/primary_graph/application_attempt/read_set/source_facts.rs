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
