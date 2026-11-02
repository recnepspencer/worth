//! Move original Query custody only after its real World has sealed admission.
use super::{
    program_runtime::WorthQueryProgramRuntimeBacking, WorthQueryProgramApplicationRuntime,
};
use crate::domain_computation::primary_graph::{
    application_runtime::RetiringPrimaryGraphRuntime, WorthQueryHomeRetirementDenial as Denial,
    WorthQueryHomeRetirementOwner as Owner, WorthQueryOperationHomeRetirementProgress as Progress,
};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_query_installation::facade::ApplicationSchema;

/// No Clone or live accessor can introduce an alias after original preflight.
pub struct WorthQueryPreparedHomeQuery<Schema, Program> {
    query: WorthQueryProgramApplicationRuntime<Schema, Program>,
    funding: Option<ExecutionMemoryReservation>,
}

/// Actual stopped owner; neither a refusal nor retry recreates Query service.
pub struct WorthQueryRetiringHomeQuery<Schema, Program> {
    runtime: RetiringPrimaryGraphRuntime<Schema>,
    _backing: Arc<WorthQueryProgramRuntimeBacking<Schema, Program>>,
    _funding: Option<ExecutionMemoryReservation>,
}

pub enum WorthQueryHomeRetirementAdvance<Schema, Program> {
    Advanced(WorthQueryRetiringHomeQuery<Schema, Program>),
    Complete(WorthQueryRetiredHomeQuery),
}

/// Retains the original inert payload after actual operational retirement.
/// Its destruction belongs to explicit unbudgeted reclaim or abandonment.
pub struct WorthQueryRetiredHomeQuery {
    _payload: Box<dyn std::any::Any + Send + Sync>,
}

impl WorthQueryRetiredHomeQuery {
    pub fn reclaim(self) {
        drop(self);
    }
}

impl<Schema: ApplicationSchema, Program> WorthQueryProgramApplicationRuntime<Schema, Program> {
    #[doc(hidden)]
    pub fn prepare_home_retirement(
        mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryPreparedHomeQuery<Schema, Program>, (Self, Denial)> {
        let admitted = request
            .run_native_completion(|_| {
                request.consult().map_err(Denial::Execution)?;
                let backing =
                    Arc::get_mut(&mut self.backing).ok_or(Denial::Busy(Owner::ProgramRuntime))?;
                // The exact future erased allocation is funded before any hook or seal.
                let funding = request
                    .reserve_retained_memory(std::mem::size_of::<
                        WorthQueryRetiringHomeQuery<Schema, Program>,
                    >() as u64)
                    .map_err(Denial::Execution)?;
                backing.runtime_mut().preflight_home_retirement(request)?;
                Ok(funding)
            })
            .map(|(admitted, _)| admitted)
            .unwrap_or_else(|cause| Err(Denial::Execution(cause)));
        match admitted {
            Ok(funding) => Ok(WorthQueryPreparedHomeQuery {
                query: self,
                funding,
            }),
            Err(cause) => Err((self, cause)),
        }
    }
}

impl<Schema: ApplicationSchema, Program> WorthQueryPreparedHomeQuery<Schema, Program> {
    /// Pre-seal refusal returns the same Query with its original Some runtime.
    pub fn resume(self) -> WorthQueryProgramApplicationRuntime<Schema, Program> {
        self.query
    }

    /// Closed to the Host's actual successful World-seal handoff.
    #[doc(hidden)]
    pub fn begin_after_world_seal(mut self) -> WorthQueryRetiringHomeQuery<Schema, Program> {
        let runtime = Arc::get_mut(&mut self.query.backing)
            .expect("prepared original Query cannot acquire a new alias")
            .runtime
            .take()
            .expect("only this consuming phase extracts the original runtime");
        WorthQueryRetiringHomeQuery {
            runtime: runtime.into_home_retirement(),
            _backing: self.query.backing,
            _funding: self.funding,
        }
    }
}

impl<Schema: ApplicationSchema, Program: 'static> WorthQueryRetiringHomeQuery<Schema, Program> {
    pub fn advance(
        mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryHomeRetirementAdvance<Schema, Program>, (Self, Denial)> {
        let advanced = request
            .run_native_completion(|_| self.runtime.advance(request))
            .map(|(advanced, _)| advanced)
            .unwrap_or_else(|cause| Err(Denial::Execution(cause)));
        match advanced {
            Ok(Progress::Advanced) => Ok(WorthQueryHomeRetirementAdvance::Advanced(self)),
            Ok(Progress::Complete) => Ok(WorthQueryHomeRetirementAdvance::Complete(
                WorthQueryRetiredHomeQuery {
                    _payload: Box::new(self),
                },
            )),
            Err(cause) => Err((self, cause)),
        }
    }
}
