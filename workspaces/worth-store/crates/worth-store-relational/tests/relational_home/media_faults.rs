//! Media fault schedules for a home: open a home through the production path
//! with a certification media admission whose rules fire at the nth
//! identified operation of a role after their activation is armed, and count
//! each role's operations with rules that never fire.

use std::num::NonZeroU64;
use std::path::Path;

use worth_execution::ExecutionRequest;
use worth_relational::facade::schema::SchemaVersionId;
use worth_store::physical_runtime::certification::{
    CertificationMediaFaultActivation, MediaFaultDirective, MediaOperationRole,
};
use worth_store::physical_runtime::{FilesystemAccessPosture, FilesystemMediaAdmission};
use worth_store_relational::{RelationalStoreHome, RelationalStoreHomeRequest};

use super::workload;

/// One fault: the `nth` operation of `role` after arming meets `directive`.
pub(crate) type Fault = (MediaOperationRole, NonZeroU64, MediaFaultDirective);

pub(crate) fn fail_before() -> MediaFaultDirective {
    MediaFaultDirective::FailBefore {
        kind: std::io::ErrorKind::Other,
        raw_os_error: None,
    }
}

/// Both directives the sweep injects: an operation that fails with no effect,
/// and one whose effect happened but whose answer is lost.
pub(crate) fn directives() -> [MediaFaultDirective; 2] {
    [fail_before(), MediaFaultDirective::IndeterminateAfterEffect]
}

/// Open the closed home at `location`, declaring the `schema` it serves, with
/// one rule per fault. `None` when the schedule refuses a directive for its
/// role.
pub(crate) fn open_with_faults(
    location: &Path,
    schema: SchemaVersionId,
    faults: &[Fault],
    execution: &ExecutionRequest<'_, '_>,
) -> Option<(RelationalStoreHome, Vec<CertificationMediaFaultActivation>)> {
    let admission =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let activations: Vec<_> = faults
        .iter()
        .map(|_| authority.one_shot_activation())
        .collect();
    let rules = faults
        .iter()
        .zip(&activations)
        .map(|((role, nth, directive), activation)| {
            authority
                .rule(*role, 1, directive.clone())
                .for_nth_identified_operation_after_activation(activation.clone(), *nth)
        })
        .collect();
    let schedule = authority.schedule(rules).ok()?;
    let request = RelationalStoreHomeRequest::new(location, workload::runtime_builder(schema));
    let home = RelationalStoreHome::certification_open_with_media(
        request,
        admission.with_fault_schedule(schedule),
        execution,
    )
    .unwrap_or_else(|refusal| panic!("a closed home opens with a fault schedule: {refusal:?}"));
    Some((home, activations))
}

/// Rules that never fire, one per role, for counting operations.
pub(crate) fn counters() -> Vec<Fault> {
    MediaOperationRole::ALL
        .iter()
        .map(|role| (*role, NonZeroU64::MAX, fail_before()))
        .collect()
}

/// How many operations of each role ran since the counters were armed,
/// omitting roles that ran none.
pub(crate) fn counts(
    activations: &[CertificationMediaFaultActivation],
) -> Vec<(MediaOperationRole, u64)> {
    MediaOperationRole::ALL
        .iter()
        .zip(activations)
        .map(|(role, activation)| (*role, activation.matching_operation_count()))
        .filter(|(_, count)| *count > 0)
        .collect()
}

pub(crate) fn arm(activations: &[CertificationMediaFaultActivation]) {
    for activation in activations {
        activation.arm().expect("the activation arms");
    }
}
