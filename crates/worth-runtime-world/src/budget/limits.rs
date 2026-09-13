use std::num::NonZeroUsize;

use super::{RuntimeWorldBudgetDenial, RuntimeWorldBudgetResource};

/// A nonzero installed limit. It has no usage counter and cannot be used as a
/// proof that capacity is currently available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeWorldBudgetLimit(NonZeroUsize);

impl RuntimeWorldBudgetLimit {
    pub const fn get(self) -> usize {
        self.0.get()
    }
}

/// All bounded Runtime World populations are installed together.
///
/// There is intentionally no `Default`: omitting a bound must be a compile- or
/// construction-time decision at the composition root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeWorldBudgets {
    live_product_branches: RuntimeWorldBudgetLimit,
    retained_composite_commits: RuntimeWorldBudgetLimit,
    history_metadata_bytes: RuntimeWorldBudgetLimit,
    active_observations: RuntimeWorldBudgetLimit,
    active_publication_attempts: RuntimeWorldBudgetLimit,
    retained_product_unpublished_records: RuntimeWorldBudgetLimit,
    retained_partial_metadata_bytes: RuntimeWorldBudgetLimit,
    unique_exact_component_pins: RuntimeWorldBudgetLimit,
    in_flight_pin_acquisition_reservations: RuntimeWorldBudgetLimit,
    owner_created_component_custody_records: RuntimeWorldBudgetLimit,
}

/// Named installation inputs for branch capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeWorldBranchBudgetInstallation {
    pub live_product_branches: u64,
}

/// Named installation inputs for history capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeWorldHistoryBudgetInstallation {
    pub retained_composite_commits: u64,
    pub history_metadata_bytes: u64,
}

/// Named installation inputs for observation capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeWorldObservationBudgetInstallation {
    pub active_observations: u64,
}

/// Named installation inputs for publication capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeWorldPublicationBudgetInstallation {
    pub active_publication_attempts: u64,
}

/// Named installation inputs for recovery capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeWorldRecoveryBudgetInstallation {
    pub retained_product_unpublished_records: u64,
    pub retained_partial_metadata_bytes: u64,
}

/// Named installation inputs for independent exact-component retention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeWorldRetentionBudgetInstallation {
    pub unique_exact_component_pins: u64,
    pub in_flight_pin_acquisition_reservations: u64,
}

/// Named installation inputs for owner-created custody records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeWorldCustodyBudgetInstallation {
    pub owner_created_component_custody_records: u64,
}

/// Complete named Runtime World capacity installation. Each responsibility is
/// a separate field so adding a bound cannot silently shift a positional
/// argument into another population.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeWorldBudgetInstallation {
    pub branches: RuntimeWorldBranchBudgetInstallation,
    pub history: RuntimeWorldHistoryBudgetInstallation,
    pub observations: RuntimeWorldObservationBudgetInstallation,
    pub publication: RuntimeWorldPublicationBudgetInstallation,
    pub recovery: RuntimeWorldRecoveryBudgetInstallation,
    pub retention: RuntimeWorldRetentionBudgetInstallation,
    pub custody: RuntimeWorldCustodyBudgetInstallation,
}

impl RuntimeWorldBudgets {
    pub fn install(
        installation: RuntimeWorldBudgetInstallation,
    ) -> Result<Self, RuntimeWorldBudgetDenial> {
        Ok(Self {
            live_product_branches: limit(
                RuntimeWorldBudgetResource::LiveProductBranches,
                installation.branches.live_product_branches,
            )?,
            retained_composite_commits: limit(
                RuntimeWorldBudgetResource::RetainedCompositeCommits,
                installation.history.retained_composite_commits,
            )?,
            history_metadata_bytes: limit(
                RuntimeWorldBudgetResource::HistoryMetadataBytes,
                installation.history.history_metadata_bytes,
            )?,
            active_observations: limit(
                RuntimeWorldBudgetResource::ActiveObservations,
                installation.observations.active_observations,
            )?,
            active_publication_attempts: limit(
                RuntimeWorldBudgetResource::ActivePublicationAttempts,
                installation.publication.active_publication_attempts,
            )?,
            retained_product_unpublished_records: limit(
                RuntimeWorldBudgetResource::RetainedProductUnpublishedRecords,
                installation.recovery.retained_product_unpublished_records,
            )?,
            retained_partial_metadata_bytes: limit(
                RuntimeWorldBudgetResource::RetainedPartialMetadataBytes,
                installation.recovery.retained_partial_metadata_bytes,
            )?,
            unique_exact_component_pins: limit(
                RuntimeWorldBudgetResource::UniqueExactComponentPins,
                installation.retention.unique_exact_component_pins,
            )?,
            in_flight_pin_acquisition_reservations: limit(
                RuntimeWorldBudgetResource::InFlightPinAcquisitionReservations,
                installation
                    .retention
                    .in_flight_pin_acquisition_reservations,
            )?,
            owner_created_component_custody_records: limit(
                RuntimeWorldBudgetResource::OwnerCreatedComponentCustodyRecords,
                installation.custody.owner_created_component_custody_records,
            )?,
        })
    }

    pub const fn live_product_branches(&self) -> RuntimeWorldBudgetLimit {
        self.live_product_branches
    }

    pub const fn retained_composite_commits(&self) -> RuntimeWorldBudgetLimit {
        self.retained_composite_commits
    }

    pub const fn history_metadata_bytes(&self) -> RuntimeWorldBudgetLimit {
        self.history_metadata_bytes
    }

    pub const fn active_observations(&self) -> RuntimeWorldBudgetLimit {
        self.active_observations
    }

    pub const fn active_publication_attempts(&self) -> RuntimeWorldBudgetLimit {
        self.active_publication_attempts
    }

    pub const fn retained_product_unpublished_records(&self) -> RuntimeWorldBudgetLimit {
        self.retained_product_unpublished_records
    }

    pub const fn retained_partial_metadata_bytes(&self) -> RuntimeWorldBudgetLimit {
        self.retained_partial_metadata_bytes
    }

    pub const fn unique_exact_component_pins(&self) -> RuntimeWorldBudgetLimit {
        self.unique_exact_component_pins
    }

    pub const fn in_flight_pin_acquisition_reservations(&self) -> RuntimeWorldBudgetLimit {
        self.in_flight_pin_acquisition_reservations
    }

    pub const fn owner_created_component_custody_records(&self) -> RuntimeWorldBudgetLimit {
        self.owner_created_component_custody_records
    }
}

fn limit(
    resource: RuntimeWorldBudgetResource,
    value: u64,
) -> Result<RuntimeWorldBudgetLimit, RuntimeWorldBudgetDenial> {
    let value = usize::try_from(value)
        .map_err(|_| RuntimeWorldBudgetDenial::SizeOverflow { resource, value })?;
    let value = NonZeroUsize::new(value).ok_or(RuntimeWorldBudgetDenial::ZeroLimit { resource })?;
    Ok(RuntimeWorldBudgetLimit(value))
}
