//! Source consequences use native two-hop observations, effects and publication.
#[path = "retirement/admission.rs"]
mod admission;
#[path = "retirement/binding.rs"]
pub(super) mod binding;
#[path = "retirement/declaration.rs"]
pub(super) mod declaration;
#[path = "retirement/evidence.rs"]
mod evidence;
#[path = "retirement/read.rs"]
mod read;
#[path = "retirement/seed.rs"]
pub(super) mod seed;
