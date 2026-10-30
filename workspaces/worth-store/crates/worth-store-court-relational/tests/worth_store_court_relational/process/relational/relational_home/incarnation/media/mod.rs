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
    open(&location, &execution).unwrap().close();
    let (counted, counters) =
        faulted_open(&location, &crate::media_faults::counters(), &execution).unwrap();
    let counted = counted.unwrap();
    let counts = crate::media_faults::counts(&counters);
    counted.certification_abort();
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
                first.close();
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
                use worth_store_relational::RelationalStoreHomeDescriptorRefusal as Descriptor;
                let expected_next = match outcome {
                    Ok(home) => {
                        assert_eq!(home.incarnation(), 2);
                        assert_eq!(home.home_identity(), identity);
                        home.certification_abort();
                        Some(3)
                    }
                    Err(worth_store_relational::RelationalStoreHomeOpenRefusal::Descriptor(
                        Descriptor::Publication(
                            Publication::Indeterminate(_) | Publication::CompletedWithoutRecord,
                        ),
                    )) => None,
                    Err(worth_store_relational::RelationalStoreHomeOpenRefusal::Readmission(_)) => {
                        Some(3)
                    }
                    Err(_) => Some(2),
                };
                let reopened = open(&location, &execution)
                    .unwrap_or_else(|r| panic!("{role:?} {ordinal} {directive:?}: {r:?}"));
                assert_eq!(reopened.home_identity(), identity);
                match expected_next {
                    Some(expected) => assert_eq!(
                        reopened.incarnation(),
                        expected,
                        "{role:?} {ordinal} {directive:?}: typed fate"
                    ),
                    None => assert!(
                        matches!(reopened.incarnation(), 2 | 3),
                        "only an indeterminate advance may select either fate"
                    ),
                }
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
