//! Canonical mutation input and separately governed admission input are distinct DATA.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;
#[test]
fn canonical_mutation_input_is_the_root_not_the_optional_governed_slot() {
    for governed in [None, Some([2; 32])] {
        let mut binding = WorthQueryApplicationIdempotencyBinding::new([3; 32], [1; 32])
            .bind_governed_input(governed.as_ref());
        binding.mutation_binding_identity =
            Some(super::super::super::mutation_binding::slot("mutation"));
        let observed = read(&binding.intent_text()).unwrap();
        assert!(observed.matches_mutation_input("mutation", &[1; 32]));
        assert!(!observed.matches_mutation_input("mutation", &[2; 32]));
        assert!(!observed.matches_mutation_input("another", &[1; 32]));
    }
}
