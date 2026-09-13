/// Finite ordinary-request ceilings declared by one query binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationQueryBindingLimits {
    maximum_results: usize,
    maximum_work: usize,
}

impl ApplicationQueryBindingLimits {
    pub const fn bounded(maximum_results: usize, maximum_work: usize) -> Self {
        Self {
            maximum_results,
            maximum_work,
        }
    }

    pub const fn maximum_results(self) -> usize {
        self.maximum_results
    }

    pub const fn maximum_work(self) -> usize {
        self.maximum_work
    }
}
