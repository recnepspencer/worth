pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::SemanticTrace, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::apply, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_digest::canonical_bytes, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace};

#[test]
fn replay_matrix_covers_nonempty_baselines_and_all_profiles() {
    let profiles = [
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::standard(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::scale(),
    ];
    for scale in profiles {
        let baselines = [
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::ContestedPlanning,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::RetentionPressure,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::VersionBoundary,
        ];
        for baseline_name in baselines {
            let branch = if baseline_name == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::VersionBoundary {
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::HazardV2
            } else {
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm
            };
            let deltas = if baseline_name == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::VersionBoundary {
                vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2]
            } else {
                vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora]
            };
            let baseline = match baseline_name {
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating => worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(scale),
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::ContestedPlanning => worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::contested(scale),
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::RetentionPressure => worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::retention_pressure(scale),
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::VersionBoundary => worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::version_boundary(scale),
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::EmptyInstallation => unreachable!(),
            };
            let child = baseline
                .branch
                .fork(branch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
                .unwrap();
            let child = apply(&child, deltas[0]).unwrap();
            let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&child);
            let trace = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::SemanticTrace::new(scale, baseline_name, branch, deltas).with_mutation(
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::MissingWrite,
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::RemoveEntity,
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
