use crate::installation::IsolatedPulseInstallation;

use super::atomic_replacement::{self, PulseSourceActionFailure};

#[derive(Clone, Copy, Debug)]
pub(crate) struct QueryStatusV1;

#[derive(Clone, Copy, Debug)]
pub(crate) struct QueryStatusV2;

impl QueryStatusV1 {
    pub(crate) const VALUE: &'static str = "ONLINE";

    pub(crate) fn apply(
        self,
        installation: &IsolatedPulseInstallation,
    ) -> Result<(), PulseSourceActionFailure> {
        atomic_replacement::apply_path(
            installation.source_root().join("platform-pulse-value.json"),
            include_bytes!("../../../query_samples/status-v1.json"),
        )
    }
}

impl QueryStatusV2 {
    pub(crate) const VALUE: &'static str = "SYNCHRONIZED";

    pub(crate) fn apply(
        self,
        installation: &IsolatedPulseInstallation,
    ) -> Result<(), PulseSourceActionFailure> {
        atomic_replacement::apply_path(
            installation.source_root().join("platform-pulse-value.json"),
            include_bytes!("../../../query_samples/status-v2.json"),
        )
    }
}
