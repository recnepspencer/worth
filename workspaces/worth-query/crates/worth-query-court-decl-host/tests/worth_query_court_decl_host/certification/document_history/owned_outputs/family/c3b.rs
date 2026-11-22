//! PR C C3b fixture families: R3 settlement (saturated-input), successor
//! independence (self-write) and mutual cross-write (cross-reader,
//! cross-converge, cross-oscillate).
use super::*;
pub mod cross_converge;
pub mod cross_oscillate;
pub mod cross_reader;
pub mod capped_consumer;
pub mod capped_reader;
pub mod cycle_consumer;
pub mod cycle_parent;
pub mod guarded_input;
pub mod saturated_consumer;
pub mod saturated_input;
pub mod self_write;

pub fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    saturated_input::contracts(contracts)?;
    self_write::contracts(contracts)?;
    cross_reader::contracts(contracts)?;
    cross_converge::contracts(contracts)?;
    cross_oscillate::contracts(contracts)?;
    saturated_consumer::contracts(contracts)?;
    capped_reader::contracts(contracts)?;
    capped_consumer::contracts(contracts)?;
    cycle_parent::contracts(contracts)?;
    cycle_consumer::contracts(contracts)?;
    guarded_input::contracts(contracts)
}
pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    saturated_input::install(setup)?;
    self_write::install(setup)?;
    cross_reader::install(setup)?;
    cross_converge::install(setup)?;
    cross_oscillate::install(setup)?;
    saturated_consumer::install(setup)?;
    capped_reader::install(setup)?;
    capped_consumer::install(setup)?;
    cycle_parent::install(setup)?;
    cycle_consumer::install(setup)?;
    guarded_input::install(setup)
}
