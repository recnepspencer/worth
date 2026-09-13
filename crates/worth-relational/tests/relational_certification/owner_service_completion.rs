#[path = "owner_service_completion/denial_matrix.rs"]
mod denial_matrix;
#[path = "owner_service_completion/equivalence.rs"]
mod equivalence;
#[path = "owner_service_completion/healthy_bundle.rs"]
mod healthy_bundle;
#[cfg(feature = "test-operation-control")]
#[path = "owner_service_completion/lifecycle_locality.rs"]
mod lifecycle_locality;
#[cfg(feature = "test-operation-control")]
#[path = "owner_service_completion/owner_lifecycle_observation.rs"]
mod owner_lifecycle_observation;
#[path = "owner_service_completion/owner_loss.rs"]
mod owner_loss;
#[path = "owner_service_completion/service_properties.rs"]
mod service_properties;
#[path = "owner_service_completion/supply_chain_cutover.rs"]
mod supply_chain_cutover;
#[path = "owner_service_completion/world.rs"]
mod world;
