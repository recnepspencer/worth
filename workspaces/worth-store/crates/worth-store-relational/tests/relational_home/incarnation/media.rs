//! The open-time incarnation decision under every identified media fault.
use super::super::{media_faults, schema_transition};
use super::*;
use std::num::NonZeroU64;
use worth_store::physical_runtime::certification::MediaOperationRole;
use worth_store::physical_runtime::{FilesystemAccessPosture, FilesystemMediaAdmission};
use worth_store_relational::{RelationalStoreHome, RelationalStoreHomeRequest};

fn faulted_open(
    location: &Path,
    faults: &[media_faults::Fault],
    execution: &ExecutionRequest<'_, '_>,
) -> Option<(
    Result<RelationalStoreHome, Refusal>,
    Vec<worth_store::physical_runtime::certification::CertificationMediaFaultActivation>,
)> {
    let media =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = media.fault_schedule_authority();
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
    let opened = RelationalStoreHome::certification_open_with_descriptor_observer(
        RelationalStoreHomeRequest::new(
            location,
            workload::runtime_builder(schema_transition::SOURCE),
        ),
        media.with_fault_schedule(schedule),
        |_| media_faults::arm(&activations),
        execution,
    );
    Some((opened, activations))
}

#[test]
fn incarnation_advance_media_sweep_preserves_identity_and_selects_one_complete_advance() {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let parent = tempfile::tempdir().unwrap();
    let location = parent.path().join("count");
    open(&location, &execution).unwrap().close();
    let (counted, counters) =
        faulted_open(&location, &media_faults::counters(), &execution).unwrap();
    counted.unwrap().close();
    let counts = media_faults::counts(&counters);
    assert!(counts
        .iter()
        .any(|(role, _)| *role == MediaOperationRole::FileSync));
    let mut cases = 0;
    for (role, count) in counts {
        for ordinal in 1..=count {
            for directive in media_faults::directives() {
                let case = tempfile::tempdir().unwrap();
                let location = case.path().join("home");
                let first = open(&location, &execution).unwrap();
                let identity = first.home_identity();
                first.close();
                let Some((outcome, activations)) = faulted_open(
                    &location,
                    &[(role, NonZeroU64::new(ordinal).unwrap(), directive.clone())],
                    &execution,
                ) else {
                    continue;
                };
                assert!(
                    activations[0].is_consumed(),
                    "{role:?} {ordinal} {directive:?}"
                );
                if let Ok(home) = outcome {
                    assert_eq!(home.incarnation(), 2);
                    assert_eq!(home.home_identity(), identity);
                    home.certification_abort();
                }
                let reopened = open(&location, &execution)
                    .unwrap_or_else(|r| panic!("{role:?} {ordinal} {directive:?}: {r:?}"));
                assert_eq!(reopened.home_identity(), identity);
                assert!(
                    matches!(reopened.incarnation(), 2 | 3),
                    "in-flight advance selected at most once"
                );
                let selected = reopened.incarnation();
                reopened.close();
                let again = open(&location, &execution).unwrap();
                assert_eq!(again.incarnation(), selected + 1);
                again.close();
                cases += 1;
            }
        }
    }
    assert!(cases > 0);
    eprintln!("incarnation media sweep: {cases} cases");
}
