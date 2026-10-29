pub(crate) use worth_query_testkit_query::facade::query::suite::operation_invalidation::*;

#[test]
fn one_owner_epoch_fans_exact_evidence_to_two_lease_deltas_in_k_plus_l_work() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, subject, candidate) =
        shared_native_leases(request_execution, "invalidation-fanout");
    consume_empty_epoch(&mut workspace, &subject, &candidate);
    workspace
        .insert(request_execution, "Vertex", |mutation| {
            mutation.aspect("identity.id", "invalidation-update")
        })
        .unwrap();

    let subject_delivery = subject.drain(&mut workspace).unwrap();
    let candidate_delivery = candidate.drain(&mut workspace).unwrap();
    assert!(subject_delivery.shares_invalidation_epoch_with(&candidate_delivery));
    assert!(subject_delivery.retains_same_impact_as(&candidate_delivery));
    assert_eq!(
        subject_delivery
            .invalidation_epoch_counters()
            .fanout_targets,
        2
    );

    let subject_delta = subject
        .consumer_invalidation_delta(subject_delivery)
        .unwrap();
    let candidate_delta = candidate
        .consumer_invalidation_delta(candidate_delivery)
        .unwrap();
    assert!(subject_delta.shares_epoch_with(&candidate_delta));
    assert!(subject_delta.retains_same_impact_as(&candidate_delta));
    assert!(subject_delta.retains_same_compatibility_evidence_as(&candidate_delta));
    assert_eq!(
        subject_delta.disposition(),
        domain::WorthQueryConsumerInvalidationDisposition::LocalPatch
    );
    assert_eq!(subject_delta.affected_native_keys().len(), 1);
    assert_eq!(subject_delta.epoch_counters().capability_index_lookups, 1);
    assert_eq!(subject_delta.epoch_counters().delivery_batches_visited, 1);
    assert_eq!(subject_delta.epoch_counters().mutation_deltas_visited, 1);
    assert_eq!(subject_delta.epoch_counters().touched_aspects_visited, 1);
    assert_eq!(subject_delta.counters().targeted_lease_deliveries, 1);
    assert_eq!(candidate_delta.counters().targeted_lease_deliveries, 1);
    assert_eq!(subject_delta.counters().native_key_index_lookups, 1);
    assert_eq!(subject_delta.counters().native_path_index_probes, 0);
    assert_eq!(subject_delta.counters().targeted_native_key_visits, 1);
    assert_eq!(
        subject_delta.counters().native_key_overlap_deduplications,
        0
    );

    let subject_projection = subject_delta.foundational_projection();
    let candidate_projection = candidate_delta.foundational_projection();
    assert_eq!(subject_projection, candidate_projection);
    assert_eq!(
        subject_projection.provenance().freshness_posture(),
        worth_foundational::facade::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained
    );
    assert_eq!(
        subject_projection.locality(),
        domain::WorthQueryConsumerInvalidationLocality::DeclaredNativeKeys
    );
    assert_eq!(subject_projection.scopes().len(), 1);
    let path = CanonicalFieldPath::single(FieldKey::new("id").unwrap());
    let aspect = AspectKey::new("identity").unwrap();
    let expected_locator = AspectValueLocator::struct_field(AspectFieldLocator::from_aspect(
        AspectLocator::new(LocatorAuthority::Projected, aspect.clone()),
        path.clone(),
    ));
    let expected_mask = AspectMaskLocator::<ProjectionMask>::projection(
        LocatorAuthority::Projected,
        aspect,
        &AspectMask::new([path]),
    );
    assert_eq!(subject_projection.scopes()[0].locator(), &expected_locator);
    assert_eq!(subject_projection.scopes()[0].mask(), &expected_mask);
    let admitted = match subject.admit_consumer_invalidation_delta(subject_delta, &workspace) {
        Ok(admitted) => admitted,
        Err(_) => panic!("current subject delta did not readmit"),
    };
    let artifact =
        match admitted.materialize_foundational_projection(&workspace, materialized_profile()) {
            Ok(artifact) => artifact,
            Err(_) => panic!("current admitted delta did not materialize"),
        };
    assert_eq!(
        artifact.role(),
        FoundationalBoundaryArtifactRole::DerivedProjection
    );
    assert_eq!(
        artifact.source(),
        FoundationalBoundaryMaterializationSource::NativeAuthority
    );
    assert_eq!(
        artifact
            .surface()
            .payload()
            .provenance()
            .freshness_posture(),
        worth_foundational::facade::FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained
    );
}
