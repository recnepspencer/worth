use worth_foundational::facade::DeterminismContract;

/// Decision context whose value may affect a producer's canonical output.
/// The declaration is an upper bound; observed getter use checks completeness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryDecisionContextDependencies {
    bits: u8,
}

impl WorthQueryDecisionContextDependencies {
    pub const NONE: Self = Self { bits: 0 };
    pub const KEY: Self = Self { bits: 1 };
    pub const PRINCIPAL: Self = Self { bits: 2 };
    pub const SCOPE: Self = Self { bits: 4 };

    pub const fn union(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }

    pub const fn contains(self, other: Self) -> bool {
        self.bits & other.bits == other.bits
    }

    pub(crate) const fn bits(self) -> u8 {
        self.bits
    }
}

/// Static, producer-specific eligibility declaration for exact input reuse.
/// It does not authorize reuse without an exactly completed handler read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryProducerInputReuseContract {
    determinism: DeterminismContract,
    context: WorthQueryDecisionContextDependencies,
}

impl WorthQueryProducerInputReuseContract {
    pub const fn new(
        determinism: DeterminismContract,
        context: WorthQueryDecisionContextDependencies,
    ) -> Self {
        Self {
            determinism,
            context,
        }
    }

    pub const fn canonical_bitwise(context: WorthQueryDecisionContextDependencies) -> Self {
        Self::new(DeterminismContract::CanonicalBitwise, context)
    }

    pub const fn determinism(self) -> DeterminismContract {
        self.determinism
    }

    pub const fn context(self) -> WorthQueryDecisionContextDependencies {
        self.context
    }

    pub(crate) const fn portable_meaning(self) -> (u8, u64, u8) {
        let (kind, equivalence) = match self.determinism {
            DeterminismContract::CanonicalBitwise => (1, 0),
            DeterminismContract::ContractEquivalent(id) => (2, id.value()),
        };
        (kind, equivalence, self.context.bits())
    }
}
