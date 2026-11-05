//! Moved whole from `domain_computation::primary_graph::vocabulary::application_query::basis_registry`: the items below and every impl they own.
use worth_relational::facade::branch::RelationalBranchRetentionTerminalOutcome;

/// Descriptive origin of this exact snapshot; it grants no read authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationBasisSelectionIdentity {
    Relational,
    Product(crate::basis::WorthQueryProductBranchReadIdentity),
}

/// Distinguishes physical Native resource release from ending one local share
/// while another admitted Query owner retains the same resources.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationBasisReleaseOutcome {
    NativeResourcesReleased {
        snapshot_released: bool,
        relational_retention: RelationalBranchRetentionTerminalOutcome,
    },
    /// This local Query share ended; another admitted share still owns the
    /// authentic snapshot, Native retention and program interpretation.
    SharedCustodyRetained,
}

impl WorthQueryApplicationBasisReleaseOutcome {
    /// Whether this release physically closed both native resources.
    pub const fn released(self) -> bool {
        matches!(
            self,
            Self::NativeResourcesReleased {
                snapshot_released: true,
                relational_retention: RelationalBranchRetentionTerminalOutcome::Released,
            }
        )
    }

    /// Whether this lease surrendered its complete local custody. A surviving
    /// shared owner is explicitly distinct from physical native release.
    pub const fn custody_released(self) -> bool {
        self.released() || matches!(self, Self::SharedCustodyRetained)
    }

    pub const fn snapshot_released(self) -> bool {
        matches!(
            self,
            Self::NativeResourcesReleased {
                snapshot_released: true,
                ..
            }
        )
    }

    pub const fn relational_retention(self) -> Option<RelationalBranchRetentionTerminalOutcome> {
        match self {
            Self::NativeResourcesReleased {
                relational_retention,
                ..
            } => Some(relational_retention),
            Self::SharedCustodyRetained => None,
        }
    }
}
