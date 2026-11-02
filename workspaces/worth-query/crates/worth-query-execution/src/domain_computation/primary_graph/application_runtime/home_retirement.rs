//! Original Query custody survives every admission or retirement refusal.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::{
    handler::HandlerRetirementCursor, WorthQueryOperationHomeRetirementDenial,
    WorthQueryOperationHomeRetirementProgress as Progress,
};
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::ApplicationSchema;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorthQueryHomeRetirementOwner {
    ProgramRuntime,
    ConditionalRuntime,
    InstalledResources,
    InstalledProducers,
    InstalledConditionals,
    InboundVerifiers,
    ExternalTransport,
    SigningOwners,
    RelationalSource,
}

#[derive(Debug)]
pub enum WorthQueryHomeRetirementDenial {
    Busy(WorthQueryHomeRetirementOwner),
    Unsupported(WorthQueryHomeRetirementOwner),
    Handler {
        binding: &'static str,
        cause: WorthQueryOperationHomeRetirementDenial,
    },
    Execution(worth_execution::WorkCeilingDenial),
}

/// Keeps the original runtime and handler cursor, never a replacement service.
pub(in crate::domain_computation::primary_graph) struct RetiringPrimaryGraphRuntime<Schema> {
    runtime: WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    handlers: HandlerRetirementCursor,
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn preflight_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryHomeRetirementDenial> {
        request
            .consult()
            .map_err(WorthQueryHomeRetirementDenial::Execution)?;
        self.mutation_handlers.preflight_home_retirement(request)?;
        self.conditional_cleanup
            .preflight_home_retirement(request)?;
        self.installed_producers.preflight_home_retirement()?;
        self.installed_conditionals.preflight_home_retirement()?;
        self.preflight_supplied_home_owners()?;
        // The actual source owner, not the durable roster, decides whether
        // its Native handle/subscription/head custody has a consuming path.
        self.runtime
            .primary_graph()
            .expect("an installed application owns its original primary graph")
            .preflight_home_retirement(request)
    }

    fn preflight_supplied_home_owners(&mut self) -> Result<(), WorthQueryHomeRetirementDenial> {
        use WorthQueryHomeRetirementDenial::{Busy, Unsupported};
        use WorthQueryHomeRetirementOwner as Owner;
        let verifiers = self
            .inbound_verifiers
            .get_mut()
            .map_err(|_| Busy(Owner::InboundVerifiers))?;
        if !verifiers.is_empty() {
            return Err(Unsupported(Owner::InboundVerifiers));
        }
        if self.external_effect_transport.get().is_some() {
            return Err(Unsupported(Owner::ExternalTransport));
        }
        if !self
            .signing_owners
            .get_mut()
            .map_err(|_| Busy(Owner::SigningOwners))?
            .is_empty()
        {
            return Err(Unsupported(Owner::SigningOwners));
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn into_home_retirement(
        self,
    ) -> RetiringPrimaryGraphRuntime<Schema> {
        RetiringPrimaryGraphRuntime {
            runtime: self,
            handlers: Default::default(),
        }
    }
}

impl<Schema: ApplicationSchema> RetiringPrimaryGraphRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Progress, WorthQueryHomeRetirementDenial> {
        match self
            .runtime
            .mutation_handlers
            .advance_home_retirement(&mut self.handlers, request)?
        {
            Progress::Advanced => Ok(Progress::Advanced),
            Progress::Complete => {
                // Source retirement remains unsupported until its real Native,
                // Bridge and provider owners supply their consuming operation.
                // It cannot be inferred from the handler observation above.
                self.runtime
                    .runtime
                    .primary_graph()
                    .expect("retirement retains its original primary graph")
                    .advance_home_retirement(request)
            }
        }
    }
}
