//! Intrinsic result maxima across the selected checked epoch.

#[derive(Clone, Copy)]
pub(super) struct ResultBounds {
    output_type: Option<u64>,
    declared_minimum: u64,
    all_declared: bool,
}

impl ResultBounds {
    pub(super) fn new(output_type: Option<u64>) -> Self {
        Self {
            output_type,
            declared_minimum: 0,
            all_declared: true,
        }
    }

    pub(super) fn with_node(self, declared: Option<u64>) -> Self {
        Self {
            declared_minimum: self.declared_minimum.max(declared.unwrap_or(0)),
            all_declared: self.all_declared && declared.is_some(),
            ..self
        }
    }

    pub(super) fn minimum(self) -> u64 {
        self.declared_minimum
            .min(self.output_type.unwrap_or(u64::MAX))
    }

    pub(super) fn maximum(self) -> Option<u64> {
        // A complete set of declarations needs exactly its largest bound.
        // Only an undeclared result needs an adaptive resource grant.
        if self.all_declared {
            Some(self.minimum())
        } else {
            self.output_type
        }
    }
}
