use worth_store_authority::StoreCurrentAuthorityIdentity;

use super::{
    FrontierPartialOrder, PitrCandidatePosture, PitrCandidateSelectionDenial, PitrRoundingPolicy,
    RecoveryTimelineAdmission, RecoveryTimelineOwner,
};

#[test]
fn candidate_selection_converges_across_observation_order() {
    let earlier = observation(90, 9, 9, 9, 8, 7);
    let exact = observation(100, 10, 10, 10, 9, 8);
    let left = RecoveryTimelineOwner::resolve_candidates(
        100,
        PitrRoundingPolicy::NearestAcknowledged,
        vec![earlier, exact],
    )
    .expect("candidate set")
    .select()
    .expect("selected candidate");
    let right = RecoveryTimelineOwner::resolve_candidates(
        100,
        PitrRoundingPolicy::NearestAcknowledged,
        vec![exact, earlier],
    )
    .expect("candidate set")
    .select()
    .expect("selected candidate");
    assert_eq!(left, right);
    assert_eq!(left.exact_frontier().wal_structural(), 10);
}

#[test]
fn exact_policy_refuses_to_round_across_an_evidence_gap() {
    let candidates = RecoveryTimelineOwner::resolve_candidates(
        100,
        PitrRoundingPolicy::ExactOnly,
        vec![observation(90, 9, 9, 9, 8, 7)],
    )
    .expect("candidate set");
    assert_eq!(
        candidates.select(),
        Err(PitrCandidateSelectionDenial::NoAdmissibleCandidate)
    );
}

#[test]
fn multiple_previous_candidates_are_canonicalized_by_distance_not_input_time() {
    let selected = RecoveryTimelineOwner::resolve_candidates(
        100,
        PitrRoundingPolicy::PreviousAcknowledged,
        vec![
            observation(70, 7, 7, 7, 7, 7),
            observation(90, 9, 9, 9, 9, 9),
            observation(80, 8, 8, 8, 8, 8),
        ],
    )
    .expect("valid past candidates must form a canonical set")
    .select()
    .expect("nearest previous acknowledgement");
    assert_eq!(selected.observed_time(), 90);
    assert_eq!(selected.clock_distance(), 10);
}

#[test]
fn unbounded_clock_uncertainty_saturates_instead_of_wrapping() {
    let uncertain = RecoveryTimelineOwner::admit_observation(RecoveryTimelineAdmission {
        observed_time: 0,
        uncertainty_before: u64::MAX,
        uncertainty_after: u64::MAX,
        checkpoint_durability: 1,
        wal_structural: 1,
        local_durable_commit: 1,
        client_acknowledged: 1,
        replication_acknowledged: 1,
        authority_identity: StoreCurrentAuthorityIdentity::from_persisted_fingerprint([4; 32]),
        source_lineage: [5; 32],
        source_identity: [6; 32],
        posture: PitrCandidatePosture::Available,
    })
    .unwrap();
    let selected = RecoveryTimelineOwner::resolve_candidates(
        i64::MIN,
        PitrRoundingPolicy::ExactOnly,
        vec![uncertain],
    )
    .unwrap()
    .select()
    .expect("saturated uncertainty interval includes the minimum instant");
    assert_eq!(selected.clock_distance(), 0);
}

#[test]
fn frontier_comparison_preserves_partial_order_instead_of_inventing_total_order() {
    let left = observation(100, 10, 10, 10, 8, 7).frontier;
    let right = observation(100, 10, 10, 10, 7, 8).frontier;
    assert_eq!(
        left.compare(right),
        FrontierPartialOrder::IncomparableDimensions
    );
}

#[test]
fn authorization_identity_preserves_rounding_and_posture_semantics() {
    let available = observation(100, 10, 10, 10, 8, 7);
    let mut degraded = available;
    degraded.posture = PitrCandidatePosture::Degraded;
    let exact = RecoveryTimelineOwner::resolve_candidates(
        100,
        PitrRoundingPolicy::ExactOnly,
        vec![available],
    )
    .unwrap()
    .select()
    .unwrap();
    let nearest = RecoveryTimelineOwner::resolve_candidates(
        100,
        PitrRoundingPolicy::NearestAcknowledged,
        vec![available],
    )
    .unwrap()
    .select()
    .unwrap();
    let degraded = RecoveryTimelineOwner::resolve_candidates(
        100,
        PitrRoundingPolicy::ExactOnly,
        vec![degraded],
    )
    .unwrap()
    .select()
    .unwrap();
    assert_ne!(exact.identity(), nearest.identity());
    assert_ne!(exact.identity(), degraded.identity());
    assert_eq!(exact.exact_frontier(), nearest.exact_frontier());
}

fn observation(
    time: i64,
    checkpoint: u64,
    wal: u64,
    local: u64,
    client: u64,
    replication: u64,
) -> super::RecoveryTimelineObservation {
    RecoveryTimelineOwner::admit_observation(RecoveryTimelineAdmission {
        observed_time: time,
        uncertainty_before: 0,
        uncertainty_after: 0,
        checkpoint_durability: checkpoint,
        wal_structural: wal,
        local_durable_commit: local,
        client_acknowledged: client,
        replication_acknowledged: replication,
        authority_identity: StoreCurrentAuthorityIdentity::from_persisted_fingerprint([4; 32]),
        source_lineage: [5; 32],
        source_identity: [6; 32],
        posture: PitrCandidatePosture::Available,
    })
    .expect("valid observation")
}
