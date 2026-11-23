//! The decision-read, R3 settlement and cross-write fixture families, in the
//! order the contribution registers them.
use super::*;

pub fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    decision_write::contracts(contracts)?;
    convergent::contracts(contracts)?;
    decision_input::contracts(contracts)?;
    parity_input::contracts(contracts)?;
    c3b::contracts(contracts)?;
    ordinary_input::contracts(contracts)?;
    diamond::contracts(contracts)?;
    predecessor_family::contracts(contracts)
}
pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    decision_write::install(setup)?;
    convergent::install(setup)?;
    decision_input::install(setup)?;
    parity_input::install(setup)?;
    c3b::install(setup)?;
    ordinary_input::install(setup)?;
    diamond::install(setup)?;
    predecessor_family::install(setup)
}
