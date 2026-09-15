use crate::runtime::tests::support::*;
use crate::runtime::worker_host::{
    publish_definition_envelope_into_worker_runtime, WorkerObservationDeliveryAttachRequest,
    WorkerPortableGraphPublication, WorkerRuntimeShell,
};

pub(in crate::runtime::tests::worker_runtime) fn portable_counter_publication(
) -> WorkerPortableGraphPublication {
    WorkerPortableGraphPublication {
        policy: RuntimePolicySpec::default(),
        sources: vec![SourceSpec {
            id: "counter".to_owned(),
            initial: SignalValue::Number(1.0),
            produces_aspects: None,
        }],
        recipes: vec![RecipeSpec {
            id: "doubleCounter".to_owned(),
            reads: vec![RecipeReadSpec::LegacyId("counter".to_owned())],
            expr: Expr::Sum {
                args: vec![read("counter"), read("counter")],
            },
            when: None,
            identity: Some(IdentitySpec::Exact),
            produces_aspects: None,
        }],
        output_ids: vec!["doubleCounter".to_owned()],
    }
}

pub(in crate::runtime::tests::worker_runtime) fn worker_shell_with_counter_graph(
) -> WorkerRuntimeShell {
    let mut worker_shell = WorkerRuntimeShell::new(RuntimePolicySpec::default()).unwrap();
    worker_shell
        .publish_graph(portable_counter_publication())
        .unwrap();
    worker_shell
}

pub(in crate::runtime::tests::worker_runtime) fn worker_shell_with_observed_counter(
) -> WorkerRuntimeShell {
    let mut worker_shell = worker_shell_with_counter_graph();
    worker_shell
        .attach_observation_delivery(double_counter_observation_attach_request())
        .unwrap();
    worker_shell
}

pub(in crate::runtime::tests::worker_runtime) fn set_counter(value: f64) -> Vec<TransactionOp> {
    vec![TransactionOp::Set {
        id: "counter".to_owned(),
        value: SignalValue::Number(value),
        aspect: None,
        aspects: None,
    }]
}

pub(in crate::runtime::tests::worker_runtime) fn double_counter_observation_attach_request(
) -> WorkerObservationDeliveryAttachRequest {
    WorkerObservationDeliveryAttachRequest {
        signal_id: "doubleCounter".to_owned(),
    }
}

/// Publishes `portable_counter_publication()` into a compatibility runtime
/// through the same definition-envelope path the worker host uses, so the
/// compatibility graph (including its public output marks) is the worker
/// graph by construction rather than a hand-copied approximation of it.
pub(in crate::runtime::tests::worker_runtime) fn define_portable_counter_graph(
    runtime: &mut RuntimeCore,
) {
    let summary = publish_definition_envelope_into_worker_runtime(
        runtime,
        portable_counter_publication().into_definition_envelope(),
    )
    .unwrap();
    assert_eq!(summary.published_source_count, 1);
    assert_eq!(summary.published_recipe_count, 1);
    assert!(runtime.is_web_output_signal("doubleCounter"));
}
