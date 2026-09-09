//! Retained representation accounting; these facts grant no runtime authority.

mod backing;
mod charge;
mod conditional_retention;
mod fork_growth;
mod fork_preparation;
mod measurement;
mod ordered_collections;
mod ordered_index;
mod preparation;
mod small_vector;

pub(crate) use backing::RetainedStorageBacking;
pub(crate) use charge::RetainedStorageCharge;
pub use conditional_retention::SignalConditionalRetentionObservation;
pub(crate) use conditional_retention::{
    SignalConditionalRetentionDenial, SignalConditionalRetentionLedger,
    SignalConditionalRetentionReservation,
};
pub(crate) use fork_growth::{RetainedStorageForkGrowth, RetainedStorageForkGrowthDenial};
pub(crate) use fork_preparation::{RetainedStorageForkCharge, RetainedStorageForkPreparation};
pub(crate) use measurement::{arc_allocation_charge, RetainedStorageMeasurement};
pub(crate) use ordered_index::{
    btree_structure_charge, ordered_index_charge, ordered_lookup_steps,
};
pub(crate) use preparation::{RetainedStoragePreparation, RetainedStoragePreparationDenial};
