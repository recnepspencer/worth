use crate::mvcc::RelationalBranchObservation;
use crate::runtime::RelationalRuntime;

/// Why a runtime refuses to read at an observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalObservationReadDenial {
    /// Another runtime issued the observation. Its commit and root mean
    /// nothing to this runtime's lineage index and commit ancestry.
    ForeignObservation,
}

impl RelationalRuntime {
    /// Admit `observation` for a read only if this runtime issued it.
    pub(super) fn admit_observation_read(
        &self,
        observation: &RelationalBranchObservation,
    ) -> Result<(), RelationalObservationReadDenial> {
        if observation.identity().runtime_instance_id() == self.runtime_instance_id() {
            Ok(())
        } else {
            Err(RelationalObservationReadDenial::ForeignObservation)
        }
    }
}
