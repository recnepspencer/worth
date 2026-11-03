//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandInterest {
    #[doc(hidden)]
    pub fn key_for_test(&self) -> &WorthQueryOutputDemandKey {
        &self.key
    }
}

impl WorthQueryOutputDemandInterest {
    #[doc(hidden)]
    pub fn owner_for_test(&self) -> &WorthQueryOutputDemandRegistry {
        &self.owner
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn state_for_test(&self) -> &Arc<Mutex<DemandRegistryState>> {
        &self.state
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn records_for_test(&self) -> &record_map::DemandRecords {
        &self.records
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn records_mut_for_test(&mut self) -> &mut record_map::DemandRecords {
        &mut self.records
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn claim_batch_for_test(&self) -> &Option<claim_readthrough::ClaimBatch> {
        &self.claim_batch
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn prerequisites_for_test(&self) -> &Vec<Arc<WorthQueryOutputDemandKey>> {
        &self.prerequisites
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn state_for_test(&self) -> &DemandState {
        &self.state
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn prerequisites_mut_for_test(&mut self) -> &mut Vec<Arc<WorthQueryOutputDemandKey>> {
        &mut self.prerequisites
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn state_mut_for_test(&mut self) -> &mut DemandState {
        &mut self.state
    }
}
