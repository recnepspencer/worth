#[path = "installed_composition/contract.rs"]
pub mod contract;
#[path = "installed_composition/declaration.rs"]
pub mod declaration;
#[path = "installed_composition/world.rs"]
pub mod world;

pub use world::{real_denial, CompositionScenario};
