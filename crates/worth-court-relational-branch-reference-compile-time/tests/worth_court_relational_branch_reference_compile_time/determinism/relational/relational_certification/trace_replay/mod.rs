pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::apply, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_digest::canonical_bytes};

#[test]
fn semantic_trace_executes_deltas_and_records_actual_divergence() {
    let scale = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court();
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(scale);
    let branch = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(
        &apply(&branch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora).unwrap(),
    );
    let trace = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::SemanticTrace::new(
        scale,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
        vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora],
    )
    .with_mutation(
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::MissingWrite,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::RemoveEntity,
        &expected,
    );
    let replay = trace.replay().unwrap();
    assert!(matches!(
        replay.first_divergence,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::MissingEntity(key) if key == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity()
    ));
    assert!(replay.replayed_trace.first_divergence.is_some());
    assert_ne!(replay.mutated_input, canonical_bytes(&replay.observation));
    let rerun = replay.replayed_trace.replay().unwrap();
    assert_eq!(rerun.mutated_input, replay.mutated_input);
    assert_eq!(rerun.first_divergence, replay.first_divergence);
    assert!(!trace.replay_fingerprint().is_empty());
}

#[test]
fn every_mutation_has_a_real_negative_twin() {
    let scale = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court();
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(scale);
    let branch = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&branch);
    let mutations = [
        (
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::MissingWrite,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::RemoveEntity,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::MissingEntity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity()),
        ),
        (
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::SiblingLeak,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::CopySiblingEntity,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::EntityValue(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity())),
        ),
        (
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::FloatingBranch,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::SelectOperatingBranch,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::FloatingBranchSelection(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating),
        ),
        (
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::WrongAncestry,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::ReplaceParent,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::WrongAncestry {
                expected: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
                observed: Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs),
            },
        ),
        (
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::DuplicateRelation,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::DuplicateRelation,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::DuplicateRelation(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1)),
        ),
        (
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::IllegalEndpoint,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::RepointEndpoint,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::IllegalEndpoint(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::InvalidEndpoint {
                relation: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort,
                source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall,
                target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
            }),
        ),
    ];
    for (id, operation, expected_mismatch) in mutations {
        let trace = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::SemanticTrace::new(
            scale,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            Vec::new(),
        )
        .with_mutation(id, operation, &expected);
        let replay = trace.replay().unwrap();
        assert_eq!(replay.first_divergence, expected_mismatch);
        let rerun = replay.replayed_trace.replay().unwrap();
        assert_eq!(rerun.mutated_input, replay.mutated_input);
    }
}

#[test]
fn trace_replay_rejects_a_forged_profile_seed() {
    let scale = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court();
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(scale);
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let mut trace = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::SemanticTrace::new(
        scale,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
        Vec::new(),
    )
    .with_mutation(
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::MissingWrite,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::RemoveEntity,
        &expected,
    );
    trace.seed += 1;
    assert!(matches!(
        trace.replay(),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::SeedMismatch { .. })
    ));
}

#[test]
fn trace_replay_rejects_unsupported_versions_and_validates_raw_relation_vectors() {
    let scale = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court();
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(scale);
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let trace = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::SemanticTrace::new(
        scale,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
        Vec::new(),
    )
    .with_mutation(
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::MissingWrite,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::RemoveEntity,
        &expected,
    );
    assert!(trace.recorded_relation_vector.is_some());
    let replay = trace.replay().unwrap();
    assert_eq!(
        replay.replayed_trace.recorded_relation_vector.as_ref(),
        Some(&replay.relation_vector_input)
    );
    assert!(!replay.relation_vector_input.is_empty());

    let mut unsupported = trace.clone();
    unsupported.version = 2;
    assert_eq!(
        unsupported.replay(),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::UnsupportedVersion(2))
    );

    let mut forged_vector = trace;
    forged_vector.recorded_relation_vector = Some(vec![0]);
    assert!(matches!(
        forged_vector.replay(),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::RecordedRelationVectorMismatch { .. })
    ));
}

#[test]
fn trace_replay_rejects_a_forged_first_divergence() {
    let scale = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court();
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(scale);
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let mut trace = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::SemanticTrace::new(
        scale,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
        Vec::new(),
    )
    .with_mutation(
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::MissingWrite,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::RemoveEntity,
        &expected,
    );
    trace.first_divergence = Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::FloatingBranchSelection(
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
    ));
    assert!(matches!(
        trace.replay(),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::ForgedFirstDivergence { .. })
    ));
}

#[test]
fn empty_baseline_rejects_domain_delta_instead_of_fabricating_success() {
    let scale = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court();
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::empty(scale);
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let trace = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::SemanticTrace::new(
        scale,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::EmptyInstallation,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
        vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo],
    )
    .with_mutation(
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::MissingWrite,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::RemoveEntity,
        &expected,
    );
    assert!(matches!(
        trace.replay(),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::DeltaApplication { .. })
    ));
}
