/// The declared equivalence policy for a query used as an output source.
///
/// Output reuse compares the installed query and parameter identities first,
/// then every consumed positive, absent, negative, and complete-set read at
/// its native source revision. A changed revision requires fresh admission,
/// even when a projected value happens to compare equal.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationQueryDependencyEquivalence {
    CompleteNativeSourceRevisions,
}

impl ApplicationQueryDependencyEquivalence {
    pub const fn identity_basis(self) -> &'static str {
        match self {
            Self::CompleteNativeSourceRevisions => "installed-query-and-parameters",
        }
    }

    pub const fn comparator(self) -> &'static str {
        match self {
            Self::CompleteNativeSourceRevisions => "exact-native-source-revision",
        }
    }

    pub const fn invalidation_basis(self) -> &'static str {
        match self {
            Self::CompleteNativeSourceRevisions => {
                "positive-absent-negative-complete-set-observations"
            }
        }
    }
}
