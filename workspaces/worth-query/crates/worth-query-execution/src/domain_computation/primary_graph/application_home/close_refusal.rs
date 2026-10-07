use worth_relational::facade::{
    durability::DurabilityError, runtime::RelationalRuntimeAdmissionHoldDenial,
};

/// Why an application could not return to its home without waiting.
#[derive(Debug, PartialEq, Eq)]
pub enum WorthQueryApplicationCloseDenial {
    AdmissionsActive,
    Owner(RelationalRuntimeAdmissionHoldDenial),
    Capture(DurabilityError),
}

/// A refused close returns the same, still-working application.
#[must_use = "a refused close still owns its application runtime"]
pub struct WorthQueryApplicationCloseRefusal<Runtime> {
    pub runtime: Runtime,
    pub denial: WorthQueryApplicationCloseDenial,
}

impl<Runtime> std::fmt::Debug for WorthQueryApplicationCloseRefusal<Runtime> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryApplicationCloseRefusal")
            .field("denial", &self.denial)
            .finish_non_exhaustive()
    }
}
