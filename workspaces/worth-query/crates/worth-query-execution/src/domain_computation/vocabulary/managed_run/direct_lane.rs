//! Moved whole from `domain_computation::managed_run::readmission::direct_lane`: the items below and every impl they own.

pub(in crate::domain_computation) struct WorthQueryDirectReadmissionTransitionPermit {
    _owner: (),
}

impl WorthQueryDirectReadmissionTransitionPermit {
    pub(in crate::domain_computation) const fn mint() -> Self {
        Self { _owner: () }
    }
}
