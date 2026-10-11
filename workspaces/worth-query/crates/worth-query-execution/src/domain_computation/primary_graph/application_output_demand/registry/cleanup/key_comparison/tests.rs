//! Comparison charges cover the declared text actually visited by the key.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryProducerApplicability, WorthQueryProducerLifecyclePosture,
};

#[test]
fn cleanup_comparison_charges_family_and_profile_text() {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let runtime = world.application.product_runtime();
    let product = runtime
        .admit_product_branch(runtime.default_branch())
        .unwrap();
    let source = super::super::super::SourceEpoch::new(
        [1; 32],
        [2; 32],
        worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            1,
            1,
        ),
        product.observation().lifecycle_incarnation(),
        1,
        [3; 32],
    );
    let key = |family: &'static str, profile: &'static str| {
        WorthQueryOutputDemandKey::new(
            family,
            "same".to_owned(),
            WorthQueryProducerApplicability::new(
                profile,
                WorthQueryProducerLifecyclePosture::Preserve,
            ),
            source.clone(),
        )
    };
    // Four producer bytes, the two declared texts, and ten fixed visits.
    assert_eq!(key_work(&key("f", "p")).unwrap(), 4 + 1 + 1 + 10);
    assert_eq!(
        key_work(&key("declared-family", "declared-profile")).unwrap(),
        4 + 15 + 16 + 10
    );
}
