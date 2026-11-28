//! Doors under `domain_computation::primary_graph::application_contribution::producer`.
#[path = "demand/mod.rs"]
pub mod demand;
#[path = "execution/mod.rs"]
pub mod execution;
#[path = "output_equivalence_contract/mod.rs"]
pub mod output_equivalence_contract;
#[path = "output_evaluation/mod.rs"]
pub mod output_evaluation;
#[path = "registry/mod.rs"]
pub mod registry;
#[path = "scheduling/mod.rs"]
pub mod scheduling;
pub use crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerCommitAuthority;
/// The orphan backstop's walk (PR C C4 r5).
pub use crate::domain_computation::primary_graph::application_contribution::producer::synchronous_outputs::court_doors_for_test::first_orphan_for_test;
/// The successor round's #277 conversion predicate (PR C C4 r6).
pub use crate::domain_computation::primary_graph::application_contribution::producer::synchronous_outputs::court_doors_for_test::{admission_converts_to_prior_effect_for_test, converts_to_prior_effect_for_test};
