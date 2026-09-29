/// Semantic result ceiling and an optional deliberate operational work cap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationQueryBindingLimits {
    maximum_results: usize,
    maximum_work: Option<usize>,
}

impl ApplicationQueryBindingLimits {
    /// Uses the runtime's finite work safeguard without duplicating its accounting.
    pub const fn results(maximum_results: usize) -> Self {
        Self {
            maximum_results,
            maximum_work: None,
        }
    }

    /// Adds an explicit cap that the runtime and request may only narrow.
    pub const fn bounded(maximum_results: usize, maximum_work: usize) -> Self {
        Self {
            maximum_results,
            maximum_work: Some(maximum_work),
        }
    }

    pub const fn maximum_results(self) -> usize {
        self.maximum_results
    }

    pub const fn maximum_work(self) -> Option<usize> {
        self.maximum_work
    }
}
