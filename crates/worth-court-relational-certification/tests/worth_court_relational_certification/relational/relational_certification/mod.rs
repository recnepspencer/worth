#[allow(unused_imports)]
pub(crate) use {};

#[allow(unused_imports)]
pub(crate) use crate::relational_certification::reference_context::mvcc_seeded_model;

#[allow(unused_imports)]
pub(crate) use {};

#[allow(unused_imports)]
pub(crate) use {};

#[allow(unused_imports)]
pub(crate) use crate::relational_certification::growth_context::scale_invariant_admission;

#[allow(unused_imports)]
pub(crate) use {};

#[test]
fn supply_chain_semantic_world_target_is_present() {
    let scale = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court();
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(scale);
    assert_eq!(baseline.scale.name, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::ScaleName::Court);
    assert_eq!(baseline.definition.entities.len(), 244);
}

#[allow(unused_imports)]
pub(crate) use worth_testkit_relational::shared_relational_certification_test_execution_policy as test_execution_policy;
mod aspect_storage;
#[path = "baseline_layers/mod.rs"]
pub(crate) mod baseline_layers;
#[path = "basis_observation/mod.rs"]
pub(crate) mod basis_observation;
mod index_retirement;
#[path = "observation_contracts/mod.rs"]
pub(crate) mod observation_contracts;
#[path = "owner_service_completion/mod.rs"]
pub(crate) mod owner_service_completion;
#[path = "profile_contracts/mod.rs"]
pub(crate) mod profile_contracts;
