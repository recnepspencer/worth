//! Discharge the original session and reader without dropping their storage.
use super::{PhysicalRecordReader, ReadPlacement, RecordReadSession};
use crate::physical_runtime::stability::PhysicalRootReadRetirementDenial;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};

#[derive(Debug)]
pub enum PhysicalRecordReadRetirementDenial {
    Execution(WorkCeilingDenial),
    ProtectionBusy,
    Residency(worth_store_buffer_pool::PhysicalResidencyRetirementDenial),
}

impl From<PhysicalRootReadRetirementDenial> for PhysicalRecordReadRetirementDenial {
    fn from(cause: PhysicalRootReadRetirementDenial) -> Self {
        match cause {
            PhysicalRootReadRetirementDenial::Busy => Self::ProtectionBusy,
            PhysicalRootReadRetirementDenial::Execution(cause) => Self::Execution(cause),
        }
    }
}

impl RecordReadSession {
    pub(in crate::physical_runtime) fn try_retire_frame(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), PhysicalRecordReadRetirementDenial> {
        request
            .consult()
            .map_err(PhysicalRecordReadRetirementDenial::Execution)?;
        match &mut self.placement {
            ReadPlacement::Inline { frame, .. } => frame.try_retire(request),
            ReadPlacement::Extent(state) => state.try_retire_frame(request),
        }
        .map_err(PhysicalRecordReadRetirementDenial::Residency)
    }

    pub(in crate::physical_runtime) fn try_retire_allocation(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), PhysicalRecordReadRetirementDenial> {
        request
            .consult()
            .map_err(PhysicalRecordReadRetirementDenial::Execution)?;
        let Some(allocation) = self._allocation.take() else {
            return Ok(());
        };
        match allocation.try_retire_for_request(request) {
            Ok(retired) => {
                self._retired_allocation = Some(retired);
                Ok(())
            }
            Err((allocation, cause)) => {
                self._allocation = Some(allocation);
                Err(PhysicalRecordReadRetirementDenial::Residency(cause))
            }
        }
    }

    pub(in crate::physical_runtime) fn try_retire_protection(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), PhysicalRecordReadRetirementDenial> {
        self.protection.try_retire(request).map_err(Into::into)
    }

    pub(in crate::physical_runtime) fn retire_lifecycle(&mut self) {
        self._lifecycle.retire();
    }
}

impl PhysicalRecordReader {
    pub(in crate::physical_runtime) fn try_retire_protection(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), PhysicalRecordReadRetirementDenial> {
        self.protection.try_retire(request).map_err(Into::into)
    }

    pub(in crate::physical_runtime) fn retire_lifecycle(&mut self) {
        self.lifecycle.retire();
    }
}
