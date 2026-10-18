use crate::relational_certification::world::supply_chain::*;

#[test]
fn semantic_trace_executes_deltas_and_records_actual_divergence() {
    let scale = SupplyChainScale::court();
    let baseline = SupplyChainBaseline::operating(scale);
    let branch = baseline
        .branch
        .fork(BranchLabel::Storm, BranchLabel::Operating)
        .unwrap();
    let expected = ExpectedSupplyChainObservation::from_branch(
        &apply(&branch, DeltaId::StormRerouteAurora).unwrap(),
    );
    let trace = SemanticTrace::new(
        scale,
        BaselineName::Operating,
        BranchLabel::Storm,
        vec![DeltaId::StormRerouteAurora],
    )
    .with_mutation(
        MutationId::MissingWrite,
        MutationOperation::RemoveEntity,
        &expected,
    );
    let replay = trace.replay().unwrap();
    assert!(matches!(
        replay.first_divergence,
        ComparisonMismatch::MissingEntity(key) if key == Anchor::AuroraEastbound.entity()
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
    let scale = SupplyChainScale::court();
    let baseline = SupplyChainBaseline::operating(scale);
    let branch = baseline
        .branch
        .fork(BranchLabel::Storm, BranchLabel::Operating)
        .unwrap();
    let expected = ExpectedSupplyChainObservation::from_branch(&branch);
    let mutations = [
        (
            MutationId::MissingWrite,
            MutationOperation::RemoveEntity,
            ComparisonMismatch::MissingEntity(Anchor::AuroraEastbound.entity()),
        ),
        (
            MutationId::SiblingLeak,
            MutationOperation::CopySiblingEntity,
            ComparisonMismatch::EntityValue(SemanticPath::entity(Anchor::AuroraEastbound.entity())),
        ),
        (
            MutationId::FloatingBranch,
            MutationOperation::SelectOperatingBranch,
            ComparisonMismatch::FloatingBranchSelection(BranchLabel::Operating),
        ),
        (
            MutationId::WrongAncestry,
            MutationOperation::ReplaceParent,
            ComparisonMismatch::WrongAncestry {
                expected: BranchLabel::Operating,
                observed: Some(BranchLabel::Customs),
            },
        ),
        (
            MutationId::DuplicateRelation,
            MutationOperation::DuplicateRelation,
            ComparisonMismatch::DuplicateRelation(RelationKey::new(RelationKind::CallAtPort, 1)),
        ),
        (
            MutationId::IllegalEndpoint,
            MutationOperation::RepointEndpoint,
            ComparisonMismatch::IllegalEndpoint(SchemaError::InvalidEndpoint {
                relation: RelationKind::CallAtPort,
                source: EntityKind::PortCall,
                target: EntityKind::Port,
            }),
        ),
    ];
    for (id, operation, expected_mismatch) in mutations {
        let trace = SemanticTrace::new(
            scale,
            BaselineName::Operating,
            BranchLabel::Storm,
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
    let scale = SupplyChainScale::court();
    let baseline = SupplyChainBaseline::operating(scale);
    let expected = ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let mut trace = SemanticTrace::new(
        scale,
        BaselineName::Operating,
        BranchLabel::Operating,
        Vec::new(),
    )
    .with_mutation(
        MutationId::MissingWrite,
        MutationOperation::RemoveEntity,
        &expected,
    );
    trace.seed += 1;
    assert!(matches!(
        trace.replay(),
        Err(TraceReplayError::SeedMismatch { .. })
    ));
}

#[test]
fn trace_replay_rejects_unsupported_versions_and_validates_raw_relation_vectors() {
    let scale = SupplyChainScale::court();
    let baseline = SupplyChainBaseline::operating(scale);
    let expected = ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let trace = SemanticTrace::new(
        scale,
        BaselineName::Operating,
        BranchLabel::Operating,
        Vec::new(),
    )
    .with_mutation(
        MutationId::MissingWrite,
        MutationOperation::RemoveEntity,
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
        Err(TraceReplayError::UnsupportedVersion(2))
    );

    let mut forged_vector = trace;
    forged_vector.recorded_relation_vector = Some(vec![0]);
    assert!(matches!(
        forged_vector.replay(),
        Err(TraceReplayError::RecordedRelationVectorMismatch { .. })
    ));
}

#[test]
fn trace_replay_rejects_a_forged_first_divergence() {
    let scale = SupplyChainScale::court();
    let baseline = SupplyChainBaseline::operating(scale);
    let expected = ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let mut trace = SemanticTrace::new(
        scale,
        BaselineName::Operating,
        BranchLabel::Operating,
        Vec::new(),
    )
    .with_mutation(
        MutationId::MissingWrite,
        MutationOperation::RemoveEntity,
        &expected,
    );
    trace.first_divergence = Some(ComparisonMismatch::FloatingBranchSelection(
        BranchLabel::Operating,
    ));
    assert!(matches!(
        trace.replay(),
        Err(TraceReplayError::ForgedFirstDivergence { .. })
    ));
}

#[test]
fn replay_matrix_covers_nonempty_baselines_and_all_profiles() {
    let profiles = [
        SupplyChainScale::court(),
        SupplyChainScale::standard(),
        SupplyChainScale::scale(),
    ];
    for scale in profiles {
        let baselines = [
            BaselineName::Operating,
            BaselineName::ContestedPlanning,
            BaselineName::RetentionPressure,
            BaselineName::VersionBoundary,
        ];
        for baseline_name in baselines {
            let branch = if baseline_name == BaselineName::VersionBoundary {
                BranchLabel::HazardV2
            } else {
                BranchLabel::Storm
            };
            let deltas = if baseline_name == BaselineName::VersionBoundary {
                vec![DeltaId::AdoptHazardClassificationV2]
            } else {
                vec![DeltaId::StormRerouteAurora]
            };
            let baseline = match baseline_name {
                BaselineName::Operating => SupplyChainBaseline::operating(scale),
                BaselineName::ContestedPlanning => SupplyChainBaseline::contested(scale),
                BaselineName::RetentionPressure => SupplyChainBaseline::retention_pressure(scale),
                BaselineName::VersionBoundary => SupplyChainBaseline::version_boundary(scale),
                BaselineName::EmptyInstallation => unreachable!(),
            };
            let child = baseline
                .branch
                .fork(branch, BranchLabel::Operating)
                .unwrap();
            let child = apply(&child, deltas[0]).unwrap();
            let expected = ExpectedSupplyChainObservation::from_branch(&child);
            let trace = SemanticTrace::new(scale, baseline_name, branch, deltas).with_mutation(
                MutationId::MissingWrite,
                MutationOperation::RemoveEntity,
                &expected,
            );
            let first = trace.replay().unwrap();
            let second = trace.replay().unwrap();
            assert_eq!(first.mutated_input, second.mutated_input);
            assert_eq!(first.first_divergence, second.first_divergence);
            assert_eq!(
                first.replayed_trace.mutation,
                second.replayed_trace.mutation
            );
            assert_eq!(
                first.replayed_trace.replay().unwrap().mutated_input,
                first.mutated_input
            );
        }
    }
}

#[test]
fn empty_baseline_rejects_domain_delta_instead_of_fabricating_success() {
    let scale = SupplyChainScale::court();
    let baseline = SupplyChainBaseline::empty(scale);
    let expected = ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let trace = SemanticTrace::new(
        scale,
        BaselineName::EmptyInstallation,
        BranchLabel::Operating,
        vec![DeltaId::HoldMedicalCargo],
    )
    .with_mutation(
        MutationId::MissingWrite,
        MutationOperation::RemoveEntity,
        &expected,
    );
    assert!(matches!(
        trace.replay(),
        Err(TraceReplayError::DeltaApplication { .. })
    ));
}
