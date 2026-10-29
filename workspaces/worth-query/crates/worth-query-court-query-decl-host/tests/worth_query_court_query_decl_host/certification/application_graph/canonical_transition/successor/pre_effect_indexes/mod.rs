pub(crate) use worth_query_testkit_certification_canonical::facade::certification::canonical_transition::successor::pre_effect_indexes::*;

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
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
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
