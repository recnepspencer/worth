//! The actual selected producer seals its original producing association once.
use super::WorthQueryApplicationIdempotencyBinding;
use worth_relational::facade::mvcc::RequiredOutputProducingImplementation;
impl WorthQueryApplicationIdempotencyBinding {
    pub(in crate::domain_computation::primary_graph) const fn producing_implementation(
        &self,
    ) -> Option<RequiredOutputProducingImplementation> {
        self.producing_implementation
    }
    pub(in crate::domain_computation::primary_graph) fn bind_producing_implementation(
        mut self,
        implementation: RequiredOutputProducingImplementation,
    ) -> Self {
        assert!(
            self.producing_implementation
                .replace(implementation)
                .is_none(),
            "one selected producer implementation"
        );
        self
    }
}
