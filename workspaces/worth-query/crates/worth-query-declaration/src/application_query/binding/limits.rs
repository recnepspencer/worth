/// Semantic maximum result cardinality of an installed query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationQueryBindingLimits {
    maximum_results: usize,
}
impl ApplicationQueryBindingLimits {
    pub const fn results(maximum_results: usize) -> Self {
        Self { maximum_results }
    }
    pub const fn maximum_results(self) -> usize {
        self.maximum_results
    }
}
