//! Certification audit of the `G7` carried cutover (`M4`): the production
//! roster derivation (carried delta when a verified predecessor roster
//! exists, full rebuild otherwise) beside an independent full-scan oracle
//! over the same selected root, with the work each performed.

use super::{
    delta_scan, full_scan, CarriedManifestRoster, RecordPublicationDirector,
    SelectedBlobManifestPin, SelectedBlobManifestPinDenial,
};

pub(in crate::physical_runtime) struct CutoverPinAudit {
    /// The production derivation advanced a carried predecessor roster.
    pub(in crate::physical_runtime) carried: bool,
    pub(in crate::physical_runtime) pins: Vec<SelectedBlobManifestPin>,
    pub(in crate::physical_runtime) oracle: Vec<SelectedBlobManifestPin>,
    /// Routes or routing blocks the production derivation inspected.
    pub(in crate::physical_runtime) inspected: u64,
    /// Routes the full-scan oracle read.
    pub(in crate::physical_runtime) routes: u64,
}

impl RecordPublicationDirector {
    pub(in crate::physical_runtime) fn certification_cutover_pin_audit(
        &self,
        maximum_pins: usize,
        memory_budget: u64,
    ) -> Result<CutoverPinAudit, SelectedBlobManifestPinDenial> {
        let mut selected = self.selected_roster_inputs(maximum_pins, memory_budget)?;
        let carried = self.carried_roster_for(selected.root, selected.tree_identity);
        let (pins, inspected) = match &carried {
            Some(carried) => delta_scan(
                selected.reader,
                carried,
                selected.limits,
                &mut selected.scratch,
            )?,
            None => full_scan(selected.reader, selected.limits, &mut selected.scratch)?,
        };
        self.carry_roster(CarriedManifestRoster {
            root: selected.root,
            tree_identity: selected.tree_identity,
            pins: pins.clone(),
        });
        let mut oracle = self.selected_roster_inputs(maximum_pins, memory_budget)?;
        if oracle.root != selected.root {
            return Err(SelectedBlobManifestPinDenial::RootUnavailable);
        }
        let (oracle_pins, routes) = full_scan(oracle.reader, oracle.limits, &mut oracle.scratch)?;
        Ok(CutoverPinAudit {
            carried: carried.is_some(),
            pins,
            oracle: oracle_pins,
            inspected,
            routes,
        })
    }
}
