//! Retire the SAME selected physical read; retain its inert data until reclaim.
use super::{StoreColdSource, StoreSelectedRead};
use std::{any::Any, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::{
    RelationalDurableMemberRetirementObservation as O,
    RelationalDurableMemberRetirementRefusal as R,
};
use worth_store::physical_runtime::{
    AnchoredRecordReadRetirementDenial as Denial, AnchoredRecordReadRetirementProgress as Progress,
    PhysicalRecordReadRetirementDenial as Physical,
};

impl StoreColdSource {
    pub(in crate::cold_source) fn validate_selected_custody(
        &self,
        original: &(dyn Any + Send + Sync),
    ) -> Result<(), R> {
        let selected = original
            .downcast_ref::<StoreSelectedRead>()
            .ok_or(R::ForeignCustody)?;
        if selected.issuer.as_ptr() != self as *const Self
            || !selected
                .serving
                .as_ref()
                .is_some_and(|serving| Arc::ptr_eq(serving, self.serving()))
        {
            return Err(R::ForeignCustody);
        }
        Ok(())
    }
    pub(in crate::cold_source) fn advance_selected_custody(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &mut (dyn Any + Send + Sync),
    ) -> O {
        if let Err(cause) = self.validate_selected_custody(original) {
            return O::Refused(cause);
        }
        if let Err(cause) = request.consult() {
            return O::Stopped(cause);
        }
        let selected = original
            .downcast_mut::<StoreSelectedRead>()
            .expect("validated original selected owner");
        if selected.retirement.is_none() {
            let Some(read) = selected.read.take() else {
                return O::Refused(R::ForeignCustody);
            };
            selected.retirement = Some(read.into_retirement());
        }
        let records = match self.serving().anchored_records() {
            Ok(records) => records,
            Err(_) => return O::Refused(R::ProviderUnavailable),
        };
        match selected
            .retirement
            .as_mut()
            .expect("original physical retirement retained")
            .advance(&records, request)
        {
            Ok(Progress::Advanced) => O::Advanced,
            Ok(Progress::Complete) => {
                selected._retired = Some(
                    selected
                        .retirement
                        .take()
                        .expect("completed original task")
                        .into_retired()
                        .unwrap_or_else(|_| {
                            unreachable!("actual physical retirement reported complete")
                        }),
                );
                // Matching borrowed source owns the other immutable Serving Arc.
                drop(selected.serving.take());
                selected.complete = true;
                O::Complete
            }
            Err(Denial::ForeignServing) => O::Refused(R::ForeignCustody),
            Err(Denial::Owner(Physical::Execution(cause))) => O::Stopped(cause),
            Err(Denial::Owner(Physical::ProtectionBusy)) => O::Busy,
            Err(Denial::Owner(Physical::Residency(
                worth_store::physical_runtime::PhysicalResidencyRetirementDenial::Busy,
            ))) => O::Busy,
            Err(Denial::Owner(Physical::Residency(
                worth_store::physical_runtime::PhysicalResidencyRetirementDenial::Execution(cause),
            ))) => O::Stopped(cause),
            Err(_) => O::Refused(R::ProviderUnavailable),
        }
    }
}
