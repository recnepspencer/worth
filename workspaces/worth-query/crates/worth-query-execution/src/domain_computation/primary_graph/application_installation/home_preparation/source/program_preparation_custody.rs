//! Exact advanced installation custody survives every ordinary Home preparation refusal.
mod contribution_installation;
use super::super::*;
use super::program_preparation_denial::QueryHomeSignalProgramPreparationDenial as D;
use super::{authored_bindings::QueryHomeAuthoredBindings, program_state::HomeServingProgramState};
use crate::domain_computation::primary_graph::application_contribution::producer::PendingProducerRegistry;
use crate::domain_computation::primary_graph::{
    application_contribution::*,
    application_runtime::installation::{
        home_attachment::HomeApplicationServices, ApplicationRuntimePublication,
    },
    conditional_operation::WorthQueryConditionalApplicationRuntimeInstallation,
};
use worth_runtime_bridge::facade::{
    BridgeConditionalRuntimeBuilder, RuntimeBridge, RuntimeBridgeRelationalSource,
};

pub(super) struct NativeProgramInput<S: ApplicationSchemaComposition, P> {
    pub(super) state: Box<HomeApplicationState<S, P>>,
    pub(super) schema: Box<InstalledNativeSchemaMeaning>,
    pub(super) restore: Option<worth_relational::facade::durability::CheckpointRestoreWork>,
    pub(super) source: RuntimeBridgeRelationalSource,
    pub(super) ordinary: RuntimeBridge,
    pub(super) seed: Option<super::super::super::super::bootstrap::WorthQueryProgramActivationSeed>,
}
struct RuntimeMeaning<S: ApplicationSchemaComposition> {
    runtime: WorthQueryExecutionRuntime,
    authority: WorthQueryExecutionInstallationAuthority,
    installed: WorthQueryInstalledApplicationSchema<S>,
}
struct ProgramCore<S: ApplicationSchemaComposition, P> {
    program: WorthQueryInstalledApplicationProgram<S, P>,
    supported: Vec<WorthQuerySupportedProgramRecord>,
    support: program_admission::WorthQueryAdmittedProgramSupport<S>,
    activation: WorthQueryProgramActivationCell,
    transition: Option<
        worth_proof::AdmittedHomeProgramTransitionSuccessor<
            worth_foundational::facade::HomeProgramTransitionInstallationBinding,
        >,
    >,
    limits: WorthQueryApplicationLimits,
    build: worth_foundational::facade::ApplicationBuildIdentity,
    clock: Option<crate::domain_computation::authorization::WorthQueryRuntimeClock>,
}
pub(super) struct HomeProgramInstallationCustody<S:ApplicationSchemaComposition,P> {
    core:ProgramCore<S,P>,
    meaning:Option<RuntimeMeaning<S>>,
    pub(super) bootstrap:Option<Box<super::super::super::super::WorthQueryPrimaryGraphBootstrap<S>>>,
    pending_producers:Option<PendingProducerRegistry<S>>,
    pending_conditionals:Option<crate::domain_computation::primary_graph::application_contribution::conditional::PendingConditionalRegistry<S>>,
    pub(super) producer_refusal:Option<RefusedProducerSeal<S>>,
    pub(super) conditional_refusal:Option<RefusedConditionalBatch<S>>,
    publication:Option<Box<ApplicationRuntimePublication<S>>>,
    conditional:Option<Box<WorthQueryConditionalApplicationRuntimeInstallation<S>>>,
    parts:Option<crate::domain_computation::primary_graph::conditional_operation::installation::home_program::HomeConditionalProgramParts<S>>,
    pub(super) producers:Option<WorthQueryInstalledApplicationProducerRegistry<S>>,
    pub(super) conditionals:Option<WorthQueryInstalledApplicationConditionalRegistry<S>>,
    pub(super) services:Option<HomeApplicationServices<S>>,
    pub(super) builder:Option<Box<BridgeConditionalRuntimeBuilder>>,
    pub(super) readiness:Vec<Box<dyn PendingOutputReadiness<S>>>,
    temporal_pending: Vec<Box<dyn crate::domain_computation::primary_graph::conditional_operation::WorthQueryPendingConditionalOperation<S>>>,
    pub(super) operations: Option<crate::domain_computation::primary_graph::conditional_operation::lifecycle::WorthQueryConditionalOperationRegistry<S>>,
    pub(super) definition_pending: Vec<Box<dyn crate::domain_computation::primary_graph::conditional_operation::PendingDefinitionDeclaration>>,
    pub(super) definition_routes: Option<crate::domain_computation::primary_graph::conditional_operation::definition_declarations::ConditionalDefinitionDeclarations>,
    pub(super) producer_routes:Option<WorthQueryInstalledOutputProducerRoutes>,
    pub(super) readiness_routes:Option<WorthQueryInstalledOutputReadinessRoutes>,
    pub(super) bindings:Option<QueryHomeAuthoredBindings>,
    _source:RuntimeBridgeRelationalSource,
    pub(super) ordinary:RuntimeBridge,
}
pub struct QueryHomeSignalProgramPreparationRefused<S: ApplicationSchemaComposition, P> {
    _custody: RefusalCustody<S, P>,
    cause: D,
}
enum RefusalCustody<S: ApplicationSchemaComposition, P> {
    Native {
        _input: Box<NativeProgramInput<S, P>>,
    },
    Installation {
        _installation: Box<HomeProgramInstallationCustody<S, P>>,
    },
}
impl<S: ApplicationSchemaComposition, P> QueryHomeSignalProgramPreparationRefused<S, P> {
    pub fn reason(&self) -> &D {
        &self.cause
    }
    pub(super) fn native(
        input: Box<NativeProgramInput<S, P>>,
        cause: WorthQueryPrimaryGraphInstallationDenial,
    ) -> Self {
        Self {
            _custody: RefusalCustody::Native { _input: input },
            cause: D::Source(cause),
        }
    }
}
impl<S: ApplicationSchemaComposition, P> HomeProgramInstallationCustody<S, P> {
    pub(super) fn from_meaning(
        state: HomeApplicationState<S, P>,
        bootstrap: super::super::super::super::WorthQueryPrimaryGraphBootstrap<S>,
        source: RuntimeBridgeRelationalSource,
        ordinary: RuntimeBridge,
    ) -> Box<Self> {
        let HomeApplicationState {
            runtime,
            authority,
            installed,
            program,
            supported,
            support,
            pending,
            activation,
            transition_installation,
            limits,
            build_identity,
            authorization_time_source,
        } = state;
        let PreparedApplicationRuntimeContributions {
            handlers,
            producers,
            conditionals,
            resources,
            managed_indexes,
        } = pending;
        // assembly::prepare already moved these exact registries into bootstrap.
        drop((handlers, resources, managed_indexes));
        let clock = match authorization_time_source {
            Some(source) => {
                crate::domain_computation::authorization::WorthQueryRuntimeClock::from_source(
                    source,
                )
            }
            None => crate::domain_computation::authorization::WorthQueryRuntimeClock::system(),
        };
        Box::new(Self {
            core: ProgramCore {
                program,
                supported,
                support,
                activation,
                transition: transition_installation,
                limits,
                build: build_identity,
                clock: Some(clock),
            },
            meaning: Some(RuntimeMeaning {
                runtime,
                authority,
                installed,
            }),
            bootstrap: Some(Box::new(bootstrap)),
            pending_producers: Some(producers),
            pending_conditionals: Some(conditionals),
            producer_refusal: None,
            conditional_refusal: None,
            publication: None,
            conditional: None,
            parts: None,
            producers: None,
            conditionals: None,
            services: None,
            builder: None,
            readiness: Vec::new(),
            temporal_pending: Vec::new(),
            operations: None,
            definition_pending: Vec::new(),
            definition_routes: None,
            producer_routes: None,
            readiness_routes: None,
            bindings: None,
            _source: source,
            ordinary,
        })
    }
    pub(super) fn refuse(
        self: Box<Self>,
        cause: D,
    ) -> QueryHomeSignalProgramPreparationRefused<S, P> {
        QueryHomeSignalProgramPreparationRefused {
            _custody: RefusalCustody::Installation {
                _installation: self,
            },
            cause,
        }
    }
    pub(super) fn conditional_budget(
        &self,
    ) -> worth_signal::facade::runtime::SignalConditionalEvaluationBudget {
        self.core.limits.conditionals
    }
    pub(super) fn finish(
        mut self: Box<Self>,
    ) -> super::program_installation::HomeSignalProgramParts<S, P> {
        let RuntimeMeaning {
            runtime,
            authority: _,
            installed,
        } = self.meaning.take().expect("original prepared meaning");
        let ProgramCore {
            program,
            supported,
            support,
            activation,
            transition,
            limits,
            build,
            clock,
        } = self.core;
        let state = HomeServingProgramState {
            runtime,
            installed,
            program,
            supported,
            support,
            activation,
            transition_installation: transition,
            limits,
            build_identity: build,
            authorization_clock: clock.expect("same original admitted clock"),
        };
        let bindings = self
            .bindings
            .take()
            .expect("actual fresh programme bindings");
        let prepared = super::program_installation::HomeServingProgramPreparation {
            state,
            bootstrap: *self.bootstrap.take().expect("actual bootstrap"),
            services: self.services.take().expect("actual installed services"),
            ordinary: self.ordinary,
            installation: bindings.installation_binding(),
            producers: self.producers.take().expect("original installed producers"),
            conditionals: self
                .conditionals
                .take()
                .expect("original installed conditionals"),
            producer_routes: self
                .producer_routes
                .take()
                .expect("original producer routes"),
            operations: self.operations.take().expect("installed temporal routes"),
            definition_routes: self
                .definition_routes
                .take()
                .expect("original installed definition declarations"),
            readiness_routes: self
                .readiness_routes
                .take()
                .expect("original readiness routes"),
        };
        super::program_installation::HomeSignalProgramParts {
            prepared,
            builder: self.builder.take().expect("same actual programme builder"),
            bindings,
        }
    }
}

impl<S: ApplicationSchemaComposition, P> std::fmt::Debug
    for QueryHomeSignalProgramPreparationRefused<S, P>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryHomeSignalProgramPreparationRefused")
            .field("cause", &self.cause)
            .finish_non_exhaustive()
    }
}
