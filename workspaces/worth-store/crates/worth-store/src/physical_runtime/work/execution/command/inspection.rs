use super::{
    PhysicalExecutorCommand, PhysicalExecutorCommandDenial, PhysicalInspectionExecutorCommand,
};
use crate::physical_runtime::{PhysicalWorkOperationFamily, ResourceAdmittedPhysicalWork};

impl PhysicalExecutorCommand {
    pub(in crate::physical_runtime) fn inspection(
        work: ResourceAdmittedPhysicalWork,
        destination: Box<[u8]>,
    ) -> Result<Self, PhysicalExecutorCommandDenial> {
        super::types::require_family(&work, PhysicalWorkOperationFamily::ArtifactRangeRead)?;
        let range = work
            .intent()
            .scope()
            .inspection_target()
            .ok_or(PhysicalExecutorCommandDenial::ArtifactCommandRequiresArtifactScope)?;
        if destination.len() != range.length() as usize {
            return Err(PhysicalExecutorCommandDenial::PayloadLengthMismatch);
        }
        Ok(Self::Inspection(PhysicalInspectionExecutorCommand {
            work,
            range,
            destination,
        }))
    }
}
