//! Retire the original unique receipt only after all live readers preflight.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
    WorthQueryOperationHomeRetirementProgress as Progress,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_world::facade::{
    ConsumedPublicationRetirement, ConsumedPublicationRetirementProgress,
    PreparedConsumedPublicationRetirement, RetiredConsumedPublication,
};

pub(super) enum PublicationReceiptRetirement {
    Vacant,
    Prepared(Option<PreparedConsumedPublicationRetirement>),
    Retiring(Option<ConsumedPublicationRetirement>),
    Retired {
        _original: RetiredConsumedPublication,
    },
}
impl WorthQueryProductPublicationReceipt {
    pub(crate) fn preflight_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        request.consult().map_err(Denial::Execution)?;
        if self.history.is_some() {
            return Err(Denial::Busy(Owner::PublicationReceipts));
        }
        let original =
            Arc::get_mut(&mut self.custody).ok_or(Denial::Busy(Owner::PublicationReceipts))?;
        // These slots hold unfinished one-shot observation handoffs. Returning
        // Busy keeps both those actual obligations and the same Query serving.
        for observation in [
            &mut original.live_successor_observation,
            &mut original.output_demand_observation,
            &mut original.client_observation,
        ] {
            if observation
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_some()
            {
                return Err(Denial::Busy(Owner::PublicationReceipts));
            }
        }
        match &original.retirement {
            PublicationReceiptRetirement::Vacant => {
                let publication = original
                    .terminal
                    .get()
                    .expect("filled original receipt")
                    .publication
                    .as_ref()
                    .expect("serving original publication");
                original.retirement = PublicationReceiptRetirement::Prepared(Some(
                    publication
                        .prepare_home_retirement(request)
                        .map_err(Denial::Publication)?,
                ));
                Ok(())
            }
            PublicationReceiptRetirement::Prepared(_) => Ok(()),
            PublicationReceiptRetirement::Retiring(_)
            | PublicationReceiptRetirement::Retired { .. } => {
                Err(Denial::Busy(Owner::PublicationReceipts))
            }
        }
    }
    pub(crate) fn advance_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<Progress, Denial> {
        request.consult().map_err(Denial::Execution)?;
        let original =
            Arc::get_mut(&mut self.custody).ok_or(Denial::Busy(Owner::PublicationReceipts))?;
        match &mut original.retirement {
            PublicationReceiptRetirement::Vacant => Err(Denial::Busy(Owner::PublicationReceipts)),
            PublicationReceiptRetirement::Prepared(prepared) => {
                let publication = original
                    .terminal
                    .get_mut()
                    .expect("filled original receipt")
                    .publication
                    .take()
                    .expect("original publication moves once");
                match prepared
                    .take()
                    .expect("same prepaid original slot")
                    .begin(publication)
                {
                    Ok(task) => {
                        *original
                            .fresh_delivery_available
                            .get_mut()
                            .unwrap_or_else(std::sync::PoisonError::into_inner) = false;
                        original.retirement = PublicationReceiptRetirement::Retiring(Some(task));
                    }
                    Err((same_prepared, same_publication, cause)) => {
                        *prepared = Some(same_prepared);
                        original
                            .terminal
                            .get_mut()
                            .expect("same original terminal")
                            .publication = Some(same_publication);
                        return Err(Denial::Publication(cause));
                    }
                }
                Ok(Progress::Advanced)
            }
            PublicationReceiptRetirement::Retiring(task) => {
                match task
                    .as_mut()
                    .expect("same original World task")
                    .advance(request, native)
                    .map_err(Denial::Publication)?
                {
                    ConsumedPublicationRetirementProgress::Advanced => Ok(Progress::Advanced),
                    ConsumedPublicationRetirementProgress::Complete => {
                        let original_task = task
                            .take()
                            .expect("completed original task")
                            .into_retired()
                            .unwrap_or_else(|_| unreachable!("World reported Complete"));
                        original.retirement = PublicationReceiptRetirement::Retired {
                            _original: original_task,
                        };
                        Ok(Progress::Complete)
                    }
                }
            }
            PublicationReceiptRetirement::Retired { .. } => Ok(Progress::Complete),
        }
    }
}
