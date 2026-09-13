use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectLocator, AspectMask, AspectMaskLocator,
    AspectValueLocator, CanonicalFieldPath, FieldKey, FoundationalBoundaryArtifactRole,
    FoundationalBoundaryMaterializationSource, LocatorAuthority, ProjectionMask,
};
use worth_query::facade::domain;

use super::installed_operation_fixture::{
    consume_empty_invalidation_epoch as consume_empty_epoch,
    materialized_invalidation_profile as materialized_profile, shared_native_leases,
    shared_native_leases_with_invalidation,
};

#[test]
fn one_owner_epoch_fans_exact_evidence_to_two_lease_deltas_in_k_plus_l_work() {
    let (mut workspace, subject, candidate) = shared_native_leases("invalidation-fanout");
    consume_empty_epoch(&mut workspace, &subject, &candidate);
    workspace
        .insert("Vertex", |mutation| {
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

#[test]
fn consequence_authority_requires_the_exact_current_lease_and_stays_consumer_authored() {
    let (mut workspace, subject, candidate) = shared_native_leases("invalidation-readmission");
    consume_empty_epoch(&mut workspace, &subject, &candidate);
    workspace
        .insert("Vertex", |mutation| {
            mutation.aspect("identity.id", "readmit-update")
        })
        .unwrap();
    let delivery = subject.drain(&mut workspace).unwrap();
    let _peer = candidate.drain(&mut workspace).unwrap();
    let delta = subject.consumer_invalidation_delta(delivery).unwrap();

    let stop = match candidate.admit_consumer_invalidation_delta(delta, &workspace) {
        Err(stop) => stop,
        Ok(_) => panic!("foreign lease admitted the subject delta"),
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryConsumerInvalidationDeltaStopKind::ForeignOrStaleLease
    );
    let delta = stop.into_delta();
    let admitted = match subject.admit_consumer_invalidation_delta(delta, &workspace) {
        Ok(admitted) => admitted,
        Err(_) => panic!("exact current lease did not readmit its delta"),
    };
    #[derive(Debug, Eq, PartialEq)]
    enum UiAction {
        PatchMountedField,
    }
    #[derive(Debug, Eq, PartialEq)]
    enum CacheAction {
        EvictEntry,
    }
    let first = match admitted.attach_consumer_authored_consequence(
        &workspace,
        domain::WorthQueryConsumerInvalidationDisposition::LocalPatch,
        UiAction::PatchMountedField,
    ) {
        Ok(consequence) => consequence,
        Err(_) => panic!("exact UI patch consequence was denied"),
    };
    let second = match admitted.attach_consumer_authored_consequence(
        &workspace,
        domain::WorthQueryConsumerInvalidationDisposition::Reexecute,
        CacheAction::EvictEntry,
    ) {
        Ok(consequence) => consequence,
        Err(_) => panic!("explicitly widened cache consequence was denied"),
    };
    assert_eq!(first.consumer_authored(), &UiAction::PatchMountedField);
    assert_eq!(second.consumer_authored(), &CacheAction::EvictEntry);
    assert_eq!(
        first.required_disposition(),
        domain::WorthQueryConsumerInvalidationDisposition::LocalPatch
    );
    assert_eq!(
        second.admitted_disposition(),
        domain::WorthQueryConsumerInvalidationDisposition::Reexecute
    );
    assert!(first
        .authority()
        .is_same_current_authority_as(admitted.delta().authority()));
    assert!(second
        .authority()
        .is_same_current_authority_as(admitted.delta().authority()));
}

#[test]
fn empty_epoch_cannot_mint_an_invalidation_delta() {
    let (mut workspace, subject, candidate) = shared_native_leases("invalidation-empty");
    let subject_delivery = subject.drain(&mut workspace).unwrap();
    let _candidate_delivery = candidate.drain(&mut workspace).unwrap();
    let stop = match subject.consumer_invalidation_delta(subject_delivery) {
        Err(stop) => stop,
        Ok(_) => panic!("empty delivery minted an invalidation delta"),
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryConsumerInvalidationDeltaStopKind::NoSemanticDelivery
    );
    assert_eq!(stop.counters().lease_impact_readmission_attempts, 1);
    assert_eq!(stop.counters().semantic_delivery_checks, 1);
    assert_eq!(stop.counters().targeted_lease_deliveries, 0);
}

#[test]
fn unsupported_invalidation_posture_stops_before_delta_translation() {
    let (mut workspace, subject, candidate) = shared_native_leases_with_invalidation(
        "invalidation-support-stop",
        domain::WorthQueryConsumerSupportPosture::Deferred,
    );
    consume_empty_epoch(&mut workspace, &subject, &candidate);
    workspace
        .insert("Vertex", |mutation| {
            mutation.aspect("identity.id", "unsupported-invalidation")
        })
        .unwrap();
    let delivery = subject.drain(&mut workspace).unwrap();
    let _peer = candidate.drain(&mut workspace).unwrap();
    let stop = match subject.consumer_invalidation_delta(delivery) {
        Err(stop) => stop,
        Ok(_) => panic!("deferred invalidation support minted a delta"),
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryConsumerInvalidationDeltaStopKind::ConsumerSupportUnavailable
    );
    assert_eq!(
        stop.support_dimension(),
        Some(domain::WorthQueryConsumerSupportDimension::Invalidation)
    );
    assert_eq!(
        stop.support_posture(),
        Some(domain::WorthQueryConsumerSupportPosture::Deferred)
    );
    assert_eq!(stop.counters().consumer_support_checks, 1);
    assert_eq!(stop.counters().disposition_classifications, 0);
    assert_eq!(stop.counters().native_access_layout_lookups, 0);
}
