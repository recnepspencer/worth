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
    state: Box<HomeQueryRetirementState<Schema, Program>>,
    funding: Option<ExecutionMemoryReservation>,
}

/// Actual stopped owner; neither a refusal nor retry recreates Query service.
pub struct WorthQueryRetiringHomeQuery<Schema, Program> {
    state: Box<HomeQueryRetirementState<Schema, Program>>,
    _funding: Option<ExecutionMemoryReservation>,
}

struct HomeQueryRetirementState<Schema, Program> {
    runtime: Option<RetiringPrimaryGraphRuntime<Schema>>,
    _backing: Option<Arc<WorthQueryProgramRuntimeBacking<Schema, Program>>>,
    retired_home_maintenance: Option<Arc<dyn std::any::Any + Send + Sync>>,
}

pub enum WorthQueryHomeRetirementAdvance<Schema, Program> {
    Advanced(WorthQueryRetiringHomeQuery<Schema, Program>),
    Complete(WorthQueryRetiredHomeQuery),
}

/// Retains the original inert payload after actual operational retirement.
/// Its destruction belongs to explicit unbudgeted reclaim or abandonment.
pub struct WorthQueryRetiredHomeQuery {
    _payload: Box<dyn std::any::Any + Send + Sync>,
    _funding: Option<ExecutionMemoryReservation>,
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
                        HomeQueryRetirementState<Schema, Program>,
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
                state: Box::new(HomeQueryRetirementState {
                    runtime: None,
                    _backing: None,
                    retired_home_maintenance: None,
                }),
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
        self.state.runtime = Some(runtime.into_home_retirement());
        self.state._backing = Some(self.query.backing);
        WorthQueryRetiringHomeQuery {
            state: self.state,
            _funding: self.funding,
        }
    }
}

impl<Schema: ApplicationSchema, Program: 'static> WorthQueryRetiringHomeQuery<Schema, Program> {
    /// Closed Host handoff of its original maintenance capability after seal.
    #[doc(hidden)]
    pub fn retire_home_maintenance<Cause>(
        &mut self,
        retire: impl FnOnce(&mut dyn crate::domain_computation::execution_runtime::product_world::WorthQueryHomeMaintenance) -> Result<(), Cause>,
    ) -> Result<Result<(), Cause>, Denial> {
        let Some(original) = self
            .state
            .runtime
            .as_mut()
            .expect("sealed original runtime")
            .home_maintenance_mut()
        else {
            return Ok(Ok(()));
        };
        let original = Arc::get_mut(original).ok_or(Denial::Busy(Owner::HomeMaintenance))?;
        let result = retire(original);
        if result.is_ok() {
            self.state.retired_home_maintenance = self
                .state
                .runtime
                .as_mut()
                .expect("sealed original runtime")
                .take_home_maintenance()
                .map(|original| original as Arc<dyn std::any::Any + Send + Sync>);
        }
        Ok(result)
    }
    pub fn advance(
        mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        bridge_owner: &std::sync::Arc<
            std::sync::RwLock<worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly>,
        >,
    ) -> Result<WorthQueryHomeRetirementAdvance<Schema, Program>, (Self, Denial)> {
        // Every original owner consults this same request. Retirement opens no
        // fresh execution scope after the close checkpoint exhausts memory.
        let advanced = self
            .state
            .runtime
            .as_mut()
            .expect("sealed original runtime")
            .advance(request, native, bridge_owner);
        match advanced {
            Ok(Progress::Advanced) => Ok(WorthQueryHomeRetirementAdvance::Advanced(self)),
            Ok(Progress::Complete) => Ok(WorthQueryHomeRetirementAdvance::Complete(
                WorthQueryRetiredHomeQuery {
                    _payload: self.state,
                    _funding: self._funding,
                },
            )),
            Err(cause) => Err((self, cause)),
        }
    }
}
