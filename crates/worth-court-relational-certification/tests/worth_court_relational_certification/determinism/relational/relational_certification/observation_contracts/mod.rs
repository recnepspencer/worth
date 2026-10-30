#[allow(unused_imports)]
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::InspectionResult, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::apply, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_digest::canonical_bytes, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_digest::digest, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema};

#[test]
fn canonical_observation_bytes_are_versioned_and_ordered() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let bytes = canonical_bytes(&expected);
    assert!(bytes.starts_with(b"supply-chain-expected-v1\0"));
    assert_eq!(bytes, canonical_bytes(&expected));
    assert_eq!(
        digest(&expected),
        [
            0x42, 0x7b, 0xf1, 0x58, 0x55, 0xb8, 0x11, 0xf5, 0x64, 0x3c, 0xf1, 0x0a, 0x82, 0x72,
            0x2d, 0xec, 0x76, 0x73, 0xc5, 0x5f, 0x53, 0x88, 0x99, 0x29, 0x53, 0x18, 0x55, 0x3e,
            0x05, 0xfe, 0x50, 0xb2,
        ]
    );
}

#[test]
fn canonical_observation_is_invariant_to_relation_insertion_permutation() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let mut permuted = expected.clone();
    let relations: Vec<_> = expected.relations.values().copied().rev().collect();
    permuted.relations.clear();
    for edge in relations {
        permuted.relations.insert(edge.key, edge);
    }
    assert_eq!(canonical_bytes(&expected), canonical_bytes(&permuted));
    assert_eq!(digest(&expected), digest(&permuted));
}
