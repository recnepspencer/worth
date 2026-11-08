//! Corruptions land through ordinary Native admission BEFORE actual World genesis.
use super::*;
use worth_foundational::facade::{AspectValue, InternedString};

enum RecordAdmissionOwner {
    NativeFinalization,
    QueryAttachment,
}

#[test]
fn selected_completion_and_activation_failures_keep_owned_home_without_initializing() {
    for (case, expected, owner) in [
        (
            "missing-completion",
            "Home root must contain exactly one bootstrap completion record",
            RecordAdmissionOwner::QueryAttachment,
        ),
        (
            "unsupported-format",
            "Home bootstrap completion format is unsupported",
            RecordAdmissionOwner::NativeFinalization,
        ),
        (
            "foreign-binding",
            "Home completion binding is foreign:",
            RecordAdmissionOwner::NativeFinalization,
        ),
        (
            "missing-activation",
            "recovered branch contains 0 program activation records, expected one",
            RecordAdmissionOwner::QueryAttachment,
        ),
        (
            "foreign-activation",
            "Home root does not activate its admitted program",
            RecordAdmissionOwner::QueryAttachment,
        ),
    ] {
        let path = tempfile::tempdir().unwrap();
        let lease = checkpoint_capture::lease(128 * 1024 * 1024);
        let execution = ExecutionRequest::leased(&lease);
        let callbacks = AtomicUsize::new(0);
        with_bootstrap_advancement_from_request(execution, |phase| {
            let (mut query, mut home, _) = prepared(path.path(), &phase, true, &callbacks);
            query.publish_home_query_seed(&phase).unwrap();
            let native = home.relational().runtime();
            let completion =
                administration::rows(native, &execution, "worth-query-bootstrap-publication");
            let [completion] = completion.as_slice() else {
                panic!("one actual completion")
            };
            match case {
                "missing-completion" => {
                    administration::delete(native, &execution, completion.entity_id)
                }
                "unsupported-format" => administration::update(
                    native,
                    &execution,
                    completion.entity_id,
                    "bootstrap-publication",
                    "format-version",
                    AspectValue::UInt64(99),
                ),
                "foreign-binding" => {
                    let AspectValue::String(InternedString::Raw(binding)) =
                        administration::field(completion, "bootstrap-publication", "binding")
                    else {
                        panic!("actual portable binding")
                    };
                    let mut foreign = binding.as_bytes().to_vec();
                    // Change one valid lowercase package digest digit, preserving grammar.
                    foreign[9] = if foreign[9] == b'0' { b'1' } else { b'0' };
                    administration::update(
                        native,
                        &execution,
                        completion.entity_id,
                        "bootstrap-publication",
                        "binding",
                        AspectValue::String(String::from_utf8(foreign).unwrap().into()),
                    );
                }
                "missing-activation" | "foreign-activation" => {
                    let rows =
                        administration::rows(native, &execution, "worth-query-program-activation");
                    let [activation] = rows.as_slice() else {
                        panic!("one actual activation")
                    };
                    if case == "missing-activation" {
                        administration::delete(native, &execution, activation.entity_id);
                    } else {
                        administration::update(
                            native,
                            &execution,
                            activation.entity_id,
                            "activation",
                            "program-revision",
                            AspectValue::String("00".repeat(32).into()),
                        );
                    }
                }
                _ => unreachable!(),
            }
            let root = home.select_query_root(&execution).unwrap();
            assert!(home
                .relational()
                .runtime()
                .history()
                .branch_head_for_observation(&root.basis().relational_basis().observation())
                .unwrap()
                .is_some());
            drop(root);
            drop(query);
            close_world(home, &execution);
        })
        .unwrap();
        assert_eq!(callbacks.load(Ordering::SeqCst), 1);
        let refused = reopen_refusal(path.path(), execution);
        match owner {
            RecordAdmissionOwner::NativeFinalization => {
                let WorthQueryApplicationHomeOpenRefused::ExistingNativeFinalization {
                    query, ..
                } = &refused
                else {
                    panic!("{case} must reach Native completion admission: {refused:?}");
                };
                assert_eq!(query.reason().kind(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected);
                assert!(
                    query.reason().subject().contains(expected),
                    "{case}: {:?}",
                    query.reason()
                );
            }
            RecordAdmissionOwner::QueryAttachment => {
                let WorthQueryApplicationHomeOpenRefused::ExistingQueryAttachment { query, .. } =
                    &refused
                else {
                    panic!("{case} must reach selected Query record admission: {refused:?}");
                };
                assert!(
                    query.detail().contains(expected),
                    "{case}: {}",
                    query.detail()
                );
            }
        }
        refused
            .close(&execution)
            .unwrap_or_else(|refused| panic!("selected owning close: {refused:?}"));
    }
}
