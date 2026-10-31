//! The open-time incarnation decision under every identified media fault.
use crate::durable_run::open;
#[allow(unused_imports)]
use crate::incarnation::media::*;

#[test]
fn incarnation_advance_media_sweep_preserves_identity_and_selects_one_complete_advance() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let parent = tempfile::tempdir().unwrap();
    let location = parent.path().join("count");
    open(&location, &execution).unwrap().close(&execution);
    let (counted, counters) =
        faulted_open(&location, &crate::media_faults::counters(), &execution).unwrap();
    let counted = counted.unwrap();
    let counts = crate::media_faults::counts(&counters);
    counted.abort_for_test();
    assert!(counts.iter().any(|(role, _)| matches!(
        *role,
        worth_store::physical_runtime::certification::MediaOperationRole::SynchronizeFileData | worth_store::physical_runtime::certification::MediaOperationRole::SynchronizeFileState
    )));
    let mut cases = 0;
    for (role, count) in counts {
        for ordinal in 1..=count {
            for directive in crate::media_faults::directives() {
                let case = tempfile::tempdir().unwrap();
                let location = case.path().join("home");
                let first = open(&location, &execution).unwrap();
                let identity = first.home_identity();
                first.close(&execution);
                let Some((outcome, activations)) = faulted_open(
                    &location,
                    &[(
                        role,
                        std::num::NonZeroU64::new(ordinal).unwrap(),
                        directive.clone(),
                    )],
                    &execution,
                ) else {
                    continue;
                };
                assert!(
                    activations[0].is_consumed(),
                    "{role:?} {ordinal} {directive:?}"
                );
                use worth_store::physical_runtime::AnchoredRecordPublicationFailure as Publication;
                use worth_store_application::ApplicationHomeDescriptorDenial as Descriptor;
                let expected_next = match outcome {
                    Ok(home) => {
                        assert_eq!(home.incarnation().get(), 2);
                        assert_eq!(home.home_identity(), identity);
                        home.abort_for_test();
                        Some(3)
                    }
                    Err(worth_store_application::ApplicationHomeOpenRefusal::Descriptor(
                        Descriptor::Publication(
                            Publication::Indeterminate(_) | Publication::CompletedWithoutRecord,
                        ),
                    )) => None,
                    // These stages consume descriptor admission; even a later
                    // native/manifest refusal retains the durable advance.
                    Err(
                        worth_store_application::ApplicationHomeOpenRefusal::Checkpoint(_)
                        | worth_store_application::ApplicationHomeOpenRefusal::RuntimeConstruction(_)
                        | worth_store_application::ApplicationHomeOpenRefusal::Relational(_),
                    ) => Some(3),
                    Err(
                        worth_store_application::ApplicationHomeOpenRefusal::Physical(_)
                        | worth_store_application::ApplicationHomeOpenRefusal::Execution(_)
                        | worth_store_application::ApplicationHomeOpenRefusal::Descriptor(_),
                    ) => Some(2),
                };
                let reopened = open(&location, &execution)
                    .unwrap_or_else(|r| panic!("{role:?} {ordinal} {directive:?}: {r:?}"));
                assert_eq!(reopened.home_identity(), identity);
                match expected_next {
                    Some(expected) => assert_eq!(
                        reopened.incarnation().get(),
                        expected,
                        "{role:?} {ordinal} {directive:?}: typed fate"
                    ),
                    None => assert!(
                        matches!(reopened.incarnation().get(), 2 | 3),
                        "only an indeterminate advance may select either fate"
                    ),
                }
                let selected = reopened.incarnation().get();
                reopened.close(&execution);
                let again = open(&location, &execution).unwrap();
                assert_eq!(again.incarnation().get(), selected + 1);
                again.close(&execution);
                cases += 1;
            }
        }
    }
    assert!(cases > 0);
    eprintln!("incarnation media sweep: {cases} cases");
}
