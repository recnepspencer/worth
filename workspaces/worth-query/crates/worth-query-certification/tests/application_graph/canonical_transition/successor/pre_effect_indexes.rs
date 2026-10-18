//! A real cold bootstrap index cannot cross codec adoption's effect boundary.
use super::*;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};

#[test]
fn missing_native_bootstrap_index_generation_refuses_before_codec_authoring() {
    let checkpoint = checkpoint_without_bootstrap_generations();
    let original_bytes = checkpoint.bytes().to_vec();
    let retained = checkpoint.clone();
    let predecessor = installation::WorthQueryCheckpointProgramPredecessor::new(
        &super::super::host::program().revision().to_string(),
    )
    .unwrap();
    output::reset_decodes();
    let mut callbacks = 0;
    let denial = installation::in_memory_rostered_program_from_checkpoint_with_transition(
        host::program(),
        installation::WorthQueryApplicationProgramRoster::new(),
        schema::CanonicalSchema::declaration().unwrap(),
        ((),),
        host::limits(),
        checkpoint,
        predecessor,
        installation::WorthQueryCheckpointTransitionResources::bounded(4096, 32, 16384).unwrap(),
        CapturePolicy::SystemAllocation,
        |_, _| {
            callbacks += 1;
            Ok(())
        },
    )
    .err()
    .expect("an actually absent required bootstrap generation must refuse adoption");
    let installation::WorthQueryInMemoryApplicationDenial::Graph(denial) = denial else {
        panic!("missing source index must refuse before native performance: {denial:?}");
    };
    assert_eq!(denial.kind(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected);
    assert!(
        denial
            .subject()
            .contains("checkpoint omitted a bootstrap identity-index generation"),
        "unexpected index refusal: {denial:?}"
    );
    assert_eq!(
        callbacks, 0,
        "source index eligibility must precede authoring"
    );
    assert_eq!(
        output::decodes(),
        0,
        "V2 must never interpret the V1 derived payload"
    );
    assert_eq!(
        retained.bytes(),
        original_bytes,
        "rejected admission preserves its exact input checkpoint"
    );
}

fn checkpoint_without_bootstrap_generations() -> installation::WorthQueryApplicationCheckpoint {
    let application = super::super::host::install(model::authored_value()).unwrap();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let runtime = application.runtime();
    let request = runtime
        .request(&principal, &scope)
        .on_branch(runtime.current_world());
    let mut demand = request
        .demand(super::super::producer::Demand)
        .start_in_program::<_, super::super::program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.settle(&request).unwrap()
    else {
        panic!("the real V1 producer must create the affected derived output");
    };
    assert!(settled.application_commit_receipt().is_some());
    drop(demand);
    runtime
        .with_host_advancement(|phase| {
            let released = runtime
                .release_primary_index_generations_for_test(&phase)
                .unwrap();
            assert!(
                released > 0,
                "the fixture must evict actual installed bootstrap generations"
            );
            let (checkpoint, sections) = runtime
                .capture_application_checkpoint_with_sections(
                    &phase,
                    CapturePolicy::SystemAllocation,
                )
                .unwrap();
            assert_eq!(sections.accepted_output_count(), 1);
            checkpoint
        })
        .unwrap()
}
