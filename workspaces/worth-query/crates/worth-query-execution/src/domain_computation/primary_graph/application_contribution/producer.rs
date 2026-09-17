use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
    ApplicationQueryMutationSource,
};
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::ApplicationSchema;

mod demand;
pub use demand::{
    WorthQueryAdmittedOutputDemand, WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQuerySelectedApplicationProducer,
};
mod execution;
use execution::{InstalledProducerExecutor, TypedInstalledProducer};

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryProducerCommitAuthority {
    Ordinary,
    ProgramOutput,
}
mod readiness;
pub(in crate::domain_computation::primary_graph) use readiness::{
    evaluate_output_readiness, install_output_readiness_routes, PendingOutputReadiness,
    TypedPendingOutputReadiness, WorthQueryInstalledOutputReadinessRoutes,
};
mod scheduling;
pub(in crate::domain_computation::primary_graph) use scheduling::{
    schedule_output_producer, WorthQueryInstalledOutputProducerRoutes,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryProducerLifecyclePosture {
    Initial,
    Preserve,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryProducerApplicability {
    profile_kind: &'static str,
    lifecycle: WorthQueryProducerLifecyclePosture,
}

impl WorthQueryProducerApplicability {
    pub const fn new(
        profile_kind: &'static str,
        lifecycle: WorthQueryProducerLifecyclePosture,
    ) -> Self {
        Self {
            profile_kind,
            lifecycle,
        }
    }

    pub const fn profile_kind(self) -> &'static str {
        self.profile_kind
    }

    pub const fn lifecycle(self) -> WorthQueryProducerLifecyclePosture {
        self.lifecycle
    }
}

pub trait WorthQueryProducerOutputFamily<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Source: ApplicationQueryBinding<Schema>;

    const IDENTITY: &'static str;
    const SUPPORTED: &'static [WorthQueryProducerApplicability];

    fn profile_kind(
        source: &<<Self::Source as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
    ) -> &'static str;
}

/// Typed request for one installed output family from one authored source.
pub trait WorthQueryApplicationOutputDemand<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type OutputFamily: WorthQueryProducerOutputFamily<Schema>;

    fn source_intent(
        &self,
    ) -> <<Self::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::Input;
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryProducerInvariantRequirement {
    identifier: &'static str,
    major: u16,
    minor: u16,
    execution_point: ApplicationInvariantExecutionPoint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryProducerDemandResources {
    work: usize,
    retained_bytes: usize,
}

impl WorthQueryProducerDemandResources {
    pub const fn new(work: usize, retained_bytes: usize) -> Self {
        Self {
            work,
            retained_bytes,
        }
    }

    pub const fn work(self) -> usize {
        self.work
    }

    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }
}

impl WorthQueryProducerInvariantRequirement {
    pub const fn new(
        identifier: &'static str,
        major: u16,
        minor: u16,
        execution_point: ApplicationInvariantExecutionPoint,
    ) -> Self {
        Self {
            identifier,
            major,
            minor,
            execution_point,
        }
    }

    pub const fn identifier(self) -> &'static str {
        self.identifier
    }

    pub const fn major(self) -> u16 {
        self.major
    }

    pub const fn minor(self) -> u16 {
        self.minor
    }

    pub const fn execution_point(self) -> ApplicationInvariantExecutionPoint {
        self.execution_point
    }
}

pub trait WorthQueryApplicationProducerProvider<Schema, Binding>: Send + Sync + 'static
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    const SEMANTIC_IDENTITY: &'static str;

    fn operation_input(
        &self,
        source: &<<<<Binding as WorthQueryApplicationProducerBinding<Schema>>::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
    ) -> <Binding::Operation as ApplicationMutationBinding<Schema>>::Input;

    fn idempotency_key(
        &self,
        source: &<<<<Binding as WorthQueryApplicationProducerBinding<Schema>>::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
        source_identity: &[u8; 32],
    ) -> <Binding::Operation as ApplicationMutationBinding<Schema>>::IdempotencyKey;

    fn demand_resources(
        &self,
        source: &<<<<Binding as WorthQueryApplicationProducerBinding<Schema>>::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
    ) -> WorthQueryProducerDemandResources;
}

pub trait WorthQueryApplicationProducerBinding<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
    Self::Operation: ApplicationMutationBinding<
        Schema,
        SourceExpectation = ApplicationQueryMutationSource<
            <<Self::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::Query,
        >,
    >,
    <Self::Operation as ApplicationMutationBinding<Schema>>::Input:
        ApplicationMutationIntent<Schema, Binding = Self::Operation> + Clone + Send + Sync,
    <Self::Operation as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Self::Operation as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
    <Self::Operation as ApplicationMutationBinding<Schema>>::Denial: std::fmt::Debug,
{
    type Operation: ApplicationMutationBinding<Schema>;
    type OutputFamily: WorthQueryProducerOutputFamily<Schema>;
    type Provider: WorthQueryApplicationProducerProvider<Schema, Self>;

    const IDENTITY: &'static str;
    const OUTPUT_ROLE: &'static str;
    const APPLICABILITY: &'static [WorthQueryProducerApplicability];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement];
    const RESOURCE_POLICY: &'static str;
    const REUSE_POLICY: &'static str;
}

mod registry;
pub use registry::WorthQueryInstalledApplicationProducerRegistry;
pub(super) use registry::{DeclaredProducerBinding, PendingProducerRegistry};
