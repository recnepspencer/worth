//! A supplier's explicit operational refusal, independent of Home custody.
use super::*;
use std::sync::atomic::AtomicBool;
use worth_query_host::facade::primary_graph::{
    WorthQueryOperationHomeRetirementDenial as Denial,
    WorthQueryOperationHomeRetirementProgress as Progress,
};

pub(in crate::facade::host::application_home) struct RefusingBalanceHandler {
    pub(in crate::facade::host::application_home) original: BalanceHandler,
    pub(in crate::facade::host::application_home) refusal: Arc<AtomicBool>,
}

impl OperationHandler<HomeIdentitySchema, SetBalanceBinding> for RefusingBalanceHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        BalanceHandler::declared_durable_capabilities()
    }
    fn preflight_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        request.consult().map_err(Denial::Execution)?;
        if self.refusal.load(Ordering::SeqCst) {
            Err(Denial::Unsupported)
        } else {
            Ok(())
        }
    }
    fn advance_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Progress, Denial> {
        self.preflight_home_retirement(request)?;
        Ok(Progress::Complete)
    }
    fn decide(
        &self,
        value: &u64,
        reader: &mut DecisionReader<'_, '_, '_, HomeIdentitySchema, SetBalanceBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<HomeIdentitySchema, Account>, ()> {
        self.original.decide(value, reader)
    }
    fn build_candidate(
        &self,
        value: &u64,
        target: WorthQueryInvariantMutationTarget<HomeIdentitySchema, Account>,
        writer: &mut CandidateWriter<'_, HomeIdentitySchema, SetBalanceBinding>,
    ) -> HandlerResult<u64, ()> {
        self.original.build_candidate(value, target, writer)
    }
}
