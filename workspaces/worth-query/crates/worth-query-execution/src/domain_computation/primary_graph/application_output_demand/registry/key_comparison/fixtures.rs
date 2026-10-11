use super::super::WorthQueryOutputDemandKey;
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryProducerApplicability, WorthQueryProducerLifecyclePosture,
};

pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn keys(
) -> (WorthQueryOutputDemandKey, WorthQueryOutputDemandKey) {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let runtime = world.application.product_runtime();
    let product = runtime
        .admit_product_branch(runtime.default_branch())
        .unwrap();
    let source = super::super::SourceEpoch::new(
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
    (key("f", "p"), key("declared-family", "declared-profile"))
}
