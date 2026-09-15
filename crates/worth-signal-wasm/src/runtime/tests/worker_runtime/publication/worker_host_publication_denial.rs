use crate::runtime::worker_host::{WorkerPortableGraphPublication, WorkerRuntimeShell};

use crate::recipe::model::{AspectSelectionSpec, RecipeReadSignalSpec};
use crate::runtime::tests::support::*;

#[test]
fn worker_runtime_shell_denies_callback_definition_envelope_publication() {
    let mut compatibility_runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    compatibility_runtime
        .define_source(SourceSpec {
            id: "counter".to_owned(),
            initial: SignalValue::Number(1.0),
            produces_aspects: None,
        })
        .unwrap();
    compatibility_runtime
        .define_web_computed_native_callback(
            "callbackDouble".to_owned(),
            Box::new(|| {
                Ok(compute_callbacks::ComputeCallbackInvocationResult {
                    value: SignalValue::Number(2.0),
                    captured_read_ids: vec!["counter".to_owned()],
                    captured_host_capability_reads: Vec::new(),
                    runtime_read_breadth: 1,
                    return_serialization_breadth: 1,
                })
            }),
        )
        .unwrap();

    let definition_envelope = compatibility_runtime.export_definitions().unwrap();
    let mut worker_shell = WorkerRuntimeShell::new(RuntimePolicySpec::default()).unwrap();
    let err = worker_shell
        .publish_definition_envelope(definition_envelope)
        .unwrap_err();

    assert_eq!(
        err.code,
        "workerRuntimePublicationRequiresPortableDefinitions"
    );
    assert!(err.message.contains("callbackDouble"));
}

#[test]
fn worker_portable_publication_rejects_unknown_reads_without_partial_sources() {
    let mut worker_shell = WorkerRuntimeShell::new(RuntimePolicySpec::default()).unwrap();

    let err = worker_shell
        .publish_graph(WorkerPortableGraphPublication {
            policy: RuntimePolicySpec::default(),
            sources: vec![SourceSpec {
                id: "base".to_owned(),
                initial: SignalValue::Number(1.0),
                produces_aspects: None,
            }],
            recipes: vec![RecipeSpec {
                id: "derived".to_owned(),
                reads: vec![RecipeReadSpec::LegacyId("missing".to_owned())],
                expr: read("missing"),
                when: None,
                identity: None,
                produces_aspects: None,
            }],
            output_ids: Vec::new(),
        })
        .unwrap_err();

    assert!(err.message.contains("missing"));
    assert!(worker_shell.read_value("base").is_err());
    assert!(worker_shell.read_value("derived").is_err());
}

#[test]
fn worker_portable_publication_rejects_later_recipe_family_reads_without_partial_sources() {
    let mut worker_shell = WorkerRuntimeShell::new(RuntimePolicySpec::default()).unwrap();

    let err = worker_shell
        .publish_definition_envelope(crate::runtime::adapters::RuntimeDefinitionEnvelope {
            policy: RuntimePolicySpec::default(),
            sources: vec![SourceSpec {
                id: "base".to_owned(),
                initial: SignalValue::Number(1.0),
                produces_aspects: None,
            }],
            recipes: Vec::new(),
            source_families: Vec::new(),
            recipe_families: vec![
                KeyedRecipeFamilySpec {
                    family_id: "derivedFamily".to_owned(),
                    reads: vec![RecipeFamilyReadSpec::Keyed {
                        family_id: "laterFamily".to_owned(),
                        scope: None,
                        aspects: Default::default(),
                    }],
                    expr: read("base"),
                    when: None,
                    identity: None,
                    produces_aspects: None,
                },
                KeyedRecipeFamilySpec {
                    family_id: "laterFamily".to_owned(),
                    reads: Vec::new(),
                    expr: number(2.0),
                    when: None,
                    identity: None,
                    produces_aspects: None,
                },
            ],
            worker_public_output_ids: Vec::new(),
            unavailable_callbacks: Vec::new(),
        })
        .unwrap_err();

    assert!(err.message.contains("laterFamily"));
    assert!(worker_shell.read_value("base").is_err());
}

#[test]
fn worker_portable_publication_rejects_invalid_recipe_aspect_without_partial_sources() {
    let mut worker_shell = WorkerRuntimeShell::new(RuntimePolicySpec::default()).unwrap();

    let err = worker_shell
        .publish_graph(WorkerPortableGraphPublication {
            policy: RuntimePolicySpec::default(),
            sources: vec![SourceSpec {
                id: "base".to_owned(),
                initial: SignalValue::Number(1.0),
                produces_aspects: None,
            }],
            recipes: vec![RecipeSpec {
                id: "derived".to_owned(),
                reads: vec![RecipeReadSpec::Signal(RecipeReadSignalSpec {
                    id: "base".to_owned(),
                    scope: None,
                    aspects: AspectSelectionSpec {
                        aspect: Some(255),
                        aspects: None,
                    },
                })],
                expr: read("base"),
                when: None,
                identity: None,
                produces_aspects: None,
            }],
            output_ids: Vec::new(),
        })
        .unwrap_err();

    assert!(err.message.contains("out of range"));
    assert!(worker_shell.read_value("base").is_err());
    assert!(worker_shell.read_value("derived").is_err());
}

#[test]
fn definition_envelope_publication_marks_the_envelope_public_outputs() {
    let mut exporter = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    exporter
        .define_source(SourceSpec {
            id: "counter".to_owned(),
            initial: SignalValue::Number(1.0),
            produces_aspects: None,
        })
        .unwrap();
    exporter
        .define_web_output(
            "doubleCounter".to_owned(),
            RecipeSpec {
                id: "doubleCounter".to_owned(),
                reads: vec![RecipeReadSpec::LegacyId("counter".to_owned())],
                expr: Expr::Sum {
                    args: vec![read("counter"), read("counter")],
                },
                when: None,
                identity: Some(IdentitySpec::Exact),
                produces_aspects: None,
            },
        )
        .unwrap();
    exporter
        .define_recipe(RecipeSpec {
            id: "privateTriple".to_owned(),
            reads: vec![RecipeReadSpec::LegacyId("counter".to_owned())],
            expr: Expr::Sum {
                args: vec![read("counter"), read("counter"), read("counter")],
            },
            when: None,
            identity: Some(IdentitySpec::Exact),
            produces_aspects: None,
        })
        .unwrap();
    let envelope = exporter.export_definitions().unwrap();
    assert_eq!(
        envelope.worker_public_output_ids,
        vec!["doubleCounter".to_owned()]
    );

    let mut imported = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    imported
        .publish_callback_free_definition_envelope(envelope)
        .unwrap();
    assert!(imported.is_web_output_signal("doubleCounter"));
    assert!(!imported.is_web_output_signal("privateTriple"));
    assert_eq!(
        imported.export_definitions().unwrap().worker_public_output_ids,
        vec!["doubleCounter".to_owned()]
    );

    // Same transaction, same committed truth: the published output is
    // standing demand in both runtimes, so both settle it at commit.
    let transaction = vec![TransactionOp::Set {
        id: "counter".to_owned(),
        value: SignalValue::Number(4.0),
        aspect: None,
        aspects: None,
    }];
    exporter.apply_transaction(transaction.clone()).unwrap();
    imported.apply_transaction(transaction).unwrap();
    assert_eq!(
        exporter.peek_value("doubleCounter").unwrap(),
        SignalValue::Number(8.0)
    );
    assert_eq!(
        imported.peek_value("doubleCounter").unwrap(),
        SignalValue::Number(8.0)
    );
    let exporter_digest = exporter
        .branch_state_proof(exporter.current_branch().id.0)
        .unwrap()
        .state_digest;
    let imported_digest = imported
        .branch_state_proof(imported.current_branch().id.0)
        .unwrap()
        .state_digest;
    assert_eq!(exporter_digest, imported_digest);
}
