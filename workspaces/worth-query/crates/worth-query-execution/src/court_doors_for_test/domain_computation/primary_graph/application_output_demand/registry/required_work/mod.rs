//! Doors under `domain_computation::primary_graph::application_output_demand::registry::required_work`.
#[path = "queue/mod.rs"]
pub mod queue;
#[path = "selection/mod.rs"]
pub mod selection;
#[path = "wave_admission/mod.rs"]
pub mod wave_admission;
pub use crate::domain_computation::primary_graph::application_output_demand::registry::required_work::RequiredWorkMembership;
pub use crate::domain_computation::primary_graph::application_output_demand::registry::required_work::SelectedRequiredWork;
pub use crate::domain_computation::primary_graph::application_output_demand::registry::required_work::SelectedRequiredWorkKind;
