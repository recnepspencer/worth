/// A separately bounded Runtime World population or byte budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeWorldBudgetResource {
    LiveProductBranches,
    RetainedCompositeCommits,
    HistoryMetadataBytes,
    ActiveObservations,
    ActivePublicationAttempts,
    RetainedProductUnpublishedRecords,
    RetainedPartialMetadataBytes,
    UniqueExactComponentPins,
    InFlightPinAcquisitionReservations,
    OwnerCreatedComponentCustodyRecords,
}

/// Construction or later admission was rejected by one named bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeWorldBudgetDenial {
    ZeroLimit {
        resource: RuntimeWorldBudgetResource,
    },
    SizeOverflow {
        resource: RuntimeWorldBudgetResource,
        value: u64,
    },
    CapacityExhausted {
        resource: RuntimeWorldBudgetResource,
        limit: usize,
    },
}

impl RuntimeWorldBudgetDenial {
    pub const fn resource(self) -> RuntimeWorldBudgetResource {
        match self {
            Self::ZeroLimit { resource }
            | Self::SizeOverflow { resource, .. }
            | Self::CapacityExhausted { resource, .. } => resource,
        }
    }
}
