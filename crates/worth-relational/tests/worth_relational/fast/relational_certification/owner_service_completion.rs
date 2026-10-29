#[path = "owner_service_completion/denial_matrix.rs"]
pub(crate) mod denial_matrix;

#[path = "owner_service_completion/equivalence.rs"]
pub(crate) mod equivalence;

#[path = "owner_service_completion/healthy_bundle.rs"]
pub(crate) mod healthy_bundle;

#[path = "owner_service_completion/lifecycle_locality.rs"]
pub(crate) mod lifecycle_locality;

#[path = "owner_service_completion/owner_lifecycle_observation.rs"]
pub(crate) mod owner_lifecycle_observation;

#[path = "owner_service_completion/owner_loss.rs"]
pub(crate) mod owner_loss;

#[path = "owner_service_completion/service_properties.rs"]
pub(crate) mod service_properties;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::relational_certification::owner_service_completion::supply_chain_cutover;

#[path = "../../fixtures/relational_certification/owner_service_completion/world.rs"]
pub(crate) mod world;
